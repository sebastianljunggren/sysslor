import type { Cadence } from './api/Cadence';

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

export function formatCadence({ amount, unit }: Cadence): string {
  const noun = unit === 'days' ? 'day' : 'week';
  return amount === 1 ? `Every ${noun}` : `Every ${amount} ${noun}s`;
}

/**
 * The server computes `due` in the family time zone; comparing it with the device's
 * local date assumes the device is in that zone, which holds for a family app.
 */
export function formatDue(due: string | null, now = new Date()): string {
  if (due === null) return 'Never done';
  const days = daysBetween(localDate(now), due);
  if (days < 0) return `Overdue ${-days} ${days === -1 ? 'day' : 'days'}`;
  if (days === 0) return 'Today';
  if (days === 1) return 'Tomorrow';
  return `In ${days} days`;
}

const timeFormat = new Intl.DateTimeFormat('en-US', { hour: 'numeric', minute: '2-digit' });
const dateFormat = new Intl.DateTimeFormat('en-US', { weekday: 'short', day: 'numeric', month: 'short' });

/** "today 6:30 PM", "yesterday 7:15 AM", "Sat, Oct 3 12:00 PM" */
export function formatInstant(ms: number, now = new Date()): string {
  const date = new Date(ms);
  const days = daysBetween(localDate(date), localDate(now));
  const day = days === 0 ? 'today' : days === 1 ? 'yesterday' : dateFormat.format(date);
  return `${day} ${timeFormat.format(date)}`;
}
