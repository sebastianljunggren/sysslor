import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { clock } from '#lib/clock.svelte.js';
import { clearQueue, enqueue, queue, type QueuedCompletion } from '#lib/queue.svelte.js';
import { executors, groups, jsonResponse, stubApi, task } from '../../test/fixtures';
import TaskItem from './TaskItem.svelte';

const now = new Date(2026, 9, 5, 18, 0);
const doneTask = task({
  priority: 2,
  due: '2026-10-06',
  overdue: false,
  last_completion: { id: 'c1', task_id: 1, executor_id: 2, completed_at: new Date(2026, 9, 5, 7, 30).getTime() },
});

function renderItem(props: Partial<Parameters<typeof render<typeof TaskItem>>[1] & object> = {}) {
  return render(TaskItem, { task: doneTask, executors, canComplete: true, oncomplete: vi.fn(), ...props });
}

describe('TaskItem', () => {
  beforeEach(() => {
    // Only `Date`, so user-event's timers keep running.
    vi.useFakeTimers({ now, toFake: ['Date'] });
    clock.now = now.getTime();
    clearQueue();
  });

  it('summarizes the task', () => {
    renderItem({ group: groups[0] });

    expect(screen.getByText('!!')).toHaveAttribute('title', 'Priority 2');
    expect(screen.getByText('Kitchen')).toHaveClass('colored');
    expect(screen.getByText('Due tomorrow')).toHaveAttribute('title', 'Tuesday, October 6, 2026');
    expect(screen.getByText('Completed 10 hours ago by Bo')).toHaveAttribute(
      'title',
      expect.stringMatching(/^Monday, October 5, 2026 at 7:30\sAM$/),
    );
  });

  it('marks overdue tasks and has nothing to expand before the first completion', () => {
    const { container } = renderItem({ task: task() });

    expect(container.querySelector('article')).toHaveClass('overdue');
    expect(screen.getByText('Never done')).toBeInTheDocument();
    expect(screen.getByRole('button', { expanded: false })).toBeDisabled();
  });

  it('completes the task unless there is no one to do it', async () => {
    const user = userEvent.setup();
    const oncomplete = vi.fn();
    const { rerender } = renderItem({ oncomplete });

    await user.click(screen.getByRole('button', { name: 'Mark Dishes as done' }));
    expect(oncomplete).toHaveBeenCalledOnce();

    await rerender({ canComplete: false });
    expect(screen.getByRole('button', { name: 'Mark Dishes as done' })).toBeDisabled();
  });

  it('moves the last completion to an earlier time under the same id', async () => {
    const user = userEvent.setup();
    const fetch = stubApi({
      'PUT /api/completions/c1': (body) => jsonResponse({ id: 'c1', ...(body as object) }),
      'GET /api/projects/1': () => jsonResponse(null),
      'GET /api/executors': () => jsonResponse([]),
    });
    renderItem();

    await user.click(screen.getByRole('button', { expanded: false }));
    const input = screen.getByLabelText('Done at');
    expect(input).toHaveValue('2026-10-05T07:30');

    await fireEvent.input(input, { target: { value: '2026-10-04T21:15' } });
    await user.click(screen.getByRole('button', { name: 'Change time' }));

    expect(fetch).toHaveBeenCalledWith(
      '/api/completions/c1',
      expect.objectContaining({
        method: 'PUT',
        body: JSON.stringify({ task_id: 1, executor_id: 2, completed_at: new Date(2026, 9, 4, 21, 15).getTime() }),
      }),
    );
    await vi.waitFor(() => expect(screen.queryByLabelText('Done at')).not.toBeInTheDocument());
  });

  it('never moves a completion into the future', async () => {
    const user = userEvent.setup();
    const fetch = stubApi({
      'PUT /api/completions/c1': (body) => jsonResponse({ id: 'c1', ...(body as object) }),
      'GET /api/projects/1': () => jsonResponse(null),
      'GET /api/executors': () => jsonResponse([]),
    });
    renderItem();

    await user.click(screen.getByRole('button', { expanded: false }));
    const input = screen.getByLabelText('Done at');
    // Like browsers that don't enforce `max` on datetime-local inputs, e.g. iOS Safari.
    input.removeAttribute('max');
    await fireEvent.input(input, { target: { value: '2026-10-05T23:00' } });
    await user.click(screen.getByRole('button', { name: 'Change time' }));

    const body = JSON.parse(fetch.mock.calls[0][1]!.body as string);
    expect(body.completed_at).toBe(now.getTime());
  });

  it('undoes the last completion', async () => {
    const user = userEvent.setup();
    const fetch = stubApi({
      'DELETE /api/completions/c1': () => jsonResponse(),
      'GET /api/projects/1': () => jsonResponse(null),
      'GET /api/executors': () => jsonResponse([]),
    });
    renderItem();

    await user.click(screen.getByRole('button', { expanded: false }));
    await user.click(screen.getByRole('button', { name: 'Undo' }));

    expect(fetch).toHaveBeenCalledWith('/api/completions/c1', expect.objectContaining({ method: 'DELETE' }));
  });

  describe('with a completion waiting to be sent', () => {
    const pending: QueuedCompletion = {
      id: 'p1',
      taskName: 'Dishes',
      input: { task_id: 1, executor_id: 1, completed_at: now.getTime() - 60_000 },
    };

    it('shows it instead of the due date and last completion', () => {
      const { container } = renderItem({ task: task(), pending });

      expect(container.querySelector('article')).not.toHaveClass('overdue');
      expect(container.querySelector('article')).toHaveClass('pending');
      expect(screen.getByText('Waiting to be saved')).toBeInTheDocument();
      expect(screen.queryByText('Never done')).not.toBeInTheDocument();
      expect(screen.getByText('Completed 1 minute ago by Anna')).toBeInTheDocument();
    });

    it('backdates it in the queue while offline', async () => {
      const user = userEvent.setup();
      vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')));
      enqueue(pending);
      renderItem({ pending });

      await user.click(screen.getByRole('button', { expanded: false }));
      await fireEvent.input(screen.getByLabelText('Done at'), { target: { value: '2026-10-04T21:15' } });
      await user.click(screen.getByRole('button', { name: 'Change time' }));

      await vi.waitFor(() => expect(screen.queryByLabelText('Done at')).not.toBeInTheDocument());
      expect(queue.entries).toEqual([
        { ...pending, input: { ...pending.input, completed_at: new Date(2026, 9, 4, 21, 15).getTime() } },
      ]);
    });

    it('undoes it without contacting the server', async () => {
      const user = userEvent.setup();
      const fetch = vi.fn();
      vi.stubGlobal('fetch', fetch);
      enqueue(pending);
      renderItem({ pending });

      await user.click(screen.getByRole('button', { expanded: false }));
      await user.click(screen.getByRole('button', { name: 'Undo' }));

      expect(queue.entries).toEqual([]);
      expect(fetch).not.toHaveBeenCalled();
    });
  });
});
