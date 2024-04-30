<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";

    let playerScreenUrl = "";
    let errorMsg = "";
    let errorTimeout: ReturnType<typeof setTimeout>;
    const noError = "todo";

    async function connectOverlay() {
        if (!playerScreenUrl.includes("?push=")) {
            setErrorMsg(
                "please enter your vdo url (from your web broser) before connecting",
                2000,
            );
            return;
        }
        await invoke("connect_overlay", { invokeMessage: playerScreenUrl });
    }

    async function setErrorMsg(msg: string, timeoutMs: number) {
        clearTimeout(errorTimeout);
        errorMsg = msg;

        errorTimeout = setTimeout(async () => {
            errorMsg = noError;
        }, timeoutMs);
    }

    onbeforeunload = () => clearTimeout(errorTimeout);
</script>

<div class="outer-content-box">
    <div class="inner-content-row">
        <h2>Server:</h2>
        <p>{errorMsg}</p>
    </div>
    <div class="inner-content-row">
        <input
            id="player-screen-url-input"
            placeholder="Enter your vdo ninja url"
            bind:value={playerScreenUrl}
        />
        <button on:click={connectOverlay}> Connect to Server </button>
    </div>
</div>

<style>
    .inner-content-row {
        display: flex;
        flex-direction: row;
    }

    .outer-content-box {
        border: 2px solid white;
        padding: 1rem;
        border-radius: 0.5rem;
    }
</style>
