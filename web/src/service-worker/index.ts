import { version } from '$app/env';
import { assets as staticFiles, immutable } from '$app/manifest';
import { resolve } from '$app/paths';
import { route } from '#lib/sw-route.js';

const sw = self as unknown as ServiceWorkerGlobalScope;

// The app shell is tied to a build. The data isn't, so it survives a deploy.
const APP_CACHE = `app-${version}`;
const API_CACHE = 'api';
// The Rust backend serves the same index.html for every client route.
const INDEX = resolve('/');
// When the hostname resolves but nothing answers (VPN down, weak signal), fetch would wait
// for the OS connect timeout, often minutes, before the cached copy is shown.
const PAGE_TIMEOUT_MS = 3000;
const API_TIMEOUT_MS = 5000;

const assets = new Set([...immutable, ...staticFiles].map(({ path }) => resolve(path)));

sw.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(APP_CACHE);
      await cache.addAll([...assets, INDEX]);
      await sw.skipWaiting();
    })(),
  );
});

sw.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      for (const key of await caches.keys()) {
        if (key.startsWith('app-') && key !== APP_CACHE) await caches.delete(key);
      }
      await sw.clients.claim();
    })(),
  );
});

sw.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);
  switch (route({ url, method: request.method, mode: request.mode }, sw.location.origin, assets)) {
    case 'passthrough':
      return;
    case 'asset':
      return event.respondWith(fromCacheFirst(request));
    case 'page':
      return event.respondWith(page(request));
    case 'api-data':
      return event.respondWith(apiData(request));
    case 'logout':
      return event.respondWith(logout(request));
  }
});

async function fromCacheFirst(request: Request): Promise<Response> {
  const cached = await caches.match(request, { cacheName: APP_CACHE });
  return cached ?? fetch(request);
}

/** Rejects if the response headers don't arrive in time. A slow body is not cut off. */
async function fetchWithTimeout(request: Request, ms: number): Promise<Response> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), ms);
  try {
    return await fetch(request, { signal: controller.signal });
  } finally {
    clearTimeout(timer);
  }
}

async function page(request: Request): Promise<Response> {
  try {
    return await fetchWithTimeout(request, PAGE_TIMEOUT_MS);
  } catch (error) {
    const cached = await caches.match(INDEX, { cacheName: APP_CACHE });
    if (cached) return cached;
    throw error;
  }
}

/** The last fetched list is shown offline. Completions are queued by the page (`queue.svelte.ts`). */
async function apiData(request: Request): Promise<Response> {
  let response: Response;
  try {
    response = await fetchWithTimeout(request, API_TIMEOUT_MS);
  } catch {
    const cached = await caches.match(request, { cacheName: API_CACHE });
    // An error makes fetch() reject in the page, as if there were no service worker.
    return cached ?? Response.error();
  }
  if (response.ok) {
    const cache = await caches.open(API_CACHE);
    await cache.put(request, response.clone());
  } else if (response.status === 401) {
    // Logged out, e.g. on another device or after a key change: don't keep the data around.
    await caches.delete(API_CACHE);
  }
  return response;
}

async function logout(request: Request): Promise<Response> {
  const response = await fetch(request);
  if (response.ok) await caches.delete(API_CACHE);
  return response;
}
