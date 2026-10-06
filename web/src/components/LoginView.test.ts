import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { jsonResponse, stubApi } from '../test/fixtures';
import LoginView from './LoginView.svelte';

describe('LoginView', () => {
  it('shows an error for a wrong password and keeps it entered', async () => {
    const user = userEvent.setup();
    const fetch = stubApi({ 'POST /api/login': () => jsonResponse({ error: 'unauthorized' }, 401) });
    render(LoginView);

    await user.type(screen.getByLabelText('Password'), 'guess');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('Wrong password.');
    expect(screen.getByLabelText('Password')).toHaveValue('guess');
    expect(fetch).toHaveBeenCalledWith('/api/login', expect.objectContaining({ body: '{"password":"guess"}' }));
  });

  it('clears the password once logged in', async () => {
    const user = userEvent.setup();
    stubApi({ 'POST /api/login': () => jsonResponse() });
    render(LoginView);

    await user.type(screen.getByLabelText('Password'), 'right');
    await user.click(screen.getByRole('button', { name: 'Log in' }));

    await vi.waitFor(() => expect(screen.getByLabelText('Password')).toHaveValue(''));
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});
