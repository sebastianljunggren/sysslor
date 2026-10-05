<script lang="ts">
  import { logIn } from '../lib/data.svelte';

  let password = $state('');
  let error = $state<string | null>(null);
  let pending = $state(false);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    pending = true;
    error = await logIn(password);
    pending = false;
    if (error === null) password = '';
  }
</script>

<form onsubmit={submit}>
  <!-- Lets password managers tell this login apart from others on the same host. -->
  <input type="text" name="username" autocomplete="username" value="sysslor" hidden />
  <label>
    Password
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="password"
      name="password"
      autocomplete="current-password"
      bind:value={password}
      required
      autofocus
    />
  </label>
  <button type="submit" class="primary" disabled={pending}>Log in</button>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</form>

<style>
  form {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  label {
    display: flex;
    flex: 1 1 12rem;
    flex-direction: column;
    gap: 0.2rem;
    color: var(--muted);
    font-size: 0.9rem;
  }

  label input {
    color: var(--fg);
  }

  .error {
    flex-basis: 100%;
    margin: 0;
    color: var(--danger);
  }
</style>
