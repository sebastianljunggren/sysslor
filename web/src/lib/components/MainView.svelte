<script lang="ts">
  import { flip } from 'svelte/animate';
  import type { Executor } from '#lib/api/Executor.js';
  import type { ProjectView } from '#lib/api/ProjectView.js';
  import type { TaskView } from '#lib/api/TaskView.js';
  import { uuidv7 } from '#lib/client.js';
  import { data, pickGroupFilter, saveCompletion, undoCompletion, type GroupFilter } from '#lib/data.svelte.js';
  import { m } from '#lib/paraglide/messages.js';
  import { pendingFor } from '#lib/queue.svelte.js';
  import { pageHref } from '#lib/route.js';
  import CompleteDialog from './CompleteDialog.svelte';
  import TaskItem from './TaskItem.svelte';

  let { view, executors }: { view: ProjectView; executors: Executor[] } = $props();

  const groupsById = $derived(new Map(view.groups.map((g) => [g.id, g])));
  const hasUngrouped = $derived(view.tasks.some((t) => t.group_id === null));
  // Falls back to all tasks if the stored group is gone, or its option isn't offered.
  const filter = $derived.by((): GroupFilter => {
    const stored = data.groupFilter;
    if (view.groups.length === 0) return null;
    if (stored === 'none') return hasUngrouped ? 'none' : null;
    return stored !== null && groupsById.has(stored) ? stored : null;
  });
  // Filtering keeps the server's order.
  const tasks = $derived(
    filter === null
      ? view.tasks
      : view.tasks.filter((t) => t.group_id === (filter === 'none' ? null : filter)),
  );
  const filterOptions = $derived<{ value: GroupFilter; label: string }[]>([
    { value: null, label: m.all_groups() },
    ...view.groups.map((g) => ({ value: g.id, label: g.name })),
    ...(hasUngrouped ? [{ value: 'none' as const, label: m.no_group() }] : []),
  ]);

  let completing = $state<TaskView | null>(null);
  let undo = $state<{ completionId: string; taskName: string; offline: boolean } | null>(null);
  let undoTimer: ReturnType<typeof setTimeout> | undefined;

  async function complete(task: TaskView, executorId: number, completedAt: number) {
    const id = uuidv7();
    const result = await saveCompletion({
      id,
      taskName: task.name,
      input: { task_id: task.id, executor_id: executorId, completed_at: completedAt },
    });
    if (result !== 'failed') showUndo(id, task.name, result === 'queued');
  }

  function showUndo(completionId: string, taskName: string, offline: boolean) {
    clearTimeout(undoTimer);
    undo = { completionId, taskName, offline };
    undoTimer = setTimeout(() => (undo = null), 8000);
  }

  async function undoLatest() {
    if (!undo) return;
    const { completionId, taskName } = undo;
    clearTimeout(undoTimer);
    undo = null;
    await undoCompletion(completionId, taskName);
  }

  $effect(() => () => clearTimeout(undoTimer));
</script>

{#if executors.length === 0}
  <p class="muted">{m.no_people()} <a href={pageHref('people')}>{m.add_people()}</a></p>
{/if}

{#if view.groups.length > 0 && view.tasks.length > 0}
  <label class="filter">
    {m.groups()}
    <select bind:value={() => filter, (v) => pickGroupFilter(v)}>
      {#each filterOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </label>
{/if}

{#if view.tasks.length === 0}
  <p class="muted">{m.no_chores()} <a href={pageHref('chores')}>{m.add_chores()}</a></p>
{:else if tasks.length === 0}
  <p class="muted">{m.no_chores_in_group()}</p>
{:else}
  <ul>
    {#each tasks as task (task.id)}
      <li animate:flip={{ duration: 400 }}>
        <TaskItem
          {task}
          pending={pendingFor(task.id)}
          group={filter === null && task.group_id !== null ? groupsById.get(task.group_id) : undefined}
          {executors}
          canComplete={executors.length > 0}
          oncomplete={() => (completing = task)}
        />
      </li>
    {/each}
  </ul>
{/if}

<CompleteDialog
  task={completing}
  {executors}
  oncomplete={(executorId, completedAt) => complete(completing!, executorId, completedAt)}
  onclose={() => (completing = null)}
/>

{#if undo}
  <div class="toast" role="status">
    <span>{undo.offline ? m.saved_offline({ name: undo.taskName }) : m.task_done({ name: undo.taskName })}</span>
    <button class="link" onclick={undoLatest}>{m.undo()}</button>
  </div>
{/if}

<style>
  .filter {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0 0 1rem;
    color: var(--muted);
  }

  .filter select {
    color: var(--fg);
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .toast {
    position: fixed;
    bottom: calc(1rem + env(safe-area-inset-bottom));
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    background: var(--fg);
    color: var(--bg);
    box-shadow: 0 2px 8px color-mix(in srgb, var(--base03) 30%, transparent);
  }

  .toast .link {
    color: var(--bg);
    font-weight: bold;
  }
</style>
