import { m } from './paraglide/messages.js';

// The hash is the route, so pages are reachable with the back button without a router.
// The manage hashes predate the menu and stay as they are, since links point at them.
export const pages = ['tasks', 'chores', 'groups', 'people', 'settings'] as const;
export type Page = (typeof pages)[number];

/** Unknown hashes open the task list; `#manage` alone or an unknown manage page opens chores. */
export function pageFromHash(hash: string): Page {
  if (hash !== '#manage' && !hash.startsWith('#manage/')) return 'tasks';
  const page = hash.slice('#manage/'.length);
  return pages.find((p) => p !== 'tasks' && p === page) ?? 'chores';
}

export function pageHref(page: Page): string {
  if (page === 'tasks') return '#/';
  if (page === 'chores') return '#manage';
  return `#manage/${page}`;
}

const labels: Record<Page, () => string> = {
  tasks: m.to_do,
  chores: m.chores,
  groups: m.groups,
  people: m.people,
  settings: m.settings,
};

export function pageLabel(page: Page): string {
  return labels[page]();
}
