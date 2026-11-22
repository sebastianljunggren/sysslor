import { describe, expect, it, vi } from 'vitest';
import type { QueuedCompletion } from './queue.svelte';

/** A fresh module, so the queue is initialized from the current localStorage. */
async function load() {
  vi.resetModules();
  return import('./queue.svelte');
}

function entry(id: string, taskId = 1, completedAt = 1000): QueuedCompletion {
  return { id, taskName: 'Dishes', input: { task_id: taskId, executor_id: 1, completed_at: completedAt } };
}

describe('queue', () => {
  it('survives a reload', async () => {
    const { enqueue } = await load();
    enqueue(entry('a'));
    enqueue(entry('b'));

    const { queue } = await load();
    expect(queue.entries.map((e) => e.id)).toEqual(['a', 'b']);
  });

  it('replaces an entry with the same id in place', async () => {
    const { enqueue, queue } = await load();
    enqueue(entry('a'));
    enqueue(entry('b'));
    enqueue(entry('a', 1, 500));

    expect(queue.entries).toEqual([entry('a', 1, 500), entry('b')]);
  });

  it('removes entries and says whether they were queued', async () => {
    const { enqueue, dequeue, isQueued } = await load();
    enqueue(entry('a'));

    expect(dequeue('a')).toBe(true);
    expect(dequeue('a')).toBe(false);
    expect(isQueued('a')).toBe(false);
    expect(localStorage.getItem('sysslor.queue')).toBeNull();
  });

  it('finds the latest pending completion of a task', async () => {
    const { enqueue, pendingFor } = await load();
    enqueue(entry('later', 1, 2000));
    enqueue(entry('earlier', 1, 1000));
    enqueue(entry('other', 2, 3000));

    expect(pendingFor(1)?.id).toBe('later');
    expect(pendingFor(3)).toBeUndefined();
  });

  it('picks up changes from other tabs', async () => {
    const { enqueue, queue } = await load();
    enqueue(entry('a'));

    localStorage.setItem('sysslor.queue', JSON.stringify([entry('a'), entry('b')]));
    window.dispatchEvent(new StorageEvent('storage', { key: 'sysslor.queue' }));

    expect(queue.entries.map((e) => e.id)).toEqual(['a', 'b']);
  });

  it('works without storage', async () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('denied');
    });
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('denied');
    });
    const { enqueue, queue } = await load();

    enqueue(entry('a'));
    expect(queue.entries).toHaveLength(1);
  });
});
