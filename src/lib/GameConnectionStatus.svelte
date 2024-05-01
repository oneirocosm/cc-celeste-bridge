<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";

    let errorMsg = "";
    let errorTimeout: ReturnType<typeof setTimeout>;
    const noError = "todo";

    async function listen() {
        let tempMsg = (await invoke("listen", { errorMsg })) as string;
        setErrorMsg(tempMsg, 2000);
    }

    async function setErrorMsg(msg: string, timeoutMs: number) {
        clearTimeout(errorTimeout);
        errorMsg = msg;

        errorTimeout = setTimeout(async () => {
            errorMsg = noError;
        }, timeoutMs);
    }
</script>

<div class="outer-content-box">
    <div class="inner-content-row">
        <h2>Game:</h2>
        <p></p>
    </div>
    <div class="inner-content-row user-input">
        <button on:click={listen}> Connect to Game </button>
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
