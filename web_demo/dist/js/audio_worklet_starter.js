var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
/**
 * The browser does not support WebAssembly.
 */
class WebAssemblyNotSupportedError extends Error {
    constructor() {
        super("WebAssembly is not supported in this browser.");
        this.name = "WebAssemblyNotSupportedError";
    }
}
/**
 * The browser does not support audio worklets.
 */
class AudioWorkletNotSupportedError extends Error {
    constructor() {
        super("AudioWorlket is not supported in this browser.");
        this.name = "AudioWorkletNotSupportedError";
    }
}
/**
 * Failed to fetch the WebAssembly module at the specified URL.
 */
class WebAssemblyFetchError extends Error {
    constructor(url, httpStatus) {
        super("Fetching WebAssembly module from " + url + " failed with status " + httpStatus);
        this.name = "WebAssemblyFetchError";
    }
}
/**
 * Determines how to handle microphone access.
 */
var MicrophoneMode;
(function (MicrophoneMode) {
    /**
     * Mic access is requested if the number of inputs
     * specified in the worklet node options is greater than zero.
     * No error is thrown if access is denied.
     */
    MicrophoneMode["optional"] = "optional";
    /**
     * Mic access is requested if the number of inputs
     * specified in the worklet node options is greater than zero.
     * An error is thrown if access is denied.
     */
    MicrophoneMode["required"] = "required";
    /**
     * Mic access is not requested, regardless of the number of inputs
     * specified in the worklet node options.
     */
    MicrophoneMode["disabled"] = "disabled";
})(MicrophoneMode || (MicrophoneMode = {}));
/**
 * Start and create an audio worklet node.
 * @param options See AudioWorkletOptions
 * @returns A promise that resolves with the created AudioWorkletNode.
 */
function startAudioWorklet(options) {
    var _a, _b, _c;
    return __awaiter(this, void 0, void 0, function* () {
        const defaultSampleRate = 44100;
        const sampleRate = (_a = options.sampleRate) !== null && _a !== void 0 ? _a : defaultSampleRate;
        const microphoneMode = (_b = options.microphoneMode) !== null && _b !== void 0 ? _b : MicrophoneMode.optional;
        // If WebAssembly is used, make sure it's supported by the browser
        const wasmIsSupported = typeof WebAssembly === "object" && typeof WebAssembly.instantiate === "function";
        if (!wasmIsSupported && options.wasmUrl !== undefined) {
            throw new WebAssemblyNotSupportedError();
        }
        // Create web audio context
        let contextOptions = { sampleRate, latencyHint: "interactive" };
        let context;
        if (window.webkitAudioContext) {
            // AudioContext is undefined in Safari and old versions of Chrome
            context = new (window.webkitAudioContext)(contextOptions);
        }
        else {
            context = new AudioContext(contextOptions);
        }
        // Make sure the browser supports audio worklets
        const audioWorkletIsSupported = context.audioWorklet !== undefined;
        if (!audioWorkletIsSupported) {
            throw new AudioWorkletNotSupportedError();
        }
        // Request microphone access.
        let micStream = null;
        const atLeastOneInputIsRequested = (_c = options.workletNodeOptions.numberOfInputs) !== null && _c !== void 0 ? _c : 0 > 0;
        if (atLeastOneInputIsRequested && microphoneMode != MicrophoneMode.disabled) {
            try {
                micStream = yield navigator.mediaDevices.getUserMedia({ audio: true, video: false });
            }
            catch (e) {
                if (microphoneMode == MicrophoneMode.required) {
                    throw e;
                }
            }
        }
        // A URL suffix used to prevent caching of the worklet processor
        // and WebAssembly src.
        const urlTimestampSuffix = "?t=" + new Date().getTime();
        yield context.audioWorklet.addModule(options.workletProcessorUrl + urlTimestampSuffix);
        const workletNodeOptions = options.workletNodeOptions;
        // Add the actual sample rate to the worklet processor
        workletNodeOptions.processorOptions = Object.assign(Object.assign({}, workletNodeOptions.processorOptions), { "sampleRate": context.sampleRate });
        // Create audio worklet node
        const workletNode = new AudioWorkletNode(context, options.workletNodeName, options.workletNodeOptions);
        // Connect outputs, if any
        if (workletNode.numberOfOutputs > 0) {
            workletNode.connect(context.destination);
        }
        // If there is a microphone stream, connect it to the worklet node
        if (micStream != null) {
            const micSource = context.createMediaStreamSource(micStream);
            micSource.connect(workletNode);
        }
        // Load WebAssembly module, if specified, and send it to the worklet node
        const wasmUrl = options.wasmUrl;
        if (wasmUrl) {
            const urlToFetch = wasmUrl + urlTimestampSuffix;
            const fetchResult = yield fetch(urlToFetch);
            if (!fetchResult.ok) {
                throw new WebAssemblyFetchError(urlToFetch, fetchResult.status);
            }
            const wasmData = yield fetchResult.arrayBuffer();
            workletNode.port.postMessage({ type: 'wasmData', data: wasmData });
        }
        return workletNode;
    });
}
