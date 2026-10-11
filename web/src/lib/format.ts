import type { Cadence } from './api/Cadence';
import type { GroupColor } from './api/GroupColor';
import { m } from './paraglide/messages.js';
import { getLocale } from './paraglide/runtime.js';

const MINUTE_MS = 60 * 1000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;

/** `YYYY-MM-DD` for the device's local calendar day. */
export function localDate(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** Value for `<input type="datetime-local">`, in device local time. */
export function localDateTime(ms: number): string {
  const date = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${localDate(date)}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function daysBetween(from: string, to: string): number {
  return Math.round((Date.parse(to) - Date.parse(from)) / DAY_MS);
}

/** In palette order, which is also the order they are offered in. */
export const groupColors: GroupColor[] = ['yellow', 'orange', 'red', 'magenta', 'violet', 'blue', 'cyan', 'green'];

const colorLabels: Record<GroupColor, () => string> = {
  yellow: m.color_yellow,
  orange: m.color_orange,
  red: m.color_red,
  magenta: m.color_magenta,
  violet: m.color_violet,
  blue: m.color_blue,
  cyan: m.color_cyan,
  green: m.color_green,
};

export function formatColor(color: GroupColor): string {
  return colorLabels[color]();
}

export function formatCadence({ amount, unit }: Cadence): string {
  return unit === 'days' ? m.cadence_days({ count: amount }) : m.cadence_weeks({ count: amount });
}

// Changing the locale reloads the page, so formatters for the current one can be cached.
// `numeric: 'auto'` gives "today", "tomorrow" and "yesterday" instead of "in 0 days".
const relativeFormat = new Intl.RelativeTimeFormat(getLocale(), { numeric: 'auto' });
const dueFormat = new Intl.DateTimeFormat(getLocale(), { dateStyle: 'full' });
const instantFormat = new Intl.DateTimeFormat(getLocale(), { dateStyle: 'full', timeStyle: 'short' });

/** Days up to two weeks, then rounded weeks, which is precise enough at a glance. */
function relativeDays(days: number): string {
  if (Math.abs(days) < 14) return relativeFormat.format(days, 'day');
  return relativeFormat.format(Math.round(days / 7), 'week');
}

/**
 * "Due tomorrow", "Was due 2 days ago" (in English).
 *
 * The server computes `due` in the configured time zone; comparing it with the device's
 * local date assumes the device is in that zone, which holds for a family app.
 */
export function formatDue(due: string | null, now = new Date()): string {
  if (due === null) return m.never_done();
  const days = daysBetween(localDate(now), due);
  const when = relativeDays(days);
  return days < 0 ? m.was_due({ when }) : m.due_relative({ when });
}

/** "Tuesday, October 6, 2026" (in English), for hovering a due date. */
export function formatDueAbsolute(due: string): string {
  const [year, month, day] = due.split('-').map(Number);
  // Not `Date.parse`, which reads a bare date as UTC midnight: the previous day west of UTC.
  return dueFormat.format(new Date(year, month - 1, day));
}

/** "just now", "4 hours ago", "yesterday", "3 days ago" (in English). */
export function formatInstant(ms: number, now = new Date()): string {
  const elapsed = Math.max(now.getTime() - ms, 0);
  if (elapsed < MINUTE_MS) return m.just_now();
  if (elapsed < HOUR_MS) return relativeFormat.format(-Math.floor(elapsed / MINUTE_MS), 'minute');
  if (elapsed < DAY_MS) return relativeFormat.format(-Math.floor(elapsed / HOUR_MS), 'hour');
  return relativeDays(-daysBetween(localDate(new Date(ms)), localDate(now)));
}

/** "Monday, October 5, 2026 at 7:30 AM" (in English), for hovering an instant. */
export function formatInstantAbsolute(ms: number): string {
  return instantFormat.format(ms);
}
