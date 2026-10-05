<script lang="ts">
  import { flip } from 'svelte/animate';
  import type { Executor } from '../lib/api/Executor';
  import type { ProjectView } from '../lib/api/ProjectView';
  import type { TaskView } from '../lib/api/TaskView';
  import { deleteCompletion, putCompletion, uuidv7 } from '../lib/client';
  import { data, mutate, pickExecutor } from '../lib/data.svelte';
  import TaskItem from './TaskItem.svelte';

  let { view, executors }: { view: ProjectView; executors: Executor[] } = $props();

  // Falls back to the first executor if the stored one was never set or is gone.
  const executor = $derived(
    executors.find((e) => e.id === data.executorId) ?? executors[0] ?? null,
  );

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
  <p class="muted">Add the people who do the chores under <a href="#edit">Edit</a>.</p>
{:else}
  <fieldset class="executors">
    <legend>Who is doing it?</legend>
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

{#if view.tasks.length === 0}
  <p class="muted">No chores yet. Add some under <a href="#edit">Edit</a>.</p>
{:else}
  <ul>
    {#each view.tasks as task (task.id)}
      <li animate:flip={{ duration: 400 }}>
        <TaskItem {task} {executors} canComplete={executor !== null} oncomplete={() => complete(task)} />
      </li>
    {/each}
  </ul>
{/if}

{#if undo}
  <div class="toast" role="status">
    <span>{undo.taskName} done</span>
    <button class="link" onclick={undoLatest}>Undo</button>
  </div>
{/if}

<style>
  .executors {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin: 0 0 1rem;
    padding: 0;
    border: none;
  }

  .executors legend {
    margin-bottom: 0.5rem;
    color: var(--muted);
  }

  .executors label {
    padding: 0.4rem 0.9rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    cursor: pointer;
  }

  .executors label.selected {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }

  .executors label:has(input:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .executors input {
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
    box-shadow: 0 2px 8px rgb(0 0 0 / 0.3);
  }

  .toast .link {
    color: var(--bg);
    font-weight: bold;
  }
</style>
