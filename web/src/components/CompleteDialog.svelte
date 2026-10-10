<script lang="ts">
  import type { Executor } from '../lib/api/Executor';
  import type { TaskView } from '../lib/api/TaskView';
  import { localDateTime } from '../lib/format';
  import { m } from '../lib/paraglide/messages.js';

  let {
    task,
    executors,
    oncomplete,
    onclose,
  }: {
    /** The task to complete; the dialog is open while set. */
    task: TaskView | null;
    executors: Executor[];
    oncomplete: (executorId: number, completedAt: number) => Promise<void>;
    onclose: () => void;
  } = $props();

  let dialog: HTMLDialogElement;
  let busy = $state(false);
  let doneAt = $state('');

  $effect(() => {
    if (task && !dialog.open) {
      doneAt = localDateTime(Date.now());
      dialog.showModal();
    } else if (!task && dialog.open) {
      dialog.close();
    }
  });

  async function pick(executorId: number) {
    if (!doneAt) return;
    busy = true;
    try {
      // Browsers that don't enforce `max` on datetime-local inputs (e.g. iOS Safari) allow the future.
      await oncomplete(executorId, Math.min(new Date(doneAt).getTime(), Date.now()));
    } finally {
      busy = false;
    }
    dialog.close();
  }

  // Clicks on the backdrop target the dialog itself; clicks on its content target children.
  function closeOnBackdrop(event: MouseEvent) {
    if (event.target === dialog && !busy) dialog.close();
  }
</script>

<dialog bind:this={dialog} aria-labelledby="complete-title" onclose={onclose} onclick={closeOnBackdrop}>
  {#if task}
    <h2 id="complete-title">{m.complete_task({ name: task.name })}</h2>
    <label class="time">
      {m.done_at()}
      <input type="datetime-local" bind:value={doneAt} max={localDateTime(Date.now())} required />
    </label>
    <div class="names" role="group" aria-label={m.who_is_doing_it()}>
      {#each executors as executor (executor.id)}
        <button disabled={busy || !doneAt} onclick={() => pick(executor.id)}>{executor.name}</button>
      {/each}
    </div>
    <div class="actions">
      <button disabled={busy} onclick={() => dialog.close()}>{m.cancel()}</button>
    </div>
  {/if}
</dialog>

<style>
  dialog {
    width: min(30rem, calc(100vw - 2rem));
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: var(--surface);
    color: var(--fg);
  }

  dialog::backdrop {
    background: color-mix(in srgb, var(--base03) 50%, transparent);
  }

  h2 {
    margin: 0 0 1rem;
    font-size: 1.2rem;
  }

  .time {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    margin-bottom: 1rem;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .names {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 50vh;
    overflow-y: auto;
  }

  .names button {
    flex: none;
    padding: 0.75rem 1rem;
    font-size: 1.05rem;
    text-align: left;
  }

  .names button:not(:disabled):hover {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }

  .actions {
    display: flex;
    justify-content: end;
    margin-top: 1rem;
  }
</style>
