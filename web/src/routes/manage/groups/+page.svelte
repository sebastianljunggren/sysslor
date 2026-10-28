<script lang="ts">
  import ManageView from '#lib/components/ManageView.svelte';
  import { data } from '#lib/data.svelte.js';
</script>

{#if data.view}
  <ManageView view={data.view} executors={data.executors} tab="groups" />
{/if}
