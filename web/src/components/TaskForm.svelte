<script lang="ts">
  import type { CadenceUnit } from '../lib/api/CadenceUnit';
  import type { Group } from '../lib/api/Group';
  import type { TaskInput } from '../lib/api/TaskInput';
  import { m } from '../lib/paraglide/messages.js';

  let {
    initial = { name: '', cadence: { amount: 7, unit: 'days' }, priority: 0, group_id: null },
    groups,
    submitLabel,
    onsubmit,
    oncancel,
  }: {
    initial?: TaskInput;
    groups: Group[];
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
  // svelte-ignore state_referenced_locally
  let groupId = $state(initial.group_id);
  let busy = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const saved = await onsubmit({ name, cadence: { amount, unit }, priority, group_id: groupId });
      if (saved && !oncancel) {
        // A create form stays open for the next task, keeping the group since chores
        // tend to be added room by room.
        name = '';
      }
    } finally {
      busy = false;
    }
  }
</script>

<form onsubmit={submit}>
  <label class="name">
    {m.name()}
    <input bind:value={name} required maxlength="200" />
  </label>
  <label>
    {m.every()}
    <span class="cadence">
      <input type="number" bind:value={amount} min="1" max="999" required />
      <select bind:value={unit}>
        <option value="days">{m.unit_days({ count: amount })}</option>
        <option value="weeks">{m.unit_weeks({ count: amount })}</option>
      </select>
    </span>
  </label>
  <label>
    {m.priority()}
    <select bind:value={priority}>
      {#each [0, 1, 2, 3, 4, 5] as p (p)}
        <option value={p}>{p === 0 ? m.priority_none() : p === 5 ? m.priority_highest({ priority: p }) : p}</option>
      {/each}
    </select>
  </label>
  {#if groups.length > 0}
    <label>
      {m.group()}
      <select bind:value={groupId}>
        <option value={null}>{m.no_group()}</option>
        {#each groups as group (group.id)}
          <option value={group.id}>{group.name}</option>
        {/each}
      </select>
    </label>
  {/if}
  <div class="actions">
    <button type="submit" class="primary" disabled={busy}>{submitLabel}</button>
    {#if oncancel}
      <button type="button" onclick={oncancel}>{m.cancel()}</button>
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
