import { vi } from 'vitest';
import type { Executor } from '../lib/api/Executor';
import type { Group } from '../lib/api/Group';
import type { ProjectView } from '../lib/api/ProjectView';
import type { TaskView } from '../lib/api/TaskView';

export const executors: Executor[] = [
  { id: 1, name: 'Anna' },
  { id: 2, name: 'Bo' },
];

export const groups: Group[] = [
  { id: 10, name: 'Kitchen', color: 'yellow' },
  { id: 11, name: 'Bathroom', color: null },
];

export function task(overrides: Partial<TaskView> = {}): TaskView {
  return {
    id: 1,
    name: 'Dishes',
    cadence: { amount: 1, unit: 'days' },
    priority: 0,
    group_id: null,
    last_completion: null,
    due: null,
    urgency: null,
    overdue: true,
    ...overrides,
  };
}

export function projectView(overrides: Partial<ProjectView> = {}): ProjectView {
  return {
    project: { id: 1, name: 'Home' },
    groups,
    tasks: [
      task({ id: 1, name: 'Dishes', group_id: 10 }),
      task({ id: 2, name: 'Towels', group_id: 11 }),
      task({ id: 3, name: 'Plants', group_id: null }),
      task({ id: 4, name: 'Fridge', group_id: 10 }),
    ],
    ...overrides,
  };
}

/** A `fetch` stub answering with JSON `body`, or 204 when `body` is undefined. */
export function jsonResponse(body?: unknown, status = body === undefined ? 204 : 200): Response {
  return new Response(body === undefined ? null : JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

/** Routes stubbed fetches by `METHOD /path`; unrouted requests reject. */
export function stubApi(routes: Record<string, (body: unknown) => Response | Promise<Response>>) {
  const fetch = vi.fn(async (url: string, init?: RequestInit) => {
    const route = routes[`${init?.method ?? 'GET'} ${url}`];
    if (!route) throw new Error(`unexpected ${init?.method} ${url}`);
    return route(typeof init?.body === 'string' ? JSON.parse(init.body) : undefined);
  });
  vi.stubGlobal('fetch', fetch);
  return fetch;
}
