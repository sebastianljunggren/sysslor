import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App.svelte';
import { executors, jsonResponse, projectView, stubApi } from './test/fixtures';

describe('App', () => {
  beforeEach(() => {
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

  afterEach(() => {
    vi.unstubAllGlobals();
    history.replaceState(null, '', '#/');
  });

  it('opens the menu and closes it after navigating', async () => {
    const user = userEvent.setup();
    render(App);
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
    render(App);
    await screen.findByRole('heading', { level: 1, name: 'Home' });

    const button = screen.getByRole('button', { name: 'Menu' });
    await user.click(button);
    await user.click(within(screen.getByRole('dialog', { name: 'Menu' })).getByRole('button', { name: 'Close menu' }));

    expect(button).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('shows the page in the hash', async () => {
    history.replaceState(null, '', '#manage/people');
    render(App);

    expect(await screen.findByRole('heading', { level: 1, name: 'People' })).toBeInTheDocument();
    expect(await screen.findByDisplayValue('Anna')).toBeInTheDocument();
  });

  it('follows hash changes', async () => {
    render(App);
    await screen.findByRole('heading', { level: 1, name: 'Home' });

    location.hash = '#manage/groups';

    expect(await screen.findByRole('heading', { level: 1, name: 'Groups' })).toBeInTheDocument();
  });
});
