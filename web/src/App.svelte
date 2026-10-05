<script lang="ts">
  import EditView from './components/EditView.svelte';
  import MainView from './components/MainView.svelte';
  import { connect, data, refreshAll } from './lib/data.svelte';

  // The hash keeps the edit view reachable with the back button without a router.
  let hash = $state(location.hash);

  $effect(() => {
    const onHashChange = () => (hash = location.hash);
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  });

  $effect(() => {
    // The event stream also triggers a fetch once connected, but don't wait for it.
    refreshAll();
    return connect();
  });
</script>

<main>
  <header>
    <h1>{data.view?.project.name ?? 'Sysslor'}</h1>
    {#if hash === '#edit'}
      <a class="button" href="#/">Done</a>
    {:else}
      <a class="button" href="#edit">Edit</a>
    {/if}
  </header>

  {#if !data.connected || data.loadError}
    <p class="banner" role="status">
      {data.loadError ?? 'No connection to the server.'} Retrying…
    </p>
  {/if}

  {#if data.actionError}
    <p class="banner error" role="alert">
      {data.actionError}
      <button class="link" onclick={() => (data.actionError = null)}>Dismiss</button>
    </p>
  {/if}

  {#if data.view === null}
    {#if !data.loadError}
      <p class="muted">Loading…</p>
    {/if}
  {:else if hash === '#edit'}
    <EditView view={data.view} executors={data.executors} />
  {:else}
    <MainView view={data.view} executors={data.executors} />
  {/if}
</main>

<style>
  :global(:root) {
    color-scheme: light dark;
    font-family: system-ui, sans-serif;
    --bg: light-dark(#fafafa, #1a1a1a);
    --fg: light-dark(#1a1a1a, #eaeaea);
    --muted: light-dark(#666, #999);
    --surface: light-dark(#fff, #252525);
    --border: light-dark(#ddd, #3a3a3a);
    --accent: light-dark(#2563eb, #60a5fa);
    --accent-fg: light-dark(#fff, #0b1220);
    --danger: light-dark(#c62828, #ef5350);
    --warning-bg: light-dark(#fff4d6, #3d3420);
    background: var(--bg);
    color: var(--fg);
  }

  :global(body) {
    margin: 0;
  }

  :global(button),
  :global(.button),
  :global(input),
  :global(select) {
    font: inherit;
    color: inherit;
  }

  :global(button),
  :global(.button) {
    display: inline-block;
    padding: 0.4rem 0.8rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: var(--surface);
    text-decoration: none;
    cursor: pointer;
  }

  :global(button:disabled) {
    opacity: 0.5;
    cursor: default;
  }

  :global(button.primary) {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }

  :global(button.danger) {
    color: var(--danger);
  }

  :global(button.link) {
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    text-decoration: underline;
  }

  :global(input),
  :global(select) {
    padding: 0.4rem;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
    background: var(--surface);
  }

  :global(.muted) {
    color: var(--muted);
  }

  main {
    max-width: 40rem;
    margin: 0 auto;
    padding: 1rem;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  h1 {
    margin: 0.5rem 0;
  }

  .banner {
    padding: 0.5rem 0.75rem;
    border-radius: 0.5rem;
    background: var(--warning-bg);
  }

  .banner.error {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    color: var(--danger);
  }
</style>
