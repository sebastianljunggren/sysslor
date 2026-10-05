<script lang="ts">
  import { flip } from 'svelte/animate';
  import type { Executor } from '../lib/api/Executor';
  import type { ProjectView } from '../lib/api/ProjectView';
  import type { TaskView } from '../lib/api/TaskView';
  import { deleteCompletion, putCompletion, uuidv7 } from '../lib/client';
  import { data, mutate, pickExecutor, pickGroupFilter, type GroupFilter } from '../lib/data.svelte';
  import { m } from '../lib/paraglide/messages.js';
  import TaskItem from './TaskItem.svelte';

  let { view, executors }: { view: ProjectView; executors: Executor[] } = $props();

  // Falls back to the first executor if the stored one was never set or is gone.
  const executor = $derived(
    executors.find((e) => e.id === data.executorId) ?? executors[0] ?? null,
  );

  const groupNames = $derived(new Map(view.groups.map((g) => [g.id, g.name])));
  const hasUngrouped = $derived(view.tasks.some((t) => t.group_id === null));
  // Falls back to all tasks if the stored group is gone, or its chip isn't shown.
  const filter = $derived.by((): GroupFilter => {
    const stored = data.groupFilter;
    if (view.groups.length === 0) return null;
    if (stored === 'none') return hasUngrouped ? 'none' : null;
    return stored !== null && groupNames.has(stored) ? stored : null;
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

  let undo = $state<{ completionId: string; taskName: string } | null>(null);
  let undoTimer: ReturnType<typeof setTimeout> | undefined;

  // Ids of completions whose PUT failed, reused on the next tap so a request that
  // reached the server despite the error isn't registered twice.
  const pendingIds = new Map<number, string>();

  async function complete(task: TaskView) {
    if (!executor) return;
    const id = pendingIds.get(task.id) ?? uuidv7();
    pendingIds.set(task.id, id);
    const completion = await mutate(() =>
      putCompletion(id, { task_id: task.id, executor_id: executor.id, completed_at: Date.now() }),
    );
    if (!completion) return;
    pendingIds.delete(task.id);
    showUndo(id, task.name);
  }

  function showUndo(completionId: string, taskName: string) {
    clearTimeout(undoTimer);
    undo = { completionId, taskName };
    undoTimer = setTimeout(() => (undo = null), 8000);
  }

  async function undoLatest() {
    if (!undo) return;
    const { completionId } = undo;
    clearTimeout(undoTimer);
    undo = null;
    await mutate(() => deleteCompletion(completionId));
  }

  $effect(() => () => clearTimeout(undoTimer));
</script>

{#if executors.length === 0}
  <p class="muted">{m.no_people()} <a href="#edit">{m.add_people()}</a></p>
{:else}
  <fieldset class="chips">
    <legend>{m.who_is_doing_it()}</legend>
    {#each executors as candidate (candidate.id)}
      <label class:selected={candidate.id === executor?.id}>
        <input
          type="radio"
          name="executor"
          checked={candidate.id === executor?.id}
          onchange={() => pickExecutor(candidate.id)}
        />
        {candidate.name}
      </label>
    {/each}
  </fieldset>
{/if}

{#if view.groups.length > 0 && view.tasks.length > 0}
  <fieldset class="chips">
    <legend>{m.groups()}</legend>
    {#each filterOptions as option (option.value)}
      <label class:selected={option.value === filter}>
        <input
          type="radio"
          name="group"
          checked={option.value === filter}
          onchange={() => pickGroupFilter(option.value)}
        />
        {option.label}
      </label>
    {/each}
  </fieldset>
{/if}

{#if view.tasks.length === 0}
  <p class="muted">{m.no_chores()} <a href="#edit">{m.add_chores()}</a></p>
{:else if tasks.length === 0}
  <p class="muted">{m.no_chores_in_group()}</p>
{:else}
  <ul>
    {#each tasks as task (task.id)}
      <li animate:flip={{ duration: 400 }}>
        <TaskItem
          {task}
          groupName={filter === null && task.group_id !== null ? groupNames.get(task.group_id) : undefined}
          {executors}
          canComplete={executor !== null}
          oncomplete={() => complete(task)}
        />
      </li>
    {/each}
  </ul>
{/if}

{#if undo}
  <div class="toast" role="status">
    <span>{m.task_done({ name: undo.taskName })}</span>
    <button class="link" onclick={undoLatest}>{m.undo()}</button>
  </div>
{/if}

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin: 0 0 1rem;
    padding: 0;
    border: none;
  }

  .chips legend {
    margin-bottom: 0.5rem;
    color: var(--muted);
  }

  .chips label {
    padding: 0.4rem 0.9rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    cursor: pointer;
  }

  .chips label.selected {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }

  .chips label:has(input:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .chips input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
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
    bottom: 1rem;
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
