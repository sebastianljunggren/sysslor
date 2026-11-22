import type { CompletionInput } from './api/CompletionInput';

const QUEUE_KEY = 'sysslor.queue';

/** A completion that hasn't reached the server yet. */
export interface QueuedCompletion {
  id: string;
  input: CompletionInput;
  /** For messages about it, also after the task is gone from the list. */
  taskName: string;
}

/**
 * Completions waiting to be sent, oldest first. Kept in localStorage so they survive a
 * reload or a closed app. Sending is in `data.svelte.ts`.
 */
export const queue = $state({
  entries: load(),
  /** Whether entries are being sent right now. */
  syncing: false,
});

/** Adds a completion, or replaces the queued one with the same id (e.g. to backdate it). */
export function enqueue(entry: QueuedCompletion) {
  const index = queue.entries.findIndex((e) => e.id === entry.id);
  if (index === -1) queue.entries.push(entry);
  else queue.entries[index] = entry;
  save();
}

/** Removes a completion. Returns whether it was queued. */
export function dequeue(id: string): boolean {
  const before = queue.entries.length;
  queue.entries = queue.entries.filter((e) => e.id !== id);
  save();
  return queue.entries.length < before;
}

export function isQueued(id: string): boolean {
  return queue.entries.some((e) => e.id === id);
}

/** The most recent queued completion of a task, if any. */
export function pendingFor(taskId: number): QueuedCompletion | undefined {
  let latest: QueuedCompletion | undefined;
  for (const entry of queue.entries) {
    if (entry.input.task_id !== taskId) continue;
    if (!latest || entry.input.completed_at >= latest.input.completed_at) latest = entry;
  }
  return latest;
}

export function clearQueue() {
  queue.entries = [];
  save();
}

function save() {
  try {
    if (queue.entries.length === 0) localStorage.removeItem(QUEUE_KEY);
    else localStorage.setItem(QUEUE_KEY, JSON.stringify(queue.entries));
  } catch {
    // Storage can be unavailable (e.g. private browsing); the queue then lasts this session.
  }
}

function load(): QueuedCompletion[] {
  try {
    const stored = JSON.parse(localStorage.getItem(QUEUE_KEY) ?? '[]');
    return Array.isArray(stored) ? stored : [];
  } catch {
    return [];
  }
}

// Another tab changed the queue. Without this, this tab's next write would drop its entries.
if (typeof window !== 'undefined') {
  window.addEventListener('storage', (event) => {
    if (event.key === QUEUE_KEY || event.key === null) queue.entries = load();
  });
}
