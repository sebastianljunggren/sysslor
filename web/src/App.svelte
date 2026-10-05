<script lang="ts">
  import EditView from './components/EditView.svelte';
  import LoginView from './components/LoginView.svelte';
  import MainView from './components/MainView.svelte';
  import { connect, data, refreshAll } from './lib/data.svelte';
  import { m } from './lib/paraglide/messages.js';

  // The hash keeps the edit view reachable with the back button without a router.
  let hash = $state(location.hash);

  $effect(() => {
    const onHashChange = () => (hash = location.hash);
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  });

  $effect(() => {
    if (!data.loggedIn) return;
    // The event stream also triggers a fetch once connected, but don't wait for it.
    refreshAll();
    return connect();
  });
</script>

<main>
  <header>
    <h1>{data.view?.project.name ?? 'Sysslor'}</h1>
    {#if data.loggedIn}
      {#if hash === '#edit'}
        <a class="button" href="#/">{m.done()}</a>
      {:else}
        <a class="button" href="#edit">{m.edit()}</a>
      {/if}
    {/if}
  </header>

  {#if !data.loggedIn}
    <LoginView />
  {:else}
    {#if !data.connected || data.loadError}
      <p class="banner" role="status">
        {data.loadError ?? m.no_connection()} {m.retrying()}
      </p>
    {/if}

    {#if data.actionError}
      <p class="banner error" role="alert">
        {data.actionError}
        <button class="link" onclick={() => (data.actionError = null)}>{m.dismiss()}</button>
      </p>
    {/if}

    {#if data.view === null}
      {#if !data.loadError}
        <p class="muted">{m.loading()}</p>
      {/if}
    {:else if hash === '#edit'}
      <EditView view={data.view} executors={data.executors} />
    {:else}
      <MainView view={data.view} executors={data.executors} />
    {/if}
  {/if}
</main>

<style>
  :global(:root) {
    color-scheme: light dark;
    font-family: system-ui, sans-serif;
    /* Solarized palette, the only color literals in the app. */
    --base03: #002b36;
    --base02: #073642;
    --base01: #586e75;
    --base00: #657b83;
    --base0: #839496;
    --base1: #93a1a1;
    --base2: #eee8d5;
    --base3: #fdf6e3;
    --yellow: #b58900;
    --orange: #cb4b16;
    --red: #dc322f;
    --magenta: #d33682;
    --violet: #6c71c4;
    --blue: #268bd2;
    --cyan: #2aa198;
    --green: #859900;

    /* Text is one step stronger than strict Solarized: muted base1 on base3 is too faint. */
    --bg: light-dark(var(--base2), var(--base03));
    --surface: light-dark(var(--base3), var(--base02));
    --fg: light-dark(var(--base01), var(--base1));
    --muted: light-dark(var(--base00), var(--base0));
    --border: light-dark(var(--base1), var(--base01));
    --accent: var(--blue);
    --accent-fg: light-dark(var(--base3), var(--base03));
    --danger: var(--red);
    --warning-bg: color-mix(in srgb, var(--yellow) 20%, var(--bg));
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
