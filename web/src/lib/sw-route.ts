/** How the service worker handles a request. */
export type Route =
  /** Not intercepted: the browser talks to the network as if there were no service worker. */
  | 'passthrough'
  /** A file from the build, served from the cache. */
  | 'asset'
  /** A page load, served from the network, or from the cached index.html when offline. */
  | 'page'
  /** Data shown in the list, from the network, or from the last response when offline. */
  | 'api-data'
  /** Logging out, which also drops the cached data. */
  | 'logout';

export interface RouteRequest {
  url: URL;
  method: string;
  mode: RequestMode;
}

const API_DATA = [/^\/api\/projects\/\d+$/, /^\/api\/executors$/];

/**
 * Decides how the service worker handles a request. `origin` is the app's own origin and
 * `assets` the paths of the cached build files.
 */
export function route({ url, method, mode }: RouteRequest, origin: string, assets: Set<string>): Route {
  if (url.origin !== origin) return 'passthrough';
  if (method === 'POST' && url.pathname === '/api/logout') return 'logout';
  if (method !== 'GET') return 'passthrough';
  if (API_DATA.some((pattern) => pattern.test(url.pathname))) return 'api-data';
  // Everything else under /api, including the event stream, must reach the server: a
  // cached answer there would be wrong rather than stale.
  if (url.pathname.startsWith('/api/') || url.pathname === '/healthz') return 'passthrough';
  if (assets.has(url.pathname)) return 'asset';
  if (mode === 'navigate') return 'page';
  // E.g. _app/version.json, which SvelteKit polls to notice a new deploy.
  return 'passthrough';
}
