import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { jsonResponse } from '../test/fixtures';
import { ApiError, deleteCompletion, getProject, isUnauthorized, putCompletion, subscribe, uuidv7 } from './client';

describe('request', () => {
  it('sends JSON only when there is a body', async () => {
    const fetch = vi.fn().mockImplementation(async () => jsonResponse({ id: 'x' }));
    vi.stubGlobal('fetch', fetch);

    await putCompletion('x', { task_id: 1, executor_id: 2, completed_at: 3 });
    await getProject(1);

    expect(fetch).toHaveBeenNthCalledWith(1, '/api/completions/x', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: '{"task_id":1,"executor_id":2,"completed_at":3}',
    });
    expect(fetch).toHaveBeenNthCalledWith(2, '/api/projects/1', { method: 'GET', headers: {}, body: undefined });
  });

  it('returns undefined for 204', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse()));
    await expect(deleteCompletion('x')).resolves.toBeUndefined();
  });

  it('uses the error body as the message', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse({ error: 'no such task' }, 404)));
    const error = await getProject(1).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({ message: 'no such task (404)', status: 404 });
  });

  it('falls back to the status text for non-JSON errors', async () => {
    const response = new Response('<html>bad gateway</html>', { status: 502, statusText: 'Bad Gateway' });
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response));
    await expect(getProject(1)).rejects.toMatchObject({ message: 'Bad Gateway (502)', status: 502 });
  });

  it('reports an unreachable server without a status', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')));
    await expect(getProject(1)).rejects.toMatchObject({ message: 'Could not reach the server.', status: null });
  });
});

describe('isUnauthorized', () => {
  it('only matches API errors with status 401', () => {
    expect(isUnauthorized(new ApiError('', 401))).toBe(true);
    expect(isUnauthorized(new ApiError('', 403))).toBe(false);
    expect(isUnauthorized(new ApiError('', null))).toBe(false);
    expect(isUnauthorized(new Error('401'))).toBe(false);
  });
});

describe('uuidv7', () => {
  it('is an RFC 9562 version 7 UUID', () => {
    expect(uuidv7()).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  });

  it('starts with the current unix milliseconds', () => {
    vi.useFakeTimers({ now: 0x0199_b2c3_d4e5 });
    expect(uuidv7().slice(0, 13)).toBe('0199b2c3-d4e5');
  });

  it('sorts by creation time across milliseconds', () => {
    vi.useFakeTimers({ now: 1_000 });
    const first = uuidv7();
    vi.advanceTimersByTime(1);
    expect(uuidv7() > first).toBe(true);
  });

  it('is unique within a millisecond', () => {
    vi.useFakeTimers({ now: 1_000 });
    expect(new Set(Array.from({ length: 100 }, uuidv7)).size).toBe(100);
  });
});

class FakeEventSource {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 2;
  static instances: FakeEventSource[] = [];

  readyState = FakeEventSource.CONNECTING;
  onopen: (() => void) | null = null;
  onerror: (() => void) | null = null;
  onmessage: ((message: { data: string }) => void) | null = null;

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this);
  }

  close() {
    this.readyState = FakeEventSource.CLOSED;
  }
}

describe('subscribe', () => {
  let onChange: ReturnType<typeof vi.fn<(project: number | null) => void>>;
  let onConnection: ReturnType<typeof vi.fn<(connected: boolean) => void>>;

  beforeEach(() => {
    vi.useFakeTimers();
    FakeEventSource.instances = [];
    vi.stubGlobal('EventSource', FakeEventSource);
    onChange = vi.fn();
    onConnection = vi.fn();
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  it('refetches everything on connect, then the project in each event', () => {
    subscribe(onChange, onConnection);
    const [source] = FakeEventSource.instances;
    expect(source.url).toBe('/api/events');

    source.onopen?.();
    source.onmessage?.({ data: '{"type":"changed","project":1}' });

    expect(onConnection).toHaveBeenCalledWith(true);
    expect(onChange.mock.calls).toEqual([[null], [1]]);
  });

  it('leaves transient errors to the browser to retry', () => {
    subscribe(onChange, onConnection);
    FakeEventSource.instances[0].onerror?.();
    vi.advanceTimersByTime(10_000);

    expect(onConnection).toHaveBeenCalledWith(false);
    expect(onChange).not.toHaveBeenCalled();
    expect(FakeEventSource.instances).toHaveLength(1);
  });

  it('reopens a stream the browser gave up on', () => {
    subscribe(onChange, onConnection);
    const [source] = FakeEventSource.instances;
    source.close();
    source.onerror?.();

    expect(onChange).toHaveBeenCalledWith(null);
    vi.advanceTimersByTime(5000);
    expect(FakeEventSource.instances).toHaveLength(2);
  });

  it('stops retrying once unsubscribed', () => {
    const unsubscribe = subscribe(onChange, onConnection);
    const [source] = FakeEventSource.instances;
    source.close();
    source.onerror?.();
    unsubscribe();

    vi.advanceTimersByTime(5000);
    expect(FakeEventSource.instances).toHaveLength(1);
  });
});
