<script lang="ts">
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/tauri";

    let errorMsg = "";
    let connectState = "disconnected";

    async function connectGame() {
        errorMsg = "";
        connectState = "connecting";
        await invoke("tcp_connect", { errorMsg })
            .then(() => {
                connectState = "disconnected";
            })
            .catch((e) => {
                connectState = "disconnected";
                errorMsg = e;
            });
    }

    async function disconnectGame() {
        connectState = "connecting";
        await invoke("tcp_disconnect", { errorMsg })
            .then(() => {
                connectState = "disconnected";
            })
            .catch((e) => {
                connectState = "disconnected";
                errorMsg = e;
            });
    }

    async function connResp() {
        await listen("tcp_conn", (event) => {
            console.log(event);
            connectState = event.payload as string;
        });
    }

    connResp();
</script>

<div
    class={connectState === "connected"
        ? "outer-content-box connect"
        : "outer-content-box noconnect"}
>
    <div class="inner-content-row">
        <h2>Game:</h2>
        <p></p>
    </div>
    <div class="inner-content-row user-input">
        {#if connectState === "disconnected"}
            <button on:click={connectGame}> Connect to Game </button>
        {:else if connectState === "connected"}
            <button on:click={disconnectGame}> Disconnect Game </button>
        {:else if connectState === "connecting"}
            <button on:click={disconnectGame}> Cancel Connection </button>
        {:else}
            <button on:click={disconnectGame} disabled>
                Disconnecting ...
            </button>
        {/if}
    </div>
</div>

<style>
    h2 {
        font-family: Audiowide;
        text-transform: uppercase;
        transform: skew(-10deg, 0deg);
        color: var(--color-moonshot-core-yellow);
    }

    .inner-content-row {
        display: flex;
        flex-direction: row;
        gap: 1rem;

        &.user-input {
            justify-content: end;
        }
    }

    .outer-content-box {
        border: 0.3rem solid var(--color-moonshot-core-pink);
        padding: 1rem;
        border-radius: 0.5rem;
        min-width: 22rem;
    }

    .connect {
        background-color: var(--color-moonshot-extra-blue-dark);
    }

    .noconnect {
        background-color: var(--color-moonshot-extra-burgundy);
    }
</style>
