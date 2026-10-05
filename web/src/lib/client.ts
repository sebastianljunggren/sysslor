import type { Completion } from './api/Completion';
import type { CompletionInput } from './api/CompletionInput';
import type { ErrorBody } from './api/ErrorBody';
import type { Event } from './api/Event';
import type { Executor } from './api/Executor';
import type { ExecutorInput } from './api/ExecutorInput';
import type { ProjectView } from './api/ProjectView';
import type { Task } from './api/Task';
import type { TaskInput } from './api/TaskInput';

// Only one project exists until multiple projects are supported.
export const DEFAULT_PROJECT = 1;

export class ApiError extends Error {}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`/api${path}`, {
      method,
      headers: body === undefined ? {} : { 'Content-Type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    throw new ApiError('Could not reach the server.');
  }
  if (!response.ok) {
    const error = await response
      .json()
      .then((body: ErrorBody) => body.error)
      .catch(() => response.statusText);
    throw new ApiError(`${error} (${response.status})`);
  }
  return response.status === 204 ? (undefined as T) : response.json();
}

export const getProject = (id: number) => request<ProjectView>('GET', `/projects/${id}`);

export const createTask = (projectId: number, input: TaskInput) =>
  request<Task>('POST', `/projects/${projectId}/tasks`, input);
export const updateTask = (id: number, input: TaskInput) =>
  request<Task>('PUT', `/tasks/${id}`, input);
export const archiveTask = (id: number) => request<void>('DELETE', `/tasks/${id}`);

export const listExecutors = () => request<Executor[]>('GET', '/executors');
export const createExecutor = (input: ExecutorInput) =>
  request<Executor>('POST', '/executors', input);
export const renameExecutor = (id: number, input: ExecutorInput) =>
  request<Executor>('PUT', `/executors/${id}`, input);

export const putCompletion = (id: string, input: CompletionInput) =>
  request<Completion>('PUT', `/completions/${id}`, input);
export const deleteCompletion = (id: string) => request<void>('DELETE', `/completions/${id}`);

/**
 * Calls `onChange` with the affected project (or `null` for "refetch everything")
 * whenever data changes on the server. Also fires on every (re)connect, since events
 * sent while disconnected are lost. Returns a function that closes the stream.
 */
export function subscribe(
  onChange: (project: number | null) => void,
  onConnection: (connected: boolean) => void,
): () => void {
  // EventSource reconnects by itself after errors.
  const source = new EventSource('/api/events');
  source.onopen = () => {
    onConnection(true);
    onChange(null);
  };
  source.onerror = () => onConnection(false);
  source.onmessage = (message) => {
    const event: Event = JSON.parse(message.data);
    onChange(event.project);
  };
  return () => source.close();
}

/** RFC 9562 UUIDv7: 48-bit millisecond timestamp, then random bits. */
export function uuidv7(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  const ms = Date.now();
  for (let i = 0; i < 6; i++) {
    bytes[i] = Math.floor(ms / 2 ** (8 * (5 - i))) & 0xff;
  }
  bytes[6] = (bytes[6] & 0x0f) | 0x70;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}
