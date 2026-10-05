import type { Cadence } from './api/Cadence';
import type { GroupColor } from './api/GroupColor';
import { m } from './paraglide/messages.js';
import { getLocale } from './paraglide/runtime.js';

const DAY_MS = 24 * 60 * 60 * 1000;

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

/**
 * The server computes `due` in the configured time zone; comparing it with the device's
 * local date assumes the device is in that zone, which holds for a family app.
 */
export function formatDue(due: string | null, now = new Date()): string {
  if (due === null) return m.never_done();
  const days = daysBetween(localDate(now), due);
  if (days < 0) return m.overdue_days({ days: -days });
  if (days === 0) return m.due_today();
  if (days === 1) return m.due_tomorrow();
  return m.due_in_days({ days });
}

// Changing the locale reloads the page, so formatters for the current one can be cached.
const timeFormat = new Intl.DateTimeFormat(getLocale(), { hour: 'numeric', minute: '2-digit' });
const dateFormat = new Intl.DateTimeFormat(getLocale(), { weekday: 'short', day: 'numeric', month: 'short' });

/** "today 6:30 PM", "yesterday 7:15 AM", "Sat, Oct 3 12:00 PM" (in English) */
export function formatInstant(ms: number, now = new Date()): string {
  const date = new Date(ms);
  const days = daysBetween(localDate(date), localDate(now));
  const time = timeFormat.format(date);
  if (days === 0) return m.instant_today({ time });
  if (days === 1) return m.instant_yesterday({ time });
  return m.instant_date({ date: dateFormat.format(date), time });
}
