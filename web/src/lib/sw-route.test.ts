import { describe, expect, it } from 'vitest';
import { route } from './sw-route';

const ORIGIN = 'https://sysslor.example';
const ASSETS = new Set(['/_app/immutable/entry/start.abc123.js', '/icon.svg']);

function routeOf(path: string, { method = 'GET', mode = 'cors' as RequestMode } = {}) {
  return route({ url: new URL(path, ORIGIN), method, mode }, ORIGIN, ASSETS);
}

describe('route', () => {
  it('caches the data the list is built from', () => {
    expect(routeOf('/api/projects/1')).toBe('api-data');
    expect(routeOf('/api/executors')).toBe('api-data');
  });

  it('never intercepts the event stream', () => {
    expect(routeOf('/api/events')).toBe('passthrough');
  });

  it('passes writes through', () => {
    expect(routeOf('/api/completions/0190-abc', { method: 'PUT' })).toBe('passthrough');
    expect(routeOf('/api/completions/0190-abc', { method: 'DELETE' })).toBe('passthrough');
    expect(routeOf('/api/executors', { method: 'POST' })).toBe('passthrough');
    expect(routeOf('/api/login', { method: 'POST' })).toBe('passthrough');
  });

  it('handles logging out', () => {
    expect(routeOf('/api/logout', { method: 'POST' })).toBe('logout');
  });

  it('passes other API paths and the health check through', () => {
    expect(routeOf('/api/projects/1/tasks')).toBe('passthrough');
    expect(routeOf('/api/nope')).toBe('passthrough');
    expect(routeOf('/healthz')).toBe('passthrough');
  });

  it('serves build files from the cache', () => {
    expect(routeOf('/_app/immutable/entry/start.abc123.js')).toBe('asset');
    expect(routeOf('/icon.svg')).toBe('asset');
  });

  it('handles page loads on any client route', () => {
    expect(routeOf('/', { mode: 'navigate' })).toBe('page');
    expect(routeOf('/manage/people', { mode: 'navigate' })).toBe('page');
  });

  it('passes the version check through', () => {
    expect(routeOf('/_app/version.json')).toBe('passthrough');
  });

  it('passes other origins through', () => {
    const url = new URL('https://elsewhere.example/api/projects/1');
    expect(route({ url, method: 'GET', mode: 'cors' }, ORIGIN, ASSETS)).toBe('passthrough');
  });
});
