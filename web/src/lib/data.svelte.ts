import type { Executor } from './api/Executor';
import type { ProjectView } from './api/ProjectView';
import {
  ApiError,
  DEFAULT_PROJECT,
  deleteCompletion,
  getProject,
  isUnauthorized,
  listExecutors,
  login,
  logout,
  putCompletion,
  subscribe,
} from './client';
import { m } from './paraglide/messages.js';
import { clearQueue, dequeue, enqueue, isQueued, queue, type QueuedCompletion } from './queue.svelte';

const GROUP_FILTER_KEY = 'sysslor.group';

/** A group id, `'none'` for ungrouped tasks, or `null` for all tasks. */
export type GroupFilter = number | 'none' | null;

export const data = $state({
  view: null as ProjectView | null,
  executors: [] as Executor[],
  /** The group filter last picked on this device. The group may no longer exist. */
  groupFilter: loadGroupFilter(),
  connected: true,
  /** Assumed until the server says otherwise, so logged-in devices skip a round trip. */
  loggedIn: true,
  loadError: null as string | null,
  actionError: null as string | null,
});

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Wraps a fetch so a slow, older response can't overwrite a newer one. */
function latest<T>(load: () => Promise<T>, apply: (value: T) => void): () => Promise<void> {
  let seq = 0;
  return async () => {
    const current = ++seq;
    try {
      const value = await load();
      if (current !== seq) return;
      apply(value);
      data.loadError = null;
    } catch (error) {
      if (isUnauthorized(error)) loggedOut();
      else if (current === seq) data.loadError = message(error);
    }
  };
}

export const refreshProject = latest(
  () => getProject(DEFAULT_PROJECT),
  (view) => (data.view = view),
);

export const refreshExecutors = latest(listExecutors, (executors) => (data.executors = executors));

export function refreshAll() {
  return Promise.all([refreshProject(), refreshExecutors()]);
}

/**
 * Keeps `data` in sync with the server and sends queued completions whenever the
 * connection may be back. Returns a function that disconnects.
 */
export function connect(): () => void {
  const disconnect = subscribe(
    (project) => {
      if (project === null) refreshAll();
      else if (project === DEFAULT_PROJECT) refreshProject();
    },
    (connected) => {
      data.connected = connected;
      if (connected) flushQueue();
    },
  );
  const onOnline = () => flushQueue();
  window.addEventListener('online', onOnline);
  flushQueue();
  return () => {
    window.removeEventListener('online', onOnline);
    disconnect();
    // A stream closed by logging out isn't a lost connection. A stale `false` would show
    // the banner after the next login until the new stream opens.
    data.connected = true;
  };
}

/**
 * Runs a write and refetches afterwards. The SSE event would trigger a refetch too,
 * but this keeps the UI responsive if the event stream is down. Errors are shown in
 * the UI; returns `undefined` on failure.
 */
export async function mutate<T>(write: () => Promise<T>): Promise<T | undefined> {
  try {
    const result = await write();
    data.actionError = null;
    refreshAll();
    return result;
  } catch (error) {
    if (isUnauthorized(error)) loggedOut();
    else data.actionError = message(error);
    return undefined;
  }
}

let flushing: Promise<Set<string>> | null = null;
let flushAgain = false;
let sending: string | null = null;

/**
 * Sends queued completions in order, until one can't be sent for now (offline, server
 * down, logged out). Resolves with the ids that reached the server. Calls while sending
 * share the run, and make it try once more at the end: the connection may have come back
 * after the attempt that failed.
 */
export function flushQueue(): Promise<Set<string>> {
  if (flushing) {
    flushAgain = true;
    return flushing;
  }
  flushing = (async () => {
    const sent = new Set<string>();
    try {
      do {
        flushAgain = false;
        await sendQueued(sent);
      } while (flushAgain && data.loggedIn);
    } finally {
      flushing = null;
      queue.syncing = false;
      if (sent.size > 0) refreshAll();
    }
    return sent;
  })();
  return flushing;
}

async function sendQueued(sent: Set<string>) {
  for (let entry = queue.entries[0]; entry; entry = queue.entries[0]) {
    queue.syncing = true;
    sending = entry.id;
    try {
      await putCompletion(entry.id, entry.input);
    } catch (error) {
      if (!(error instanceof ApiError)) throw error;
      // Unreachable, or e.g. a 502 during a deploy: try again later.
      if (error.status === null || error.status >= 500) return;
      if (error.status === 401) {
        loggedOut();
        return;
      }
      // E.g. the task was archived meanwhile. Retrying won't help.
      dequeue(entry.id);
      data.actionError = m.sync_failed({ name: entry.taskName, error: error.message });
      continue;
    } finally {
      sending = null;
    }
    sent.add(entry.id);
    // If it was backdated while being sent, the loop sends the new version next.
    if (queue.entries[0] === entry) dequeue(entry.id);
  }
}

/** What became of a saved completion. */
export type SaveResult = 'saved' | 'queued' | 'failed';

/**
 * Creates or replaces a completion. Without a connection it is queued and sent later,
 * so this only fails if the server rejects it (the error is then shown in the UI).
 */
export async function saveCompletion(entry: QueuedCompletion): Promise<SaveResult> {
  enqueue(entry);
  const sent = await flushQueue();
  if (sent.has(entry.id) && !isQueued(entry.id)) {
    data.actionError = null;
    return 'saved';
  }
  return isQueued(entry.id) ? 'queued' : 'failed';
}

/** Undoes a completion, whether or not it has reached the server yet. */
export async function undoCompletion(id: string) {
  // Removing it from the queue mid-request wouldn't stop it from arriving.
  if (sending === id) await flushing;
  if (dequeue(id)) return;
  await mutate(() => deleteCompletion(id));
}

/** Returns an error message to show, or `null` once logged in. */
export async function logIn(password: string): Promise<string | null> {
  try {
    await login({ password });
  } catch (error) {
    return isUnauthorized(error) ? m.wrong_password() : message(error);
  }
  data.loadError = null;
  data.actionError = null;
  data.loggedIn = true;
  return null;
}

export async function logOut() {
  try {
    await logout();
  } catch (error) {
    data.actionError = message(error);
    return;
  }
  // Like the cached data, they shouldn't outlive the session on a shared device.
  clearQueue();
  loggedOut();
}

function loggedOut() {
  data.view = null;
  data.executors = [];
  data.loggedIn = false;
}

export function pickGroupFilter(filter: GroupFilter) {
  data.groupFilter = filter;
  try {
    if (filter === null) localStorage.removeItem(GROUP_FILTER_KEY);
    else localStorage.setItem(GROUP_FILTER_KEY, String(filter));
  } catch {
    // Storage can be unavailable (e.g. private browsing); the pick then lasts this session.
  }
}

function loadGroupFilter(): GroupFilter {
  try {
    const stored = localStorage.getItem(GROUP_FILTER_KEY);
    if (stored === null || stored === 'none') return stored;
    const id = Number(stored);
    return Number.isInteger(id) ? id : null;
  } catch {
    return null;
  }
}
