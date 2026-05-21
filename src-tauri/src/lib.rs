use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use tauri::{Manager, State};

#[derive(Debug, Default)]
pub struct AppState {
    pub settings: Mutex<HashMap<String, String>>,
}

// --- Settings commands ---

#[tauri::command]
fn get_setting(key: String, state: State<'_, Arc<AppState>>) -> Option<String> {
    state.settings.lock().get(&key).cloned()
}

#[tauri::command]
fn set_setting(key: String, value: String, state: State<'_, Arc<AppState>>) {
    state.settings.lock().insert(key, value);
}

// --- Git diff types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub filename: String,
    pub status: String, // added, removed, modified, renamed
    pub additions: u32,
    pub deletions: u32,
    pub patch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub base: String,
    pub head: String,
    pub files: Vec<FileDiff>,
    pub total_additions: u32,
    pub total_deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSummary {
    pub filename: String,
    pub summary: String,
}

// --- GitHub branches command ---

#[tauri::command]
fn fetch_github_branches(owner: String, repo: String) -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::new();
    let mut branches = Vec::new();
    let mut page = 1;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/{}/branches?per_page=100&page={}",
            owner, repo, page
        );
        let resp = client
            .get(&url)
            .header("User-Agent", "lazydiff")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .map_err(|e| format!("Failed to fetch branches: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("GitHub API error: {}", resp.status()));
        }

        let json: Vec<serde_json::Value> =
            resp.json().map_err(|e| format!("Failed to parse branches: {}", e))?;

        if json.is_empty() {
            break;
        }
        for b in &json {
            if let Some(name) = b["name"].as_str() {
                branches.push(name.to_string());
            }
        }
        if json.len() < 100 {
            break;
        }
        page += 1;
    }

    branches.sort();
    Ok(branches)
}

// --- GitHub compare (direct params) ---

#[tauri::command]
fn fetch_github_compare(owner: String, repo: String, base: String, head: String) -> Result<DiffResult, String> {
    let compare_url = format!(
        "https://api.github.com/repos/{}/{}/compare/{}...{}",
        owner, repo, base, head
    );
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&compare_url)
        .header("User-Agent", "lazydiff")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|e| format!("Failed to fetch diff: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API error: {}", resp.status()));
    }

    let json: serde_json::Value =
        resp.json().map_err(|e| format!("Failed to parse diff: {}", e))?;

    let files = json["files"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|f| FileDiff {
            filename: f["filename"].as_str().unwrap_or("").to_string(),
            status: f["status"].as_str().unwrap_or("modified").to_string(),
            additions: f["additions"].as_u64().unwrap_or(0) as u32,
            deletions: f["deletions"].as_u64().unwrap_or(0) as u32,
            patch: f["patch"].as_str().unwrap_or("").to_string(),
        })
        .collect::<Vec<_>>();

    let total_additions = files.iter().map(|f| f.additions).sum();
    let total_deletions = files.iter().map(|f| f.deletions).sum();

    Ok(DiffResult {
        base,
        head,
        files,
        total_additions,
        total_deletions,
    })
}

// --- GitHub API command (URL-based, kept for PR URLs) ---

#[tauri::command]
fn fetch_github_diff(url: String) -> Result<DiffResult, String> {
    // Parse PR URL: https://github.com/owner/repo/pull/123
    // Parse branch URL: https://github.com/owner/repo/tree/branch-name
    // Parse compare URL: https://github.com/owner/repo/compare/base...head
    let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();

    let (owner, repo, base, head) = if let Some(pos) = parts.iter().position(|&p| p == "pull") {
        let owner = parts.get(pos - 2).ok_or("Invalid PR URL")?;
        let repo = parts.get(pos - 1).ok_or("Invalid PR URL")?;
        let pr_num = parts.get(pos + 1).ok_or("Invalid PR URL")?;
        // Fetch PR to get base/head branches
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_num
        );
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&pr_url)
            .header("User-Agent", "lazydiff")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .map_err(|e| format!("Failed to fetch PR: {}", e))?;
        let json: serde_json::Value =
            resp.json().map_err(|e| format!("Failed to parse PR: {}", e))?;
        let base_ref = json["base"]["ref"]
            .as_str()
            .unwrap_or("main")
            .to_string();
        let head_ref = json["head"]["ref"]
            .as_str()
            .unwrap_or("HEAD")
            .to_string();
        (
            owner.to_string(),
            repo.to_string(),
            base_ref,
            head_ref,
        )
    } else if let Some(pos) = parts.iter().position(|&p| p == "compare") {
        let owner = parts.get(pos - 2).ok_or("Invalid compare URL")?;
        let repo = parts.get(pos - 1).ok_or("Invalid compare URL")?;
        let compare = parts.get(pos + 1).ok_or("Invalid compare URL")?;
        let branches: Vec<&str> = compare.split("...").collect();
        if branches.len() != 2 {
            return Err("Invalid compare URL format, expected base...head".into());
        }
        (
            owner.to_string(),
            repo.to_string(),
            branches[0].to_string(),
            branches[1].to_string(),
        )
    } else if let Some(pos) = parts.iter().position(|&p| p == "tree") {
        let owner = parts.get(pos - 2).ok_or("Invalid branch URL")?;
        let repo = parts.get(pos - 1).ok_or("Invalid branch URL")?;
        let branch = parts[pos + 1..].join("/");
        (
            owner.to_string(),
            repo.to_string(),
            "main".to_string(),
            branch,
        )
    } else {
        return Err(
            "Unrecognized URL format. Use a PR, branch, or compare URL from GitHub.".into(),
        );
    };

    // Fetch compare diff
    let compare_url = format!(
        "https://api.github.com/repos/{}/{}/compare/{}...{}",
        owner, repo, base, head
    );
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&compare_url)
        .header("User-Agent", "lazydiff")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|e| format!("Failed to fetch diff: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API error: {}", resp.status()));
    }

    let json: serde_json::Value =
        resp.json().map_err(|e| format!("Failed to parse diff: {}", e))?;

    let files = json["files"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|f| FileDiff {
            filename: f["filename"].as_str().unwrap_or("").to_string(),
            status: f["status"].as_str().unwrap_or("modified").to_string(),
            additions: f["additions"].as_u64().unwrap_or(0) as u32,
            deletions: f["deletions"].as_u64().unwrap_or(0) as u32,
            patch: f["patch"].as_str().unwrap_or("").to_string(),
        })
        .collect::<Vec<_>>();

    let total_additions = files.iter().map(|f| f.additions).sum();
    let total_deletions = files.iter().map(|f| f.deletions).sum();

    Ok(DiffResult {
        base,
        head,
        files,
        total_additions,
        total_deletions,
    })
}

// --- Local git diff command ---

#[tauri::command]
fn fetch_local_diff(repo_path: String, branch: String, base_branch: Option<String>) -> Result<DiffResult, String> {
    // Use provided base or detect main/master
    let base = match base_branch {
        Some(ref b) if !b.is_empty() => b.as_str(),
        _ => {
            let check_main = Command::new("git")
                .args(["rev-parse", "--verify", "main"])
                .current_dir(&repo_path)
                .output();
            if check_main.map(|o| o.status.success()).unwrap_or(false) {
                "main"
            } else {
                "master"
            }
        }
    };

    // Get diff stat
    let output = Command::new("git")
        .args(["diff", "--numstat", &format!("{}...{}", base, branch)])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    let numstat = String::from_utf8_lossy(&output.stdout);

    // Get patch
    let patch_output = Command::new("git")
        .args(["diff", "-U3", &format!("{}...{}", base, branch)])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to get patch: {}", e))?;

    let full_patch = String::from_utf8_lossy(&patch_output.stdout).to_string();

    // Parse file patches
    let file_patches: HashMap<String, String> = parse_file_patches(&full_patch);

    let mut files = Vec::new();
    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let additions = parts[0].parse::<u32>().unwrap_or(0);
            let deletions = parts[1].parse::<u32>().unwrap_or(0);
            let filename = parts[2].to_string();
            let status = if additions > 0 && deletions > 0 {
                "modified"
            } else if additions > 0 {
                "added"
            } else {
                "removed"
            };
            let patch = file_patches.get(&filename).cloned().unwrap_or_default();
            files.push(FileDiff {
                filename,
                status: status.to_string(),
                additions,
                deletions,
                patch,
            });
        }
    }

    let total_additions = files.iter().map(|f| f.additions).sum();
    let total_deletions = files.iter().map(|f| f.deletions).sum();

    Ok(DiffResult {
        base: base.to_string(),
        head: branch,
        files,
        total_additions,
        total_deletions,
    })
}

fn parse_file_patches(full_patch: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut current_file: Option<String> = None;
    let mut current_patch = String::new();

    for line in full_patch.lines() {
        if line.starts_with("diff --git") {
            if let Some(ref file) = current_file {
                result.insert(file.clone(), current_patch.clone());
            }
            // Extract filename from "diff --git a/path b/path"
            let parts: Vec<&str> = line.split(" b/").collect();
            current_file = parts.get(1).map(|s| s.to_string());
            current_patch = String::new();
        } else if current_file.is_some() {
            current_patch.push_str(line);
            current_patch.push('\n');
        }
    }
    if let Some(file) = current_file {
        result.insert(file, current_patch);
    }
    result
}

// --- List local branches ---

#[tauri::command]
fn list_branches(repo_path: String) -> Result<Vec<String>, String> {
    // List both local and remote branches
    let output = Command::new("git")
        .args(["branch", "-a", "--format=%(refname:short)"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to list branches: {}", e))?;

    let mut seen = std::collections::HashSet::new();
    let mut branches = Vec::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.contains("HEAD") {
            continue;
        }
        // Strip "origin/" prefix for remote branches
        let name = trimmed
            .strip_prefix("origin/")
            .unwrap_or(trimmed)
            .to_string();
        if seen.insert(name.clone()) {
            branches.push(name);
        }
    }

    branches.sort();
    Ok(branches)
}

// --- Current branch ---

#[tauri::command]
fn current_branch(repo_path: String) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| format!("Failed to get current branch: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// --- Claude API command ---

#[tauri::command]
fn summarize_with_claude(api_key: String, file_diffs: Vec<FileDiff>, style: Option<String>) -> Result<Vec<AiSummary>, String> {
    let client = reqwest::blocking::Client::new();
    let is_caveman = style.as_deref() == Some("caveman");

    let mut summaries = Vec::new();

    for file in &file_diffs {
        if file.patch.is_empty() {
            summaries.push(AiSummary {
                filename: file.filename.clone(),
                summary: format!("File {} (no patch content available)", file.status),
            });
            continue;
        }

        let prompt = if is_caveman {
            format!(
                r#"Analyze code diff. Reply EXACTLY this format. Use CAVEMAN COMPRESSION style:
- Strip grammar. Keep facts. 2-5 words per sentence.
- No connectives (because, however, therefore). No filler words.
- Active voice. Present tense. Action verbs.
- Keep specifics: numbers, names, technical terms.

Format:

ROLE: [2-5 word description. What file do.]
CALLS: [Short names comma-separated. Or "nothing".]
CALLED_BY: [Short names comma-separated. Or "unknown".]
CHANGES:
- [Max 5 bullets. Caveman style. "Add index." "Remove old auth." "Change timeout 30s to 60s."]

File: {}
Status: {}
Diff:
```
{}
```"#,
                file.filename, file.status, file.patch
            )
        } else {
            format!(
                r#"Analyze this code diff. Reply with EXACTLY this format (keep each section to one line where possible):

ROLE: [One sentence: what this file does in the project]
CALLS: [Comma-separated: what this file imports/calls/depends on, or "nothing" if standalone]
CALLED_BY: [Comma-separated: what likely calls/imports this file based on its exports, or "unknown"]
CHANGES:
- [bullet point describing what changed functionally, plain language, no code]
- [max 5 bullets]

Rules:
- ROLE must always be filled, even for new files. Infer from the code.
- For CALLS/CALLED_BY, use short names (e.g. "AuthService", "database", "API router"), not file paths.
- For CHANGES bullets: "Now does X" for additions, "No longer does X" for removals, "Instead of X, now does Y" for modifications.
- Keep everything very concise. Plain language, not code.

File: {}
Status: {}
Diff:
```
{}
```"#,
                file.filename, file.status, file.patch
            )
        };

        let body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 500,
            "messages": [{
                "role": "user",
                "content": prompt
            }]
        });

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Claude API error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(format!("Claude API error {}: {}", status, body));
        }

        let json: serde_json::Value =
            resp.json().map_err(|e| format!("Failed to parse Claude response: {}", e))?;

        let text = json["content"][0]["text"]
            .as_str()
            .unwrap_or("No summary available")
            .to_string();

        summaries.push(AiSummary {
            filename: file.filename.clone(),
            summary: text,
        });
    }

    Ok(summaries)
}

// --- Update checker ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub download_url: String,
    pub release_url: String,
}

#[tauri::command]
fn check_for_updates() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION");
    let url = "https://api.github.com/repos/jjolmo/lazydiff/releases/latest";
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "lazydiff")
        .send()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value =
        resp.json().map_err(|e| format!("Failed to parse: {}", e))?;
    let tag = json["tag_name"].as_str().unwrap_or("v0.0.0");
    let latest = tag.trim_start_matches('v');
    let release_url = json["html_url"].as_str().unwrap_or("").to_string();
    let mut download_url = String::new();
    if let Some(assets) = json["assets"].as_array() {
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            if name.ends_with(".AppImage") || name.ends_with(".msi") || name.ends_with(".dmg") {
                download_url = asset["browser_download_url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                break;
            }
        }
    }
    Ok(UpdateInfo {
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        has_update: latest != current,
        download_url,
        release_url,
    })
}

// --- Self-update (macOS) ---

#[tauri::command]
fn run_self_update() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"#!/bin/bash
REPO="jjolmo/lazydiff"
APP_NAME="LazyDiff.app"
INSTALL_DIR="/Applications"

RELEASE_JSON=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest")
PARSED=$(echo "$RELEASE_JSON" | python3 -c "
import sys, json
data = json.load(sys.stdin)
tag = data.get('tag_name', '')
dmg_url = ''
for asset in data.get('assets', []):
    if asset['name'].endswith('.dmg'):
        dmg_url = asset['browser_download_url']
        break
print(f'{tag}|{dmg_url}')
" 2>&1)

TAG=$(echo "$PARSED" | cut -d'|' -f1)
DMG_URL=$(echo "$PARSED" | cut -d'|' -f2)

[ -z "$TAG" ] && exit 1
[ -z "$DMG_URL" ] && exit 1

TMP_DIR=$(mktemp -d)
TMP_DMG="$TMP_DIR/lazydiff.dmg"
MOUNT_POINT="$TMP_DIR/mount"

curl -L --fail -o "$TMP_DMG" "$DMG_URL" || { rm -rf "$TMP_DIR"; exit 1; }

mkdir -p "$MOUNT_POINT"
hdiutil attach "$TMP_DMG" -mountpoint "$MOUNT_POINT" -nobrowse -quiet || { rm -rf "$TMP_DIR"; exit 1; }

[ ! -d "$MOUNT_POINT/$APP_NAME" ] && { hdiutil detach "$MOUNT_POINT" 2>/dev/null; rm -rf "$TMP_DIR"; exit 1; }

osascript -e 'quit app "LazyDiff"' 2>/dev/null || true
sleep 2
pkill -f "LazyDiff" 2>/dev/null || true
sleep 1

rm -rf "$INSTALL_DIR/$APP_NAME"
cp -R "$MOUNT_POINT/$APP_NAME" "$INSTALL_DIR/$APP_NAME"
xattr -cr "$INSTALL_DIR/$APP_NAME"

hdiutil detach "$MOUNT_POINT" 2>/dev/null || true
rm -rf "$TMP_DIR"

open "$INSTALL_DIR/$APP_NAME"
"#;
        let tmp_script = std::env::temp_dir().join("lazydiff_update.sh");
        std::fs::write(&tmp_script, script).map_err(|e| e.to_string())?;

        Command::new("chmod")
            .args(["+x", &tmp_script.to_string_lossy()])
            .output()
            .map_err(|e| e.to_string())?;

        // Run in background via osascript Terminal so user sees progress
        Command::new("osascript")
            .args([
                "-e",
                &format!(
                    "tell application \"Terminal\" to do script \"{}\"",
                    tmp_script.to_string_lossy()
                ),
            ])
            .spawn()
            .map_err(|e| e.to_string())?;

        Ok("Update started in Terminal".to_string())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Self-update is only supported on macOS. On Linux use your package manager or download the new AppImage/deb.".to_string())
    }
}

// --- Desktop entry (Linux) ---

#[tauri::command]
fn create_desktop_entry(app_handle: tauri::AppHandle) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").map_err(|e| e.to_string())?;
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe_path.to_string_lossy().to_string();
        let icons_dir = std::path::PathBuf::from(&home).join(".local/share/icons");
        std::fs::create_dir_all(&icons_dir).map_err(|e| e.to_string())?;
        let icon_dest = icons_dir.join("lazydiff.png");
        let resource_path = app_handle
            .path()
            .resource_dir()
            .map_err(|e| e.to_string())?;
        let icon_src = resource_path.join("icons/128x128.png");
        if icon_src.exists() {
            std::fs::copy(&icon_src, &icon_dest).map_err(|e| e.to_string())?;
        }
        let apps_dir = std::path::PathBuf::from(&home).join(".local/share/applications");
        std::fs::create_dir_all(&apps_dir).map_err(|e| e.to_string())?;
        let desktop_path = apps_dir.join("lazydiff.desktop");
        let content = format!(
            "[Desktop Entry]\nType=Application\nName=LazyDiff\nComment=Human-readable PR diff viewer\n\
             Exec={}\nIcon=lazydiff\nTerminal=false\nCategories=Development;\nStartupWMClass=lazydiff\n",
            exe_str
        );
        std::fs::write(&desktop_path, &content).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&desktop_path, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| e.to_string())?;
        }
        Ok(desktop_path.to_string_lossy().to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = app_handle;
        Err("Desktop entries are only supported on Linux".to_string())
    }
}

pub fn run() {
    let state = Arc::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_setting,
            set_setting,
            fetch_github_diff,
            fetch_github_compare,
            fetch_github_branches,
            fetch_local_diff,
            list_branches,
            current_branch,
            summarize_with_claude,
            check_for_updates,
            run_self_update,
            create_desktop_entry,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_title("LazyDiff").ok();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
