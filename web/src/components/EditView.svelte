<script lang="ts">
  import type { Executor } from '../lib/api/Executor';
  import type { ProjectView } from '../lib/api/ProjectView';
  import type { TaskInput } from '../lib/api/TaskInput';
  import type { TaskView } from '../lib/api/TaskView';
  import {
    archiveTask,
    createExecutor,
    createTask,
    renameExecutor,
    renameProject,
    updateTask,
  } from '../lib/client';
  import { logOut, mutate } from '../lib/data.svelte';
  import { formatCadence } from '../lib/format';
  import { m } from '../lib/paraglide/messages.js';
  import { getLocale, locales, setLocale, type Locale } from '../lib/paraglide/runtime.js';
  import TaskForm from './TaskForm.svelte';

  let { view, executors }: { view: ProjectView; executors: Executor[] } = $props();

  // Each language in its own name, so it can be found whatever the current locale is.
  const languageNames: Record<Locale, string> = { en: 'English', sv: 'Svenska' };

  // Absent means unchanged, so a rename from another device shows up.
  let projectDraft = $state<string | undefined>();
  let editingTask = $state<number | null>(null);
  let newExecutor = $state('');
  // Rename drafts by executor id; absent means unchanged.
  let drafts = $state<Record<number, string>>({});

  // Alphabetical, so the list doesn't jump around while tasks are edited.
  const tasks = $derived(view.tasks.toSorted((a, b) => a.name.localeCompare(b.name, getLocale())));

  async function saveProjectName(event: SubmitEvent) {
    event.preventDefault();
    const name = projectDraft;
    if (name === undefined) return;
    if (await mutate(() => renameProject(view.project.id, { name }))) projectDraft = undefined;
  }

  async function addTask(input: TaskInput) {
    return (await mutate(() => createTask(view.project.id, input))) !== undefined;
  }

  async function saveTask(id: number, input: TaskInput) {
    const saved = (await mutate(() => updateTask(id, input))) !== undefined;
    if (saved) editingTask = null;
    return saved;
  }

  async function archive(task: TaskView) {
    if (!confirm(m.confirm_remove_task({ name: task.name }))) return;
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
  <h2>{m.project_name()}</h2>
  <form class="row" onsubmit={saveProjectName}>
    <input
      class="grow"
      aria-label={m.project_name()}
      value={projectDraft ?? view.project.name}
      oninput={(e) => (projectDraft = e.currentTarget.value)}
      required
      maxlength="200"
    />
    {#if projectDraft !== undefined && projectDraft !== view.project.name}
      <button type="submit" class="primary">{m.save()}</button>
      <button type="button" onclick={() => (projectDraft = undefined)}>{m.cancel()}</button>
    {/if}
  </form>
</section>

<section>
  <h2>{m.chores()}</h2>
  <ul>
    {#each tasks as task (task.id)}
      <li>
        {#if editingTask === task.id}
          <TaskForm
            initial={{ name: task.name, cadence: task.cadence, priority: task.priority }}
            submitLabel={m.save()}
            onsubmit={(input) => saveTask(task.id, input)}
            oncancel={() => (editingTask = null)}
          />
        {:else}
          <div class="row">
            <span class="grow">
              <strong>{task.name}</strong>
              <span class="muted">
                · {formatCadence(task.cadence)}{task.priority > 0 ? ` · ${m.priority_value({ priority: task.priority })}` : ''}
              </span>
            </span>
            <button onclick={() => (editingTask = task.id)}>{m.edit()}</button>
            <button class="danger" onclick={() => archive(task)}>{m.remove()}</button>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
  <h3>{m.new_chore()}</h3>
  <TaskForm submitLabel={m.add()} onsubmit={addTask} />
</section>

<section>
  <h2>{m.people()}</h2>
  <ul>
    {#each executors as executor (executor.id)}
      <li>
        <form class="row" onsubmit={(e) => rename(e, executor)}>
          <input
            class="grow"
            aria-label={m.name()}
            value={drafts[executor.id] ?? executor.name}
            oninput={(e) => (drafts[executor.id] = e.currentTarget.value)}
            required
            maxlength="200"
          />
          {#if drafts[executor.id] !== undefined && drafts[executor.id] !== executor.name}
            <button type="submit" class="primary">{m.save()}</button>
            <button type="button" onclick={() => delete drafts[executor.id]}>{m.cancel()}</button>
          {/if}
        </form>
      </li>
    {/each}
  </ul>
  <form class="row" onsubmit={addExecutor}>
    <input class="grow" bind:value={newExecutor} placeholder={m.name()} required maxlength="200" />
    <button type="submit" class="primary">{m.add()}</button>
  </form>
</section>

<section>
  <h2>{m.this_device()}</h2>
  <div class="row">
    <label class="row">
      {m.language()}
      <!-- Reloads the page, so nothing needs to react to the change. -->
      <select value={getLocale()} onchange={(e) => setLocale(e.currentTarget.value as Locale)}>
        {#each locales as locale (locale)}
          <option value={locale}>{languageNames[locale]}</option>
        {/each}
      </select>
    </label>
    <button onclick={logOut}>{m.log_out()}</button>
  </div>
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
