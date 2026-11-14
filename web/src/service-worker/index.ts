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

async function page(request: Request): Promise<Response> {
  try {
    return await fetch(request);
  } catch (error) {
    const cached = await caches.match(INDEX, { cacheName: APP_CACHE });
    if (cached) return cached;
    throw error;
  }
}

/** The last fetched list is shown offline. Writes still need a connection. */
async function apiData(request: Request): Promise<Response> {
  let response: Response;
  try {
    response = await fetch(request);
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
