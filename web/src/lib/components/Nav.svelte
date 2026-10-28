<script lang="ts">
  import { m } from '#lib/paraglide/messages.js';
  import { pageHref, pageLabel, type Page } from '#lib/route.js';

  let { page, onnavigate }: { page: Page; onnavigate?: () => void } = $props();

  const sections: { title: string | null; pages: Page[] }[] = [
    { title: null, pages: ['tasks'] },
    { title: m.manage(), pages: ['chores', 'groups', 'people', 'settings'] },
  ];
</script>

<nav aria-label={m.menu()}>
  {#each sections as section, i (i)}
    {#if section.title !== null}
      <h2>{section.title}</h2>
    {/if}
    <ul>
      {#each section.pages as p (p)}
        <li>
          <a href={pageHref(p)} aria-current={p === page ? 'page' : undefined} onclick={() => onnavigate?.()}>
            {pageLabel(p)}
          </a>
        </li>
      {/each}
    </ul>
  {/each}
</nav>

<style>
  h2 {
    margin: 1.25rem 0 0.25rem;
    padding-left: calc(0.75rem + 2px);
    font-size: 0.75rem;
    font-weight: normal;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--muted);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  a {
    display: block;
    padding: 0.5rem 0.75rem;
    border-left: 2px solid transparent;
    color: var(--muted);
    text-decoration: none;
  }

  a:hover {
    color: var(--fg);
  }

  a[aria-current='page'] {
    border-left-color: var(--accent);
    color: var(--fg);
  }
</style>
