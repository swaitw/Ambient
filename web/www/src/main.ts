import "./style.css";

declare global {
    interface Window {
        audio_ctx: AudioContext | null;
        audio_params: AudioProcessorParams;
        audioStart: () => void;
        audioStop: () => void;
        setupAudio: () => void;
    }
}

async function URLFromFiles(files: string[]) {
    const promises = files.map((file) =>
        fetch(file).then((response) => response.text())
    );

    const texts = await Promise.all(promises);
    const text = texts.join("");
    const blob = new Blob([text], { type: "application/javascript" });
    return URL.createObjectURL(blob);
}

import audioProcessorUrl from "./audio_processor.ts?url";

async function setupAudio() {
    let audio_ctx = new AudioContext();
    audio_ctx.suspend();
    window.audio_ctx = audio_ctx;

    console.info(`Importing audio processor from ${audioProcessorUrl}`);
    let module = await URLFromFiles([audioProcessorUrl]);

    if (window.audio_ctx.audioWorklet === undefined) {
        console.error("No AudioWorklet.");
    } else {
        window.audio_ctx.audioWorklet.addModule(module).then(() => {
            let dataSAB = new SharedArrayBuffer(2048 * 4); // 4 is the byte lenth
            let pointerSAB = new SharedArrayBuffer(2 * 4);
            let writePtr = new Uint32Array(pointerSAB, 0, 1);
            let readPtr = new Uint32Array(pointerSAB, 4, 1);

            window.audio_params = {
                dataSAB,
                pointerSAB,
                writePtr,
                readPtr,
            };

            const n = new AudioWorkletNode(audio_ctx, "audio_processor", {
                processorOptions: {
                    dataSAB: dataSAB,
                    pointerSAB: pointerSAB,
                    writePtr: writePtr,
                    readPtr: readPtr,
                },
            });
            n.connect(audio_ctx.destination);
        });
    }
}

window.setupAudio = setupAudio;

window.audioStart = () => {
    if (window.audio_ctx) {
        window.audio_ctx.resume();
    }
};

window.audioStop = () => {
    if (window.audio_ctx) {
        window.audio_ctx.suspend();
    }
};

import("ambient_web")
    .catch((e) => console.error("Error importing `ambient`:", e))
    .then((ambient) => {
        if (!ambient) {
            console.error("Ambient is null");
            return;
        }

        let target = window.document.getElementById("instance-container");

        if (!target) {
            console.error("No target");
            return;
        }

        const urlParams = new URLSearchParams(window.location.search);
        const packageId = urlParams.get('package_id');
        const version = urlParams.get('version');
        const deploymentId = urlParams.get('deployment_id');
        const userId = urlParams.get('userId');
        const context = urlParams.get('context');
        const debuggerOn = urlParams.get('debugger') != null;
        const serverUrl = urlParams.get('server_url');
        const maxPlayers = urlParams.get('max_players');

        let params = new URLSearchParams();
        if (packageId) {
            params.set('package_id', packageId);
        }
        if (version) {
            params.set('version', version);
        }
        if (deploymentId) {
            params.set('deployment_id', deploymentId);
        }

        if (maxPlayers) {
            params.set('max_players', maxPlayers);
        }
       
        if (context) {
            params.set('context', context);
        }

        const url = serverUrl && serverUrl || params.size != 0 && `https://api.ambient.run/servers/ensure-running?${params.toString()}` || "https://127.0.0.1:9000";

        console.log(`Connecting to ${url}`)

        let settings = {
            enableLogging: true,
            enablePanicHook: true,
            logFilter: "info",
            allowVersionMismatch: true,
            debugger: debuggerOn,
            userId: userId,
        };

        (async () => {
            try {
                await ambient.start(target, url, settings)
            }
             catch (e) {
                console.error("Error starting ambient: ", e);
             }
        })()
        // setupAudio();
    });
