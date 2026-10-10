import '@testing-library/jest-dom/vitest';
import { afterEach, vi } from 'vitest';

// Before any app module loads: `format.ts` caches `Intl` formatters for the locale at import.
localStorage.setItem('PARAGLIDE_LOCALE', 'en');

afterEach(() => {
  // Before touching storage, which a test may have mocked to throw.
  vi.restoreAllMocks();
  vi.useRealTimers();
  localStorage.clear();
  localStorage.setItem('PARAGLIDE_LOCALE', 'en');
});

// jsdom has no Web Animations API, which `animate:flip` and transitions use.
Element.prototype.getAnimations ??= () => [];
Element.prototype.animate ??= function () {
  const animation = { cancel() {}, finish() {}, onfinish: null as (() => void) | null };
  queueMicrotask(() => animation.onfinish?.());
  return animation as unknown as Animation;
};

// jsdom has no modal dialogs.
HTMLDialogElement.prototype.showModal ??= function (this: HTMLDialogElement) {
  this.open = true;
};
HTMLDialogElement.prototype.close ??= function (this: HTMLDialogElement) {
  if (!this.open) return;
  this.open = false;
  this.dispatchEvent(new Event('close'));
};
