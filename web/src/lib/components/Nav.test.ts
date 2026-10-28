import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Nav from './Nav.svelte';

describe('Nav', () => {
  it('links every page and marks the current one', () => {
    render(Nav, { page: 'people' });

    expect(screen.getByRole('link', { name: 'To do' })).toHaveAttribute('href', '/');
    expect(screen.getByRole('link', { name: 'Chores' })).toHaveAttribute('href', '/manage/chores');
    expect(screen.getByRole('link', { name: 'Groups' })).toHaveAttribute('href', '/manage/groups');
    expect(screen.getByRole('link', { name: 'Settings' })).toHaveAttribute('href', '/manage/settings');
    expect(screen.getByRole('link', { name: 'People', current: 'page' })).toHaveAttribute('href', '/manage/people');
    expect(screen.getAllByRole('link', { current: 'page' })).toHaveLength(1);
  });

  it('reports navigation', async () => {
    const user = userEvent.setup();
    const onnavigate = vi.fn();
    render(Nav, { page: 'tasks', onnavigate });

    await user.click(screen.getByRole('link', { name: 'Groups' }));

    expect(onnavigate).toHaveBeenCalledOnce();
  });
});
