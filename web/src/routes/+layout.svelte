<script lang="ts">
  import { page } from '$app/state';
  import Shell from '#lib/components/Shell.svelte';
  import { pageFromRouteId } from '#lib/route.js';

  let { children } = $props();
</script>

<Shell page={pageFromRouteId(page.route.id)}>
  {@render children()}
</Shell>
