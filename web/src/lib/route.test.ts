import { describe, expect, it } from 'vitest';
import { pageFromHash, pageHref, pages } from './route';

describe('pageFromHash', () => {
  it('opens the task list for an empty or unknown hash', () => {
    expect(pageFromHash('')).toBe('tasks');
    expect(pageFromHash('#/')).toBe('tasks');
    expect(pageFromHash('#people')).toBe('tasks');
    expect(pageFromHash('#managed')).toBe('tasks');
  });

  it('opens chores for the bare manage hash or an unknown manage page', () => {
    expect(pageFromHash('#manage')).toBe('chores');
    expect(pageFromHash('#manage/')).toBe('chores');
    expect(pageFromHash('#manage/nope')).toBe('chores');
    expect(pageFromHash('#manage/tasks')).toBe('chores');
  });

  it('round-trips every page through its href', () => {
    for (const page of pages) expect(pageFromHash(pageHref(page))).toBe(page);
  });
});
