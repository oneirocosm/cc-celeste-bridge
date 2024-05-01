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
    <div class="inner-content-row user-input">
        <input
            id="player-screen-url-input"
            placeholder="Enter your vdo ninja url"
            bind:value={playerScreenUrl}
        />
        <button on:click={connectOverlay}> Connect to Server </button>
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
        background-color: var(--color-moonshot-extra-blue-dark);
    }
</style>
