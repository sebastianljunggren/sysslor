import { beforeEach, describe, expect, it, vi } from 'vitest';
import { executors, jsonResponse, projectView, stubApi } from '../test/fixtures';

/** A fresh module, so `data` is initialized from the current localStorage. */
async function load() {
  vi.resetModules();
  return import('./data.svelte');
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => (resolve = r));
  return { promise, resolve };
}

describe('refreshProject', () => {
  it('ignores a slower, older response', async () => {
    const { data, refreshProject } = await load();
    const older = deferred<Response>();
    const newer = projectView({ project: { id: 1, name: 'Newer' } });
    const responses = [older.promise, Promise.resolve(jsonResponse(newer))];
    stubApi({ 'GET /api/projects/1': () => responses.shift()! });

    const first = refreshProject();
    await refreshProject();
    older.resolve(jsonResponse(projectView({ project: { id: 1, name: 'Older' } })));
    await first;

    expect(data.view?.project.name).toBe('Newer');
  });

  it('shows load errors and clears them on success', async () => {
    const { data, refreshProject } = await load();
    let fail = true;
    stubApi({
      'GET /api/projects/1': () => (fail ? jsonResponse({ error: 'boom' }, 500) : jsonResponse(projectView())),
    });

    await refreshProject();
    expect(data.loadError).toBe('boom (500)');

    fail = false;
    await refreshProject();
    expect(data.loadError).toBeNull();
  });

  it('logs out on 401', async () => {
    const { data, refreshProject } = await load();
    data.view = projectView();
    data.executors = executors;
    stubApi({ 'GET /api/projects/1': () => jsonResponse({ error: 'unauthorized' }, 401) });

    await refreshProject();

    expect(data).toMatchObject({ loggedIn: false, view: null, executors: [], loadError: null });
  });
});

describe('mutate', () => {
  it('returns the result and refetches', async () => {
    const { data, mutate } = await load();
    data.actionError = 'old error';
    const fetch = stubApi({
      'GET /api/projects/1': () => jsonResponse(projectView()),
      'GET /api/executors': () => jsonResponse(executors),
    });

    await expect(mutate(async () => 'saved')).resolves.toBe('saved');

    expect(data.actionError).toBeNull();
    await vi.waitFor(() => expect(data.view).not.toBeNull());
    expect(fetch).toHaveBeenCalledTimes(2);
  });

  it('shows the error and returns undefined on failure', async () => {
    const { data, mutate } = await load();
    const fetch = stubApi({});

    await expect(mutate(() => Promise.reject(new Error('nope')))).resolves.toBeUndefined();

    expect(data.actionError).toBe('nope');
    expect(fetch).not.toHaveBeenCalled();
  });
});

describe('logIn', () => {
  it('says the password is wrong on 401', async () => {
    const { data, logIn } = await load();
    data.loggedIn = false;
    stubApi({ 'POST /api/login': () => jsonResponse({ error: 'unauthorized' }, 401) });

    await expect(logIn('guess')).resolves.toBe('Wrong password.');
    expect(data.loggedIn).toBe(false);
  });

  it('clears old errors once logged in', async () => {
    const { data, logIn } = await load();
    Object.assign(data, { loggedIn: false, loadError: 'a', actionError: 'b' });
    stubApi({ 'POST /api/login': () => jsonResponse() });

    await expect(logIn('right')).resolves.toBeNull();
    expect(data).toMatchObject({ loggedIn: true, loadError: null, actionError: null });
  });
});

describe('logOut', () => {
  it('stays logged in if the request fails', async () => {
    const { data, logOut } = await load();
    data.view = projectView();
    stubApi({ 'POST /api/logout': () => jsonResponse({ error: 'boom' }, 500) });

    await logOut();

    expect(data.loggedIn).toBe(true);
    expect(data.view).not.toBeNull();
    expect(data.actionError).toBe('boom (500)');
  });
});

describe('device preferences', () => {
  beforeEach(() => localStorage.clear());

  it('remembers the picked group across reloads', async () => {
    const first = await load();
    first.pickGroupFilter(10);

    const { data } = await load();
    expect(data.groupFilter).toBe(10);
  });

  it.each([
    ['none', 'none'],
    ['11', 11],
    ['kitchen', null],
    ['1.5', null],
  ] as const)('reads stored group filter %j as %j', async (stored, expected) => {
    localStorage.setItem('sysslor.group', stored);
    const { data } = await load();
    expect(data.groupFilter).toBe(expected);
  });

  it('forgets the group filter when showing all', async () => {
    const { pickGroupFilter } = await load();
    pickGroupFilter('none');
    pickGroupFilter(null);
    expect(localStorage.getItem('sysslor.group')).toBeNull();
  });

  it('works without storage', async () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new DOMException('denied', 'SecurityError');
    });
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new DOMException('denied', 'SecurityError');
    });

    const { data, pickGroupFilter } = await load();
    expect(data.groupFilter).toBeNull();

    pickGroupFilter(10);
    expect(data.groupFilter).toBe(10);
  });
});

describe('connect', () => {
  it('resets the connection banner when disconnecting', async () => {
    const sources: { onerror: (() => void) | null; readyState: number; close(): void }[] = [];
    vi.stubGlobal(
      'EventSource',
      class {
        static CLOSED = 2;
        readyState = 0;
        onerror: (() => void) | null = null;
        constructor() {
          sources.push(this);
        }
        close() {}
      },
    );
    const { data, connect } = await load();

    const disconnect = connect();
    sources[0].onerror?.();
    expect(data.connected).toBe(false);

    disconnect();
    expect(data.connected).toBe(true);
  });
});


describe('flushQueue', () => {
  const entry = (id: string, completedAt = 1000) => ({
    id,
    taskName: `Task ${id}`,
    input: { task_id: 1, executor_id: 1, completed_at: completedAt },
  });

  async function loadWithQueue(...ids: string[]) {
    const store = await load();
    const queue = await import('./queue.svelte');
    for (const id of ids) queue.enqueue(entry(id));
    return { ...store, ...queue };
  }

  /** PUTs answer with `status(id)`, `'offline'` rejects. Records the request order. */
  function stubPuts(status: (id: string) => number | 'offline' = () => 200) {
    const requests: string[] = [];
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: string, init?: RequestInit) => {
        const method = init?.method ?? 'GET';
        requests.push(`${method} ${url}`);
        if (url === '/api/projects/1') return jsonResponse(projectView());
        if (url === '/api/executors') return jsonResponse(executors);
        if (method === 'DELETE') return jsonResponse();
        const id = url.split('/').pop()!;
        const answer = status(id);
        if (answer === 'offline') throw new TypeError('Failed to fetch');
        return jsonResponse(answer === 200 ? { id, ...JSON.parse(init!.body as string) } : { error: 'boom' }, answer);
      }),
    );
    return requests;
  }

  const puts = (requests: string[]) => requests.filter((r) => r.startsWith('PUT'));

  it('sends entries in order, then refetches', async () => {
    const { flushQueue, queue } = await loadWithQueue('a', 'b');
    const requests = stubPuts();

    expect(await flushQueue()).toEqual(new Set(['a', 'b']));

    expect(puts(requests)).toEqual(['PUT /api/completions/a', 'PUT /api/completions/b']);
    expect(queue.entries).toEqual([]);
    await vi.waitFor(() => expect(requests).toContain('GET /api/projects/1'));
  });

  it('keeps the rest in order when the server is out of reach', async () => {
    const { flushQueue, queue } = await loadWithQueue('a', 'b', 'c');
    const requests = stubPuts((id) => (id === 'b' ? 'offline' : 200));

    await flushQueue();

    expect(puts(requests)).toHaveLength(2);
    expect(queue.entries.map((e) => e.id)).toEqual(['b', 'c']);
    expect(JSON.parse(localStorage.getItem('sysslor.queue')!)).toHaveLength(2);
  });

  it('keeps entries the server fails on, and drops ones it rejects', async () => {
    const { data, flushQueue, queue } = await loadWithQueue('a', 'b');
    stubPuts((id) => (id === 'a' ? 422 : 503));

    await flushQueue();

    expect(data.actionError).toBe('Could not save Task a: boom (422)');
    expect(queue.entries.map((e) => e.id)).toEqual(['b']);
  });

  it('keeps the queue for after logging in again', async () => {
    const { data, flushQueue, queue } = await loadWithQueue('a');
    stubPuts(() => 401);

    await flushQueue();

    expect(data.loggedIn).toBe(false);
    expect(queue.entries).toHaveLength(1);
  });

  it('sends once when asked again while sending', async () => {
    const { flushQueue } = await loadWithQueue('a');
    const requests = stubPuts();

    await Promise.all([flushQueue(), flushQueue()]);

    expect(puts(requests)).toHaveLength(1);
  });

  it('sends the new version of an entry backdated while it was being sent', async () => {
    const { flushQueue, enqueue, queue } = await loadWithQueue('a');
    const bodies: unknown[] = [];
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: string, init?: RequestInit) => {
        if (init?.method !== 'PUT') return jsonResponse(url === '/api/executors' ? [] : projectView());
        bodies.push(JSON.parse(init.body as string));
        if (bodies.length === 1) enqueue(entry('a', 500));
        return jsonResponse({ id: 'a', ...(bodies.at(-1) as object) });
      }),
    );

    await flushQueue();

    expect(bodies).toMatchObject([{ completed_at: 1000 }, { completed_at: 500 }]);
    expect(queue.entries).toEqual([]);
  });

  it('deletes an entry undone while it was being sent', async () => {
    const { flushQueue, undoCompletion, queue } = await loadWithQueue('a');
    const requests = stubPuts();

    const flushed = flushQueue();
    const undone = undoCompletion('a');
    await Promise.all([flushed, undone]);

    expect(requests.filter((r) => !r.startsWith('GET'))).toEqual([
      'PUT /api/completions/a',
      'DELETE /api/completions/a',
    ]);
    expect(queue.entries).toEqual([]);
  });

  it('tells whether a completion was saved, queued or rejected', async () => {
    const { saveCompletion } = await loadWithQueue();
    let status: number | 'offline' = 200;
    stubPuts(() => status);

    expect(await saveCompletion(entry('a'))).toBe('saved');
    status = 'offline';
    expect(await saveCompletion(entry('b'))).toBe('queued');
    status = 422;
    expect(await saveCompletion(entry('b'))).toBe('failed');
  });

  it('is sent when the browser is back online', async () => {
    vi.stubGlobal(
      'EventSource',
      class {
        close() {}
      },
    );
    const { connect, queue } = await loadWithQueue('a');
    let online = false;
    const requests = stubPuts(() => (online ? 200 : 'offline'));
    const disconnect = connect();
    await vi.waitFor(() => expect(puts(requests)).toHaveLength(1));

    online = true;
    window.dispatchEvent(new Event('online'));

    await vi.waitFor(() => expect(queue.entries).toEqual([]));
    disconnect();
  });

  it('is cleared by logging out', async () => {
    const { logOut, queue } = await loadWithQueue('a');
    stubApi({ 'POST /api/logout': () => jsonResponse() });

    await logOut();

    expect(queue.entries).toEqual([]);
    expect(localStorage.getItem('sysslor.queue')).toBeNull();
  });
});
