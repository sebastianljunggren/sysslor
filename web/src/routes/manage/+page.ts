import { redirect } from '@sveltejs/kit';
import { pageHref } from '#lib/route.js';

export function load() {
  redirect(307, pageHref('chores'));
}
