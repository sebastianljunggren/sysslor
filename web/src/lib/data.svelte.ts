import type { Executor } from './api/Executor';
import type { ProjectView } from './api/ProjectView';
import {
  DEFAULT_PROJECT,
  getProject,
  isUnauthorized,
  listExecutors,
  login,
  logout,
  subscribe,
} from './client';

const EXECUTOR_KEY = 'sysslor.executor';

export const data = $state({
  view: null as ProjectView | null,
  executors: [] as Executor[],
  /** The executor last picked on this device. May no longer exist. */
  executorId: loadExecutorId(),
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

/** Keeps `data` in sync with the server. Returns a function that disconnects. */
export function connect(): () => void {
  return subscribe(
    (project) => {
      if (project === null) refreshAll();
      else if (project === DEFAULT_PROJECT) refreshProject();
    },
    (connected) => (data.connected = connected),
  );
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

/** Returns an error message to show, or `null` once logged in. */
export async function logIn(password: string): Promise<string | null> {
  try {
    await login({ password });
  } catch (error) {
    return isUnauthorized(error) ? 'Wrong password.' : message(error);
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
  loggedOut();
}

function loggedOut() {
  data.view = null;
  data.executors = [];
  data.loggedIn = false;
}

export function pickExecutor(id: number) {
  data.executorId = id;
  try {
    localStorage.setItem(EXECUTOR_KEY, String(id));
  } catch {
    // Storage can be unavailable (e.g. private browsing); the pick then lasts this session.
  }
}

function loadExecutorId(): number | null {
  try {
    const stored = localStorage.getItem(EXECUTOR_KEY);
    return stored === null ? null : Number(stored);
  } catch {
    return null;
  }
}
