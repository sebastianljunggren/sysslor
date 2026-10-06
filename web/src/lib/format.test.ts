import { describe, expect, it } from 'vitest';
import { formatCadence, formatDue, formatInstant, localDate, localDateTime } from './format';

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
    ['2026-10-03', 'Overdue 2 days'],
    ['2026-10-04', 'Overdue 1 day'],
    ['2026-10-05', 'Today'],
    ['2026-10-06', 'Tomorrow'],
    ['2026-10-12', 'In 7 days'],
  ])('formats %s', (due, expected) => {
    expect(formatDue(due, now)).toBe(expected);
  });

  it('counts calendar days late in the evening', () => {
    expect(formatDue('2026-10-06', new Date(2026, 9, 5, 23, 59))).toBe('Tomorrow');
  });

  it('counts calendar days across the end of DST', () => {
    // Europe/Stockholm leaves DST on 2026-10-25, so that day has 25 hours.
    expect(formatDue('2026-10-26', new Date(2026, 9, 24, 23, 30))).toBe('In 2 days');
  });
});

describe('formatInstant', () => {
  const now = new Date(2026, 9, 5, 0, 30);

  it('says today for the same calendar day', () => {
    expect(formatInstant(new Date(2026, 9, 5, 0, 5).getTime(), now)).toMatch(/^today 12:05\sAM$/);
  });

  it('says yesterday for the previous calendar day, even minutes ago', () => {
    expect(formatInstant(new Date(2026, 9, 4, 23, 55).getTime(), now)).toMatch(/^yesterday 11:55\sPM$/);
  });

  it('shows the date for older instants', () => {
    expect(formatInstant(new Date(2026, 9, 3, 12, 0).getTime(), now)).toMatch(/^Sat, Oct 3 12:00\sPM$/);
  });
});
