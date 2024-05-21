<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { listen } from "@tauri-apps/api/event";
    import parse from "url-parse";

    let serverUrl = "";
    let playerScreenUrl = "";
    let errorMsg = "";
    let connectState = "disconnected";

    type BtnMsgs = { [key: string]: string };
    const btnMsgs: BtnMsgs = {
        disconnected: "Connect to Server",
        connecting: "Cancel Connection",
        connected: "Disconnect Server",
        disconnecting: "Disconnecting ...",
    };

    async function connectOverlay() {
        errorMsg = "";
        const screenUrlParsed = parse(playerScreenUrl, true);
        const token = screenUrlParsed.query.push ?? "";
        let tempMsg = [];
        if (token === "") {
            tempMsg.push(
                "please enter your gameplay vdo url (from your web browser) before connecting",
            );
        }
        const serverUrlParsed = parse(serverUrl, {});
        const host = serverUrlParsed.host ?? "";
        if (host === "") {
            tempMsg.push(
                "please enter the server url (ask the stream host) before connecting",
            );
        }
        if (tempMsg.length !== 0) {
            errorMsg = tempMsg.join("\n\n");
            return;
        }

        connectState = "connecting";
        await invoke("ws_connect", { token: token, host: serverUrlParsed.href })
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

    type BtnFuncs = { [key: string]: any };
    const btnFuncs: BtnFuncs = {
        disconnected: connectOverlay,
        connecting: disconnectOverlay,
        connected: disconnectOverlay,
        disconnecting: disconnectOverlay,
    };

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
        <pre><p>{errorMsg}</p></pre>
    </div>
    <div class="inner-content-row user-input">
        <div class="inputbox-box">
            <div class="label-input-box">
                <p>server url</p>
                <input
                    id="player-screen-url-input"
                    placeholder="Enter server url"
                    bind:value={serverUrl}
                    disabled={connectState !== "disconnected"}
                />
            </div>
            <div class="label-input-box">
                <p>vdo ninja url (game)</p>
                <input
                    id="player-screen-url-input"
                    placeholder="Enter your vdo ninja url"
                    bind:value={playerScreenUrl}
                    disabled={connectState !== "disconnected"}
                />
            </div>
        </div>
        <button
            on:click={btnFuncs[connectState]()}
            disabled={connectState === "disconnecting"}
        >
            {btnMsgs[connectState]}
        </button>
    </div>
</div>

<style>
    h2 {
        font-family: Audiowide;
        text-transform: uppercase;
        transform: skew(-10deg, 0deg);
        color: var(--color-moonshot-core-yellow);
    }
    p {
        font-family: Audiowide;
        font-size: 1rem;
        margin: 0;
        padding: 0;
        color: var(--color-moonshot-extra-gold);
    }

    pre {
        white-space: pre-wrap;
    }

    input {
        overflow-x: hidden;
    }

    button {
        align-self: end;
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

    .inputbox-box {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        margin-right: auto;
        margin-left: 2rem;
    }

    .label-input-box {
        display: flex;
        flex-direction: column;
        align-self: flex-start;
        align-items: start;
    }

    .connect {
        background-color: var(--color-moonshot-extra-blue-dark);
    }

    .noconnect {
        background-color: var(--color-moonshot-extra-burgundy);
    }
</style>
