import { redirect } from '@sveltejs/kit';
import { pageHref } from '#lib/route.js';

// Unknown paths open the task list, so a mistyped or outdated link still lands somewhere.
export function load() {
  redirect(307, pageHref('tasks'));
}
