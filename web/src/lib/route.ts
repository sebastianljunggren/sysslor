import { m } from './paraglide/messages.js';

// Each page is a SvelteKit route; its href is also its route id.
export const pages = ['tasks', 'chores', 'groups', 'people', 'settings'] as const;
export type Page = (typeof pages)[number];

export function pageHref(page: Page): string {
  return page === 'tasks' ? '/' : `/manage/${page}`;
}

/** The id is `null` while an error page shows; that and any other route count as the task list. */
export function pageFromRouteId(id: string | null): Page {
  return pages.find((p) => pageHref(p) === id) ?? 'tasks';
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
