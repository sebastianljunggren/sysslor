import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { groups } from '../test/fixtures';
import TaskForm from './TaskForm.svelte';

describe('TaskForm', () => {
  it('submits the entered task', async () => {
    const user = userEvent.setup();
    const onsubmit = vi.fn().mockResolvedValue(true);
    render(TaskForm, { groups, submitLabel: 'Add', onsubmit });

    await user.type(screen.getByLabelText('Name'), 'Vacuum');
    const amount = screen.getByRole('spinbutton');
    await user.clear(amount);
    await user.type(amount, '2');
    // The unit select has no accessible name of its own; the "Every" label names the amount.
    await user.selectOptions(screen.getAllByRole('combobox')[0], 'weeks');
    await user.selectOptions(screen.getByLabelText('Priority'), '3');
    await user.selectOptions(screen.getByLabelText('Group'), 'Bathroom');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(onsubmit).toHaveBeenCalledWith({
      name: 'Vacuum',
      cadence: { amount: 2, unit: 'weeks' },
      priority: 3,
      group_id: 11,
    });
  });

  it('clears the name but keeps the group after creating', async () => {
    const user = userEvent.setup();
    const onsubmit = vi.fn().mockResolvedValue(true);
    render(TaskForm, { groups, submitLabel: 'Add', onsubmit });

    await user.type(screen.getByLabelText('Name'), 'Vacuum');
    await user.selectOptions(screen.getByLabelText('Group'), 'Kitchen');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByLabelText('Name')).toHaveValue('');
    expect(screen.getByLabelText('Group')).toHaveDisplayValue('Kitchen');
  });

  it('keeps the name when saving fails', async () => {
    const user = userEvent.setup();
    render(TaskForm, { groups, submitLabel: 'Add', onsubmit: vi.fn().mockResolvedValue(false) });

    await user.type(screen.getByLabelText('Name'), 'Vacuum');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByLabelText('Name')).toHaveValue('Vacuum');
  });

  it('keeps the name when editing', async () => {
    const user = userEvent.setup();
    render(TaskForm, {
      initial: { name: 'Dishes', cadence: { amount: 1, unit: 'days' }, priority: 2, group_id: 10 },
      groups,
      submitLabel: 'Save',
      onsubmit: vi.fn().mockResolvedValue(true),
      oncancel: vi.fn(),
    });

    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(screen.getByLabelText('Name')).toHaveValue('Dishes');
  });

  it('disables submitting while saving', async () => {
    const user = userEvent.setup();
    let finish!: (saved: boolean) => void;
    const onsubmit = vi.fn(() => new Promise<boolean>((resolve) => (finish = resolve)));
    render(TaskForm, { groups, submitLabel: 'Add', onsubmit });

    await user.type(screen.getByLabelText('Name'), 'Vacuum');
    await user.click(screen.getByRole('button', { name: 'Add' }));
    expect(screen.getByRole('button', { name: 'Add' })).toBeDisabled();

    finish(true);
    await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Add' })).toBeEnabled());
  });

  it('hides the group picker without groups', () => {
    render(TaskForm, { groups: [], submitLabel: 'Add', onsubmit: vi.fn() });
    expect(screen.queryByLabelText('Group')).not.toBeInTheDocument();
  });
});
