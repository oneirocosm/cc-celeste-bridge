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
    <div class="inner-content-row">
        <button on:click={listen}> Connect to Game </button>
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
