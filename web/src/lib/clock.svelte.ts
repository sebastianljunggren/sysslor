const TICK_MS = 30 * 1000;

/** The current time, updated often enough that relative times ("4 minutes ago") stay right. */
export const clock = $state({ now: Date.now() });

function tick() {
  clock.now = Date.now();
}

export function startClock(): () => void {
  const interval = setInterval(tick, TICK_MS);
  // Timers are throttled or paused in background tabs, so catch up as soon as it is shown.
  const onVisibilityChange = () => {
    if (document.visibilityState === 'visible') tick();
  };
  document.addEventListener('visibilitychange', onVisibilityChange);
  return () => {
    clearInterval(interval);
    document.removeEventListener('visibilitychange', onVisibilityChange);
  };
}
