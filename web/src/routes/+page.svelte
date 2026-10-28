<script lang="ts">
  import MainView from '#lib/components/MainView.svelte';
  import { data } from '#lib/data.svelte.js';
</script>

{#if data.view}
  <MainView view={data.view} executors={data.executors} />
{/if}
