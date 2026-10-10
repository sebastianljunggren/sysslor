import { fireEvent, render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { executors, task } from '../test/fixtures';
import CompleteDialog from './CompleteDialog.svelte';

const now = new Date(2026, 9, 5, 18, 0);

function renderDialog(props: Partial<Parameters<typeof render<typeof CompleteDialog>>[1] & object> = {}) {
  const oncomplete = vi.fn().mockResolvedValue(undefined);
  const onclose = vi.fn();
  render(CompleteDialog, { task: task(), executors, oncomplete, onclose, ...props });
  return { oncomplete, onclose };
}

describe('CompleteDialog', () => {
  beforeEach(() => {
    // Only `Date`, so user-event's timers keep running.
    vi.useFakeTimers({ now, toFake: ['Date'] });
  });

  it('asks who did it, now by default', () => {
    renderDialog();

    const dialog = screen.getByRole('dialog', { name: 'Complete Dishes' });
    const names = within(dialog).getByRole('group', { name: 'Who is doing it?' });
    expect(within(names).getAllByRole('button').map((b) => b.textContent)).toEqual(['Anna', 'Bo']);
    expect(within(dialog).getByLabelText('Done at')).toHaveValue('2026-10-05T18:00');
  });

  it('stays closed without a task', () => {
    renderDialog({ task: null });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('completes as the picked person at the picked time', async () => {
    const user = userEvent.setup();
    const { oncomplete, onclose } = renderDialog();

    await fireEvent.input(screen.getByLabelText('Done at'), { target: { value: '2026-10-05T07:30' } });
    await user.click(screen.getByRole('button', { name: 'Bo' }));

    expect(oncomplete).toHaveBeenCalledWith(2, new Date(2026, 9, 5, 7, 30).getTime());
    expect(onclose).toHaveBeenCalledOnce();
  });

  it('never completes in the future', async () => {
    const user = userEvent.setup();
    const { oncomplete } = renderDialog();
    const input = screen.getByLabelText('Done at');
    // Like browsers that don't enforce `max` on datetime-local inputs, e.g. iOS Safari.
    input.removeAttribute('max');

    await fireEvent.input(input, { target: { value: '2026-10-05T23:00' } });
    await user.click(screen.getByRole('button', { name: 'Anna' }));

    expect(oncomplete).toHaveBeenCalledWith(1, now.getTime());
  });

  it('cancels without completing', async () => {
    const user = userEvent.setup();
    const { oncomplete, onclose } = renderDialog();

    await user.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(oncomplete).not.toHaveBeenCalled();
    expect(onclose).toHaveBeenCalledOnce();
  });
});
