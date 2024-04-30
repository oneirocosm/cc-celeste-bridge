<script lang="ts">
  import Greet from "./lib/Greet.svelte";
  import ServerConnectionStatus from "./lib/ServerConnectionStatus.svelte";
  import GameConnectionStatus from "./lib/GameConnectionStatus.svelte";
</script>

<main class="container">
  <h1>Welcome to Tauri!</h1>

  <ServerConnectionStatus />
  <GameConnectionStatus />
</main>

<style>
</style>
