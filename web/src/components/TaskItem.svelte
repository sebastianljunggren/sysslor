<script lang="ts">
  import type { Executor } from '../lib/api/Executor';
  import type { Group } from '../lib/api/Group';
  import type { TaskView } from '../lib/api/TaskView';
  import { deleteCompletion, putCompletion } from '../lib/client';
  import { mutate } from '../lib/data.svelte';
  import { formatCadence, formatDue, formatInstant, localDateTime } from '../lib/format';
  import { m } from '../lib/paraglide/messages.js';

  let {
    task,
    group,
    executors,
    canComplete,
    oncomplete,
  }: {
    task: TaskView;
    /** Shown as a tag; absent for ungrouped tasks or when filtering on one group. */
    group?: Group;
    executors: Executor[];
    canComplete: boolean;
    /** Asks who did it and when. */
    oncomplete: () => void;
  } = $props();

  let busy = $state(false);
  let expanded = $state(false);
  let backdateTo = $state('');

  const last = $derived(task.last_completion);
  const lastBy = $derived(executors.find((e) => e.id === last?.executor_id)?.name ?? '?');

  async function run(action: () => Promise<unknown>) {
    busy = true;
    try {
      await action();
    } finally {
      busy = false;
    }
  }

  function toggle() {
    expanded = !expanded;
    if (expanded && last) backdateTo = localDateTime(last.completed_at);
  }

  async function backdate(event: SubmitEvent) {
    event.preventDefault();
    if (!last || !backdateTo) return;
    const completedAt = new Date(backdateTo).getTime();
    // Replacing the completion under the same id is what moves it in time.
    const saved = await mutate(() =>
      putCompletion(last.id, {
        task_id: last.task_id,
        executor_id: last.executor_id,
        completed_at: Math.min(completedAt, Date.now()),
      }),
    );
    if (saved) expanded = false;
  }

  async function undo() {
    if (!last) return;
    await mutate(() => deleteCompletion(last.id));
    expanded = false;
  }
</script>

<article class:overdue={task.overdue}>
  <div class="row">
    <button class="summary" aria-expanded={expanded} disabled={!last} onclick={toggle}>
      <span class="name">
        {task.name}
        {#if task.priority > 0}
          <span class="priority" title={m.priority_value({ priority: task.priority })}>{'!'.repeat(task.priority)}</span>
        {/if}
        {#if group}
          <span
            class="group"
            class:colored={group.color !== null}
            style:--group-color={group.color ? `var(--${group.color})` : undefined}>{group.name}</span
          >
        {/if}
      </span>
      <span class="meta">
        <span class="due">{formatDue(task.due)}</span>
        · {formatCadence(task.cadence)}
        {#if last}
          · {lastBy} {formatInstant(last.completed_at)}
        {/if}
      </span>
    </button>
    <button
      class="complete"
      aria-label={m.mark_done({ name: task.name })}
      disabled={!canComplete}
      onclick={oncomplete}>✓</button
    >
  </div>

  {#if expanded && last}
    <form class="details" onsubmit={(e) => run(() => backdate(e))}>
      <label>
        {m.done_at()}
        <input type="datetime-local" bind:value={backdateTo} max={localDateTime(Date.now())} required />
      </label>
      <button type="submit" disabled={busy}>{m.change_time()}</button>
      <button type="button" class="danger" disabled={busy} onclick={() => run(undo)}>{m.undo()}</button>
    </form>
  {/if}
</article>

<style>
  article {
    border: 1px solid var(--border);
    border-left: 4px solid var(--border);
    border-radius: 0.5rem;
    background: var(--surface);
  }

  article.overdue {
    border-left-color: var(--danger);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
  }

  .summary {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
    padding: 0.25rem;
    border: none;
    background: none;
    text-align: left;
  }

  .summary:disabled {
    opacity: 1;
  }

  .name {
    font-size: 1.1rem;
    font-weight: 600;
  }

  .priority {
    color: var(--danger);
  }

  .group {
    margin-left: 0.25rem;
    padding: 0.05rem 0.5rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--muted);
    font-size: 0.8rem;
    font-weight: normal;
    vertical-align: middle;
  }

  /* A faint fill under a stronger border, so both read as the same color but stay distinct. */
  .group.colored {
    border-color: color-mix(in srgb, var(--group-color) 70%, transparent);
    background: color-mix(in srgb, var(--group-color) 18%, transparent);
    color: var(--fg);
  }

  .meta {
    color: var(--muted);
    font-size: 0.9rem;
  }

  .overdue .due {
    color: var(--danger);
    font-weight: 600;
  }

  .complete {
    flex: none;
    width: 3rem;
    height: 3rem;
    border-radius: 50%;
    font-size: 1.4rem;
  }

  .complete:not(:disabled):hover {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }

  .details {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 0.5rem;
    padding: 0 0.75rem 0.75rem;
  }

  .details label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    color: var(--muted);
    font-size: 0.9rem;
  }
</style>
