<script lang="ts">
  let {
    value = $bindable(''),
    items = [],
    placeholder = '',
    label = '',
  }: {
    value: string;
    items: string[];
    placeholder?: string;
    label?: string;
  } = $props();

  let focused = $state(false);
  let query = $state('');
  let highlightIndex = $state(-1);
  let inputEl: HTMLInputElement | undefined = $state();

  let filtered = $derived.by(() => {
    if (!focused) return [];
    const q = query.toLowerCase();
    return items.filter(i => !q || i.toLowerCase().includes(q)).slice(0, 30);
  });

  let showDropdown = $derived(focused && (filtered.length > 0 || (items.length === 0 && query === '')));

  function select(item: string) {
    value = item;
    query = item;
    focused = false;
    highlightIndex = -1;
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    query = target.value;
    value = target.value;
    highlightIndex = -1;
  }

  function handleFocus() {
    focused = true;
    query = value;
  }

  function handleBlur() {
    // Delay to allow click on dropdown
    setTimeout(() => { focused = false; }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightIndex = Math.min(highlightIndex + 1, filtered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightIndex = Math.max(highlightIndex - 1, 0);
    } else if (e.key === 'Enter' && highlightIndex >= 0) {
      e.preventDefault();
      select(filtered[highlightIndex]);
    } else if (e.key === 'Escape') {
      focused = false;
    }
  }

  // Sync external value changes
  $effect(() => {
    if (!focused) query = value;
  });
</script>

<div class="typeahead">
  {#if label}
    <span class="label">{label}</span>
  {/if}
  <div class="input-wrapper">
    <input
      bind:this={inputEl}
      type="text"
      {placeholder}
      value={query}
      oninput={handleInput}
      onfocus={handleFocus}
      onblur={handleBlur}
      onkeydown={handleKeydown}
      autocomplete="off"
    />
    {#if showDropdown}
      <div class="dropdown">
        {#if filtered.length > 0}
          {#each filtered as item, i}
            <button
              class="dropdown-item"
              class:highlighted={i === highlightIndex}
              onmousedown={() => select(item)}
            >
              {item}
            </button>
          {/each}
        {:else}
          <div class="dropdown-empty">No branches loaded. Select a repo first.</div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .typeahead {
    display: flex;
    align-items: center;
    gap: 6px;
    position: relative;
  }
  .label {
    font-size: 11px;
    color: var(--color-text-muted);
    flex-shrink: 0;
    white-space: nowrap;
  }
  .input-wrapper {
    position: relative;
    flex: 1;
    min-width: 120px;
  }
  input {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
  }
  input:focus {
    outline: none;
    border-color: var(--color-accent);
  }
  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    max-height: 200px;
    overflow-y: auto;
    z-index: 50;
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
  }
  .dropdown-item {
    display: block;
    width: 100%;
    padding: 6px 10px;
    border: none;
    background: none;
    color: var(--color-text-primary);
    font-size: 12px;
    font-family: var(--font-mono);
    text-align: left;
    cursor: pointer;
  }
  .dropdown-item:hover,
  .dropdown-item.highlighted {
    background: var(--color-bg-hover);
  }
  .dropdown-empty {
    padding: 8px 10px;
    font-size: 11px;
    color: var(--color-text-muted);
    font-style: italic;
  }
</style>
