import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { executors, projectView, task } from '../test/fixtures';
import EditView from './EditView.svelte';

/** Group headings followed by the chore names under them, in document order. */
function choreOutline() {
  return [...document.querySelectorAll('h3, li strong')].map((el) => (el.tagName === 'H3' ? `# ${el.textContent!.trim()}` : el.textContent));
}

describe('EditView', () => {
  it('shows only the selected tab', () => {
    render(EditView, { view: projectView(), executors, tab: 'chores' });

    expect(screen.getByRole('link', { name: 'Chores', current: 'page' })).toHaveAttribute('href', '#edit');
    expect(screen.getByRole('link', { name: 'People' })).toHaveAttribute('href', '#edit/people');
    expect(screen.getByRole('searchbox', { name: 'Search chores' })).toBeInTheDocument();
    expect(screen.queryByDisplayValue('Anna')).not.toBeInTheDocument();
  });

  it('lists people on the people tab', () => {
    render(EditView, { view: projectView(), executors, tab: 'people' });

    expect(screen.getByRole('link', { name: 'People', current: 'page' })).toBeInTheDocument();
    expect(screen.getByDisplayValue('Anna')).toBeInTheDocument();
    expect(screen.queryByRole('searchbox')).not.toBeInTheDocument();
  });

  it('lists chores under their group, ungrouped last', () => {
    render(EditView, { view: projectView(), executors, tab: 'chores' });

    expect(choreOutline()).toEqual(['# Kitchen', 'Dishes', 'Fridge', '# Bathroom', 'Towels', '# No group', 'Plants']);
  });

  it('lists chores without headings when there are no groups', () => {
    const view = projectView({ groups: [], tasks: [task({ id: 1, name: 'Plants' }), task({ id: 2, name: 'Dishes' })] });
    render(EditView, { view, executors, tab: 'chores' });

    expect(choreOutline()).toEqual(['Dishes', 'Plants']);
  });

  it('filters chores by name and drops empty groups', async () => {
    const user = userEvent.setup();
    render(EditView, { view: projectView(), executors, tab: 'chores' });

    await user.type(screen.getByRole('searchbox', { name: 'Search chores' }), 'TOW');
    expect(choreOutline()).toEqual(['# Bathroom', 'Towels']);

    await user.type(screen.getByRole('searchbox', { name: 'Search chores' }), 'x');
    expect(choreOutline()).toEqual([]);
    expect(screen.getByText('No matching chores.')).toBeInTheDocument();
  });

  it('opens the new chore form on demand', async () => {
    const user = userEvent.setup();
    render(EditView, { view: projectView(), executors, tab: 'chores' });

    const toggle = screen.getByRole('button', { name: 'New chore' });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('button', { name: 'Add' })).not.toBeInTheDocument();

    await user.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const form = screen.getByRole('button', { name: 'Add' }).closest('form')!;
    expect(within(form).getByLabelText('Name')).toBeInTheDocument();
  });

  it('starts with the new chore form open when there are no chores', () => {
    render(EditView, { view: projectView({ tasks: [] }), executors, tab: 'chores' });

    expect(screen.getByRole('button', { name: 'New chore' })).toHaveAttribute('aria-expanded', 'true');
    expect(screen.queryByText('No matching chores.')).not.toBeInTheDocument();
  });
});
