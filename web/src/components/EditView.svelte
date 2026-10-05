<script lang="ts">
  import type { Executor } from '../lib/api/Executor';
  import type { ProjectView } from '../lib/api/ProjectView';
  import type { TaskInput } from '../lib/api/TaskInput';
  import type { TaskView } from '../lib/api/TaskView';
  import { archiveTask, createExecutor, createTask, renameExecutor, updateTask } from '../lib/client';
  import { mutate } from '../lib/data.svelte';
  import { formatCadence } from '../lib/format';
  import TaskForm from './TaskForm.svelte';

  let { view, executors }: { view: ProjectView; executors: Executor[] } = $props();

  let editingTask = $state<number | null>(null);
  let newExecutor = $state('');
  // Rename drafts by executor id; absent means unchanged.
  let drafts = $state<Record<number, string>>({});

  // Alphabetical, so the list doesn't jump around while tasks are edited.
  const tasks = $derived(view.tasks.toSorted((a, b) => a.name.localeCompare(b.name, 'en-US')));

  async function addTask(input: TaskInput) {
    return (await mutate(() => createTask(view.project.id, input))) !== undefined;
  }

  async function saveTask(id: number, input: TaskInput) {
    const saved = (await mutate(() => updateTask(id, input))) !== undefined;
    if (saved) editingTask = null;
    return saved;
  }

  async function archive(task: TaskView) {
    if (!confirm(`Remove "${task.name}"? Its history is kept.`)) return;
    await mutate(() => archiveTask(task.id));
  }

  async function addExecutor(event: SubmitEvent) {
    event.preventDefault();
    if (await mutate(() => createExecutor({ name: newExecutor }))) newExecutor = '';
  }

  async function rename(event: SubmitEvent, executor: Executor) {
    event.preventDefault();
    const name = drafts[executor.id];
    if (name === undefined) return;
    if (await mutate(() => renameExecutor(executor.id, { name }))) delete drafts[executor.id];
  }
</script>

<section>
  <h2>Chores</h2>
  <ul>
    {#each tasks as task (task.id)}
      <li>
        {#if editingTask === task.id}
          <TaskForm
            initial={{ name: task.name, cadence: task.cadence, priority: task.priority }}
            submitLabel="Save"
            onsubmit={(input) => saveTask(task.id, input)}
            oncancel={() => (editingTask = null)}
          />
        {:else}
          <div class="row">
            <span class="grow">
              <strong>{task.name}</strong>
              <span class="muted">
                · {formatCadence(task.cadence)}{task.priority > 0 ? ` · priority ${task.priority}` : ''}
              </span>
            </span>
            <button onclick={() => (editingTask = task.id)}>Edit</button>
            <button class="danger" onclick={() => archive(task)}>Remove</button>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
  <h3>New chore</h3>
  <TaskForm submitLabel="Add" onsubmit={addTask} />
</section>

<section>
  <h2>People</h2>
  <ul>
    {#each executors as executor (executor.id)}
      <li>
        <form class="row" onsubmit={(e) => rename(e, executor)}>
          <input
            class="grow"
            aria-label="Name"
            value={drafts[executor.id] ?? executor.name}
            oninput={(e) => (drafts[executor.id] = e.currentTarget.value)}
            required
            maxlength="200"
          />
          {#if drafts[executor.id] !== undefined && drafts[executor.id] !== executor.name}
            <button type="submit" class="primary">Save</button>
            <button type="button" onclick={() => delete drafts[executor.id]}>Cancel</button>
          {/if}
        </form>
      </li>
    {/each}
  </ul>
  <form class="row" onsubmit={addExecutor}>
    <input class="grow" bind:value={newExecutor} placeholder="Name" required maxlength="200" />
    <button type="submit" class="primary">Add</button>
  </form>
</section>

<style>
  section {
    margin-bottom: 2rem;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin: 0 0 1rem;
    padding: 0;
    list-style: none;
  }

  li {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: var(--surface);
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
  }

  .grow {
    flex: 1 1 10rem;
    min-width: 0;
  }
</style>
