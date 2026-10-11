import { afterEach, describe, expect, it, vi } from 'vitest';
import { clock, startClock } from './clock.svelte';

describe('clock', () => {
  let stop: () => void = () => {};
  afterEach(() => stop());

  it('ticks while the app runs', () => {
    vi.useFakeTimers({ now: new Date(2026, 9, 5, 12, 0) });
    stop = startClock();

    vi.advanceTimersByTime(30 * 1000);

    expect(clock.now).toBe(new Date(2026, 9, 5, 12, 0, 30).getTime());
  });

  it('catches up when the page is shown again', () => {
    stop = startClock();
    vi.useFakeTimers({ now: new Date(2026, 9, 5, 18, 0), toFake: ['Date'] });

    document.dispatchEvent(new Event('visibilitychange'));

    expect(clock.now).toBe(new Date(2026, 9, 5, 18, 0).getTime());
  });
});
