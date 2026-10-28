import { describe, expect, it } from 'vitest';
import { pageFromRouteId, pageHref, pages } from './route';

describe('pageFromRouteId', () => {
  it('opens the task list for a missing or unknown route', () => {
    expect(pageFromRouteId(null)).toBe('tasks');
    expect(pageFromRouteId('/[...path]')).toBe('tasks');
    expect(pageFromRouteId('/manage')).toBe('tasks');
  });

  it('round-trips every page through its href', () => {
    for (const page of pages) expect(pageFromRouteId(pageHref(page))).toBe(page);
  });
});
