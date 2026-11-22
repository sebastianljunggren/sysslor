import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { executors, jsonResponse, projectView, stubApi } from '../../test/fixtures';
import { data } from '#lib/data.svelte.js';
import { clearQueue, enqueue } from '#lib/queue.svelte.js';
import Shell from './Shell.svelte';

const navigation = vi.hoisted(() => ({ after: () => {} }));

vi.mock('$app/navigation', () => ({
  afterNavigate: (callback: () => void) => (navigation.after = callback),
}));

const children = createRawSnippet(() => ({ render: () => '<p>Page content</p>' }));

describe('Shell', () => {
  beforeEach(() => {
    clearQueue();
    data.connected = true;
    stubApi({
      'GET /api/projects/1': () => jsonResponse(projectView()),
      'GET /api/executors': () => jsonResponse(executors),
    });
    // jsdom has no EventSource; a stream that never opens is enough here.
    vi.stubGlobal(
      'EventSource',
      class {
        close() {}
      },
    );
  });

  it('opens the menu and closes it after navigating', async () => {
    const user = userEvent.setup();
    render(Shell, { page: 'tasks', children });
    await screen.findByRole('heading', { level: 1, name: 'Home' });

    const button = screen.getByRole('button', { name: 'Menu' });
    expect(button).toHaveAttribute('aria-expanded', 'false');

    await user.click(button);
    expect(button).toHaveAttribute('aria-expanded', 'true');
    const drawer = screen.getByRole('dialog', { name: 'Menu' });
    expect(within(drawer).getByRole('link', { name: 'To do', current: 'page' })).toBeInTheDocument();

    await user.click(within(drawer).getByRole('link', { name: 'People' }));
    expect(button).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes the menu with its close button', async () => {
    const user = userEvent.setup();
    render(Shell, { page: 'tasks', children });
    await screen.findByRole('heading', { level: 1, name: 'Home' });

    const button = screen.getByRole('button', { name: 'Menu' });
    await user.click(button);
    await user.click(within(screen.getByRole('dialog', { name: 'Menu' })).getByRole('button', { name: 'Close menu' }));

    expect(button).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('closes the menu after navigating without it, like with the back button', async () => {
    const user = userEvent.setup();
    render(Shell, { page: 'tasks', children });
    await screen.findByRole('heading', { level: 1, name: 'Home' });

    const button = screen.getByRole('button', { name: 'Menu' });
    await user.click(button);
    navigation.after();

    await vi.waitFor(() => expect(button).toHaveAttribute('aria-expanded', 'false'));
  });

  it('shows the page once the project has loaded', async () => {
    render(Shell, { page: 'people', children });

    expect(await screen.findByText('Page content')).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1, name: 'People' })).toBeInTheDocument();
  });

  it('follows page changes', async () => {
    const { rerender } = render(Shell, { page: 'tasks', children });
    await screen.findByRole('heading', { level: 1, name: 'Home' });

    await rerender({ page: 'groups', children });

    expect(screen.getByRole('heading', { level: 1, name: 'Groups' })).toBeInTheDocument();
  });

  it('counts completions waiting for a connection', async () => {
    for (const id of ['a', 'b']) {
      enqueue({ id, taskName: 'Dishes', input: { task_id: 1, executor_id: 1, completed_at: 0 } });
    }
    // The routes above don't answer PUTs, so sending them fails like it would offline.
    render(Shell, { page: 'tasks', children });

    expect(await screen.findByText('2 chores done offline, waiting to be saved.')).toBeInTheDocument();
  });

  it('says in one banner that it is offline and what will be saved later', async () => {
    enqueue({ id: 'a', taskName: 'Dishes', input: { task_id: 1, executor_id: 1, completed_at: 0 } });
    render(Shell, { page: 'tasks', children });
    data.connected = false;

    const banner = await screen.findByText(/No connection to the server/);
    expect(banner).toHaveTextContent(
      'No connection to the server. 1 completed chore will be saved when the connection is back.',
    );
    expect(screen.getAllByRole('status')).toHaveLength(1);
  });
});
