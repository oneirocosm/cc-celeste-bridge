<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { listen } from "@tauri-apps/api/event";
    import parse from "url-parse";

    let playerScreenUrl = "";
    let errorMsg = "";
    let connectState = "disconnected";

    async function connectOverlay() {
        const url = parse(playerScreenUrl, true);
        const token = url.query.push ?? "";
        if (token === "") {
            errorMsg =
                "please enter your vdo url (from your web browser) before connecting";
            return;
        }
        connectState = "connecting";

        await invoke("ws_connect", { token: token })
            .then(() => {
                connectState = "disconnected";
            })
            .catch((e) => {
                connectState = "disconnected";
                errorMsg = e;
            });
    }

    async function disconnectOverlay() {
        connectState = "disconnecting";
        await invoke("ws_disconnect");
        connectState = "disconnected";
    }

    async function connResp() {
        await listen("ws_conn", (event) => {
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
        <h2>Server:</h2>
        <p>{errorMsg}</p>
    </div>
    <div class="inner-content-row user-input">
        {#if connectState === "disconnected"}
            <input
                id="player-screen-url-input"
                placeholder="Enter your vdo ninja url"
                bind:value={playerScreenUrl}
            />
            <button on:click={connectOverlay}> Connect to Server </button>
        {:else if connectState === "connected"}
            <input
                id="player-screen-url-input"
                placeholder="Enter your vdo ninja url"
                bind:value={playerScreenUrl}
                disabled
            />
            <button on:click={disconnectOverlay}> Disconnect Server </button>
        {:else if connectState === "connecting"}
            <input
                id="player-screen-url-input"
                placeholder="Enter your vdo ninja url"
                bind:value={playerScreenUrl}
                disabled
            />
            <button on:click={disconnectOverlay}> Cancel Connection </button>
        {:else}
            <input
                id="player-screen-url-input"
                placeholder="Enter your vdo ninja url"
                bind:value={playerScreenUrl}
                disabled
            />
            <button on:click={disconnectOverlay} disabled>
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

    input {
        overflow-x: hidden;
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
    }

    .connect {
        background-color: var(--color-moonshot-extra-blue-dark);
    }

    .noconnect {
        background-color: var(--color-moonshot-extra-burgundy);
    }
</style>
