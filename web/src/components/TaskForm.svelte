<script lang="ts">
  import type { CadenceUnit } from '../lib/api/CadenceUnit';
  import type { TaskInput } from '../lib/api/TaskInput';

  let {
    initial = { name: '', cadence: { amount: 7, unit: 'days' }, priority: 0 },
    submitLabel,
    onsubmit,
    oncancel,
  }: {
    initial?: TaskInput;
    submitLabel: string;
    /** Resolves to whether the save succeeded. */
    onsubmit: (input: TaskInput) => Promise<boolean>;
    oncancel?: () => void;
  } = $props();

  // The form edits a copy; `initial` only seeds it.
  // svelte-ignore state_referenced_locally
  let name = $state(initial.name);
  // svelte-ignore state_referenced_locally
  let amount = $state(initial.cadence.amount);
  // svelte-ignore state_referenced_locally
  let unit = $state<CadenceUnit>(initial.cadence.unit);
  // svelte-ignore state_referenced_locally
  let priority = $state(initial.priority);
  let busy = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const saved = await onsubmit({ name, cadence: { amount, unit }, priority });
      if (saved && !oncancel) {
        // A create form stays open for the next task.
        name = '';
      }
    } finally {
      busy = false;
    }
  }
</script>

<form onsubmit={submit}>
  <label class="name">
    Name
    <input bind:value={name} required maxlength="200" />
  </label>
  <label>
    Every
    <span class="cadence">
      <input type="number" bind:value={amount} min="1" max="999" required />
      <select bind:value={unit}>
        <option value="days">{amount === 1 ? 'day' : 'days'}</option>
        <option value="weeks">{amount === 1 ? 'week' : 'weeks'}</option>
      </select>
    </span>
  </label>
  <label>
    Priority
    <select bind:value={priority}>
      {#each [0, 1, 2, 3, 4, 5] as p (p)}
        <option value={p}>{p === 0 ? 'None' : p === 5 ? '5 (highest)' : p}</option>
      {/each}
    </select>
  </label>
  <div class="actions">
    <button type="submit" class="primary" disabled={busy}>{submitLabel}</button>
    {#if oncancel}
      <button type="button" onclick={oncancel}>Cancel</button>
    {/if}
  </div>
</form>

<style>
  form {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 0.75rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    color: var(--muted);
    font-size: 0.9rem;
  }

  label input,
  label select {
    color: var(--fg);
  }

  .name {
    flex: 1 1 12rem;
  }

  .cadence {
    display: flex;
    gap: 0.25rem;
  }

  .cadence input {
    width: 4rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
