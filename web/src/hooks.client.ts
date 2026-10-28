import { startClock } from '#lib/clock.svelte.js';
import { getLocale } from '#lib/paraglide/runtime.js';

export function init() {
  document.documentElement.lang = getLocale();
  startClock();
}
