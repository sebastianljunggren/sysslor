import { describe, expect, it } from 'vitest';
import {
  formatCadence,
  formatDue,
  formatDueAbsolute,
  formatInstant,
  formatInstantAbsolute,
  localDate,
  localDateTime,
} from './format';

describe('localDate', () => {
  it('pads month and day', () => {
    expect(localDate(new Date(2026, 0, 5, 23, 59))).toBe('2026-01-05');
  });
});

describe('localDateTime', () => {
  it('formats in device local time', () => {
    expect(localDateTime(new Date(2026, 2, 7, 8, 5).getTime())).toBe('2026-03-07T08:05');
  });
});

describe('formatCadence', () => {
  it('uses singular and plural forms', () => {
    expect(formatCadence({ amount: 1, unit: 'days' })).toBe('Every day');
    expect(formatCadence({ amount: 3, unit: 'days' })).toBe('Every 3 days');
    expect(formatCadence({ amount: 1, unit: 'weeks' })).toBe('Every week');
    expect(formatCadence({ amount: 2, unit: 'weeks' })).toBe('Every 2 weeks');
  });
});

describe('formatDue', () => {
  const now = new Date(2026, 9, 5, 12, 0);

  it('says never done without a due date', () => {
    expect(formatDue(null, now)).toBe('Never done');
  });

  it.each([
    ['2026-09-21', 'Was due 2 weeks ago'],
    ['2026-10-03', 'Was due 2 days ago'],
    ['2026-10-04', 'Was due yesterday'],
    ['2026-10-05', 'Due today'],
    ['2026-10-06', 'Due tomorrow'],
    ['2026-10-12', 'Due in 7 days'],
    ['2026-10-18', 'Due in 13 days'],
    ['2026-10-26', 'Due in 3 weeks'],
  ])('formats %s', (due, expected) => {
    expect(formatDue(due, now)).toBe(expected);
  });

  it('counts calendar days late in the evening', () => {
    expect(formatDue('2026-10-06', new Date(2026, 9, 5, 23, 59))).toBe('Due tomorrow');
  });

  it('counts calendar days across the end of DST', () => {
    // Europe/Stockholm leaves DST on 2026-10-25, so that day has 25 hours.
    expect(formatDue('2026-10-26', new Date(2026, 9, 24, 23, 30))).toBe('Due in 2 days');
  });
});

describe('formatDueAbsolute', () => {
  it('shows the calendar day itself, not the one before', () => {
    expect(formatDueAbsolute('2026-10-06')).toBe('Tuesday, October 6, 2026');
  });
});

describe('formatInstant', () => {
  const now = new Date(2026, 9, 5, 0, 30);
  const ago = (minutes: number) => now.getTime() - minutes * 60 * 1000;

  it.each([
    [0.5, 'just now'],
    [5, '5 minutes ago'],
    [59, '59 minutes ago'],
    [60, '1 hour ago'],
    [4 * 60 + 59, '4 hours ago'],
  ])('formats %s minutes ago', (minutes, expected) => {
    expect(formatInstant(ago(minutes), now)).toBe(expected);
  });

  it('counts hours across midnight within a day', () => {
    expect(formatInstant(new Date(2026, 9, 4, 18, 30).getTime(), now)).toBe('6 hours ago');
  });

  it.each([
    [new Date(2026, 9, 4, 0, 0), 'yesterday'],
    [new Date(2026, 9, 3, 23, 0), '2 days ago'],
    [new Date(2026, 9, 2, 12, 0), '3 days ago'],
    [new Date(2026, 8, 15, 12, 0), '3 weeks ago'],
  ])('counts calendar days after a day', (date, expected) => {
    expect(formatInstant(date.getTime(), now)).toBe(expected);
  });

  it('treats an instant in the future, from clock skew, as just now', () => {
    expect(formatInstant(now.getTime() + 60 * 1000, now)).toBe('just now');
  });
});

describe('formatInstantAbsolute', () => {
  it('shows the date and time', () => {
    expect(formatInstantAbsolute(new Date(2026, 9, 5, 7, 30).getTime())).toMatch(
      /^Monday, October 5, 2026 at 7:30\sAM$/,
    );
  });
});
