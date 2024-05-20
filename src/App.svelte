<script lang="ts">
  import Greet from "./lib/Greet.svelte";
  import ServerConnectionStatus from "./lib/ServerConnectionStatus.svelte";
  import GameConnectionStatus from "./lib/GameConnectionStatus.svelte";
</script>

<main class="container">
  <h1>Crowd Control Celeste Bridge</h1>

  <div class="controls">
    <ServerConnectionStatus />
    <GameConnectionStatus />
  </div>
</main>

<style>
  h1 {
    font-size: 3rem;
    line-height: 3rem;
    font-family: Audiowide;
    text-transform: uppercase;
    transform: skew(-10deg, 0deg);
    color: var(--color-moonshot-core-yellow);
    min-width: 33rem;
  }

  .controls {
    width: 80%;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-left: auto;
    margin-right: auto;
  }
</style>
