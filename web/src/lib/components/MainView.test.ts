import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { data } from '#lib/data.svelte.js';
import { clearQueue, queue } from '#lib/queue.svelte.js';
import { executors, jsonResponse, projectView } from '../../test/fixtures';
import MainView from './MainView.svelte';

const UUIDV7 = /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;

function taskNames() {
  return screen.getAllByRole('button', { name: /^Mark .* as done$/ }).map((b) => b.getAttribute('aria-label')!.slice(5, -8));
}

async function completeAs(user: ReturnType<typeof userEvent.setup>, task: string, person: string) {
  await user.click(screen.getByRole('button', { name: `Mark ${task} as done` }));
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: person }));
}

/**
 * Completions succeed unless `fail()` returns an error status, or `'offline'` to make the
 * request reject. Refetches return the fixtures.
 */
function stubCompletions(fail: () => number | 'offline' | null = () => null) {
  const fetch = vi.fn(async (url: string, init?: RequestInit) => {
    if (url.startsWith('/api/completions/')) {
      if (init?.method === 'DELETE') return jsonResponse();
      const failure = fail();
      if (failure === 'offline') throw new TypeError('Failed to fetch');
      if (failure !== null) return jsonResponse({ error: 'boom' }, failure);
      return jsonResponse({ id: url.split('/').pop(), ...JSON.parse(init!.body as string) });
    }
    if (url === '/api/projects/1') return jsonResponse(projectView());
    if (url === '/api/executors') return jsonResponse(executors);
    throw new Error(`unexpected ${init?.method} ${url}`);
  });
  vi.stubGlobal('fetch', fetch);
  return fetch;
}

describe('MainView', () => {
  beforeEach(() => {
    data.groupFilter = null;
    data.actionError = null;
    clearQueue();
  });

  it('asks to add people and disables completing without any', () => {
    render(MainView, { view: projectView(), executors: [] });
    expect(screen.getByRole('link', { name: 'Add people' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Mark Dishes as done' })).toBeDisabled();
  });

  describe('group filter', () => {
    it('shows all tasks in server order with their groups', () => {
      render(MainView, { view: projectView(), executors });

      expect(screen.getByRole('combobox', { name: 'Groups' })).toHaveDisplayValue('All');
      expect(taskNames()).toEqual(['Dishes', 'Towels', 'Plants', 'Fridge']);
      expect(screen.getAllByText('Kitchen', { selector: '.group' })).toHaveLength(2);
    });

    it('filters on a group, keeping server order and hiding the group tag', async () => {
      const user = userEvent.setup();
      render(MainView, { view: projectView(), executors });

      await user.selectOptions(screen.getByRole('combobox', { name: 'Groups' }), 'Kitchen');

      expect(taskNames()).toEqual(['Dishes', 'Fridge']);
      expect(screen.queryByText('Kitchen', { selector: '.group' })).not.toBeInTheDocument();
      expect(localStorage.getItem('sysslor.group')).toBe('10');
    });

    it('filters on ungrouped tasks', async () => {
      const user = userEvent.setup();
      render(MainView, { view: projectView(), executors });

      await user.selectOptions(screen.getByRole('combobox', { name: 'Groups' }), 'No group');

      expect(taskNames()).toEqual(['Plants']);
    });

    it('falls back to all if the picked group is gone', () => {
      data.groupFilter = 99;
      render(MainView, { view: projectView(), executors });

      expect(screen.getByRole('combobox', { name: 'Groups' })).toHaveDisplayValue('All');
      expect(taskNames()).toHaveLength(4);
    });

    it('offers no group only when some task has none', () => {
      data.groupFilter = 'none';
      const view = projectView();
      view.tasks = view.tasks.filter((t) => t.group_id !== null);
      render(MainView, { view, executors });

      expect(screen.queryByRole('option', { name: 'No group' })).not.toBeInTheDocument();
      expect(screen.getByRole('combobox', { name: 'Groups' })).toHaveDisplayValue('All');
    });

    it('hides the filter without groups', () => {
      render(MainView, { view: projectView({ groups: [] }), executors });
      expect(screen.queryByRole('combobox', { name: 'Groups' })).not.toBeInTheDocument();
    });
  });

  describe('completing', () => {
    it('asks who did it without preselecting anyone', async () => {
      const user = userEvent.setup();
      render(MainView, { view: projectView(), executors });
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

      await user.click(screen.getByRole('button', { name: 'Mark Towels as done' }));

      const dialog = screen.getByRole('dialog', { name: 'Complete Towels' });
      expect(within(dialog).getByRole('button', { name: 'Anna' })).toBeEnabled();
      expect(within(dialog).getByRole('button', { name: 'Bo' })).toBeEnabled();
      expect(within(dialog).getByLabelText('Done at')).toBeInTheDocument();
    });

    it('saves a completion by the picked person and offers to undo it', async () => {
      const user = userEvent.setup();
      const fetch = stubCompletions();
      render(MainView, { view: projectView(), executors });

      await completeAs(user, 'Towels', 'Bo');

      const [url, init] = fetch.mock.calls[0];
      const id = url.replace('/api/completions/', '');
      expect(id).toMatch(UUIDV7);
      expect(JSON.parse(init!.body as string)).toMatchObject({ task_id: 2, executor_id: 2 });
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

      const toast = await screen.findByRole('status');
      expect(toast).toHaveTextContent('Towels done');
      await user.click(within(toast).getByRole('button', { name: 'Undo' }));

      expect(fetch).toHaveBeenCalledWith(`/api/completions/${id}`, expect.objectContaining({ method: 'DELETE' }));
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });

    it('queues a completion without a connection and retries it under the same id', async () => {
      const user = userEvent.setup();
      let offline = true;
      const fetch = stubCompletions(() => (offline ? 'offline' : null));
      render(MainView, { view: projectView(), executors });
      const puts = () => fetch.mock.calls.filter(([, init]) => init?.method === 'PUT').map(([url]) => url);

      await completeAs(user, 'Dishes', 'Anna');
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      expect(await screen.findByRole('status')).toHaveTextContent(
        'Dishes done, will be saved when the connection is back',
      );
      expect(screen.getByText('Waiting to be saved')).toBeInTheDocument();
      expect(screen.getByText(/Completed just now by Anna/)).toBeInTheDocument();
      expect(data.actionError).toBeNull();

      // Sending the next completion sends the queued one first.
      offline = false;
      await completeAs(user, 'Towels', 'Bo');
      expect(await screen.findByRole('status')).toHaveTextContent('Towels done');

      const [first, retry, towels] = puts();
      expect(retry).toBe(first);
      expect(towels).not.toBe(first);
      expect(queue.entries).toEqual([]);
    });

    it('queues a completion while the server is down', async () => {
      const user = userEvent.setup();
      stubCompletions(() => 502);
      render(MainView, { view: projectView(), executors });

      await completeAs(user, 'Dishes', 'Anna');

      expect(await screen.findByRole('status')).toHaveTextContent('will be saved');
      expect(queue.entries).toHaveLength(1);
    });

    it('undoes a queued completion without contacting the server', async () => {
      const user = userEvent.setup();
      const fetch = stubCompletions(() => 'offline');
      render(MainView, { view: projectView(), executors });

      await completeAs(user, 'Dishes', 'Anna');
      await user.click(within(await screen.findByRole('status')).getByRole('button', { name: 'Undo' }));

      expect(queue.entries).toEqual([]);
      expect(fetch.mock.calls.some(([, init]) => init?.method === 'DELETE')).toBe(false);
      expect(screen.queryByText('Waiting to be saved')).not.toBeInTheDocument();
    });

    it('shows why the server rejected a completion and drops it', async () => {
      const user = userEvent.setup();
      stubCompletions(() => 422);
      render(MainView, { view: projectView(), executors });

      await completeAs(user, 'Dishes', 'Anna');

      expect(data.actionError).toBe('Could not save Dishes: boom (422)');
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
      expect(queue.entries).toEqual([]);
    });

    it('hides the undo offer after 8 seconds', async () => {
      vi.useFakeTimers();
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      stubCompletions();
      render(MainView, { view: projectView(), executors });

      await completeAs(user, 'Dishes', 'Anna');
      // vi.waitFor doesn't give Svelte a chance to render under fake timers.
      await tick();
      expect(screen.getByRole('status')).toBeInTheDocument();

      await vi.advanceTimersByTimeAsync(7_900);
      expect(screen.getByRole('status')).toBeInTheDocument();
      await vi.advanceTimersByTimeAsync(200);
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
    });
  });
});
