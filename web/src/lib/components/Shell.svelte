<script lang="ts">
  import type { Snippet } from 'svelte';
  import { afterNavigate } from '$app/navigation';
  import { connect, data, refreshAll } from '#lib/data.svelte.js';
  import { m } from '#lib/paraglide/messages.js';
  import { pageLabel, type Page } from '#lib/route.js';
  import LoginView from './LoginView.svelte';
  import Nav from './Nav.svelte';

  let { page, children }: { page: Page; children: Snippet } = $props();
  const projectName = $derived(data.view?.project.name ?? 'Sysslor');

  let drawer = $state<HTMLDialogElement>();
  let drawerOpen = $state(false);

  function openDrawer() {
    drawer?.showModal();
    drawerOpen = true;
  }

  function closeDrawer() {
    drawer?.close();
  }

  // Clicks on the backdrop target the dialog itself; clicks on its content target children.
  function closeOnBackdrop(event: MouseEvent) {
    if (event.target === drawer) closeDrawer();
  }

  // The back button doesn't go through the menu's links.
  afterNavigate(closeDrawer);

  // An open modal keeps the page inert, also after the window grows past the breakpoint
  // where the menu button disappears.
  $effect(() => {
    const wide = window.matchMedia?.('(min-width: 52rem)');
    if (!wide) return;
    const onChange = () => wide.matches && closeDrawer();
    wide.addEventListener('change', onChange);
    return () => wide.removeEventListener('change', onChange);
  });

  $effect(() => {
    if (!data.loggedIn) return;
    // The event stream also triggers a fetch once connected, but don't wait for it.
    refreshAll();
    return connect();
  });
</script>

{#if !data.loggedIn}
  <main>
    <header>
      <h1>{projectName}</h1>
    </header>
    <LoginView />
  </main>
{:else}
  <div class="layout">
    <aside class="sidebar">
      <p class="project">{projectName}</p>
      <Nav {page} />
    </aside>

    <dialog
      class="drawer"
      id="menu"
      aria-label={m.menu()}
      bind:this={drawer}
      onclose={() => (drawerOpen = false)}
      onclick={closeOnBackdrop}
    >
      <div class="drawer-header">
        <p class="project">{projectName}</p>
        <button class="icon-button" aria-label={m.close_menu()} onclick={closeDrawer}>
          <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
            <path d="M6 6l12 12M18 6L6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
      </div>
      <Nav {page} onnavigate={closeDrawer} />
    </dialog>

    <main>
      <header>
        <button class="icon-button menu-button" aria-label={m.menu()} aria-controls="menu" aria-expanded={drawerOpen} onclick={openDrawer}>
          <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
            <path d="M4 6h16M4 12h16M4 18h16" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
        <h1>{page === 'tasks' ? projectName : pageLabel(page)}</h1>
      </header>

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
      {:else}
        {@render children()}
      {/if}
    </main>
  </div>
{/if}

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

  /* The native arrow sits flush against the right border whatever the padding, so draw a
     chevron in the text color instead. */
  :global(select) {
    appearance: none;
    padding-right: 2rem;
    background-image:
      linear-gradient(
        to top right,
        transparent calc(50% - 0.75px),
        currentColor calc(50% - 0.75px) calc(50% + 0.75px),
        transparent calc(50% + 0.75px)
      ),
      linear-gradient(
        to top left,
        transparent calc(50% - 0.75px),
        currentColor calc(50% - 0.75px) calc(50% + 0.75px),
        transparent calc(50% + 0.75px)
      );
    background-position:
      right 1.05rem center,
      right 0.75rem center;
    background-size: 0.3rem 0.3rem;
    background-repeat: no-repeat;
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
    gap: 0.75rem;
  }

  .project {
    margin: 0 0 0.5rem;
    padding: 0 calc(0.75rem + 2px);
    font-weight: bold;
    overflow-wrap: anywhere;
  }

  .sidebar {
    display: none;
  }

  .icon-button {
    display: flex;
    padding: 0.4rem;
    border: none;
    background: none;
  }

  .menu-button {
    margin-left: -0.4rem;
  }

  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .drawer-header .project {
    margin: 0;
  }

  .drawer {
    width: min(16rem, 80vw);
    height: 100dvh;
    max-height: none;
    margin: 0;
    padding: 1rem 0.5rem;
    border: none;
    border-right: 1px solid var(--border);
    background: var(--surface);
    color: var(--fg);
    transform: translateX(-100%);
    transition:
      transform 0.2s ease-out,
      overlay 0.2s allow-discrete,
      display 0.2s allow-discrete;
  }

  .drawer[open] {
    transform: none;
  }

  @starting-style {
    .drawer[open] {
      transform: translateX(-100%);
    }
  }

  .drawer::backdrop {
    background: color-mix(in srgb, var(--base03) 40%, transparent);
  }

  @media (prefers-reduced-motion: reduce) {
    .drawer {
      transition: none;
    }
  }

  /* Room for the menu next to a full-width page. */
  @media (min-width: 52rem) {
    .layout {
      display: grid;
      grid-template-columns: 13rem 1fr;
      min-height: 100dvh;
    }

    .sidebar {
      display: block;
      position: sticky;
      top: 0;
      height: 100dvh;
      box-sizing: border-box;
      padding: 1.5rem 0.5rem;
      border-right: 1px solid var(--border);
      background: var(--surface);
      overflow-y: auto;
    }

    .menu-button {
      display: none;
    }

    main {
      width: 100%;
      box-sizing: border-box;
    }
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
