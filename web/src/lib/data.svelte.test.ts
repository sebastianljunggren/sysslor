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

