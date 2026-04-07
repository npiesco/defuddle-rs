import init, { parseJson as wasmParseJson } from "./pkg/defuddle_rs.js";

let initPromise;
let initialized = false;

function ensureInitialized() {
  if (!initialized) {
    throw new Error(
      "defuddle WASM package is not initialized. Call initDefuddleWasm() before invoking the parser.",
    );
  }
}

function parseResponse(responseJson) {
  const parsed = JSON.parse(responseJson);

  if (parsed && typeof parsed === "object" && "error" in parsed) {
    throw new Error(String(parsed.error));
  }

  return parsed;
}

export function initDefuddleWasm(input) {
  if (!initPromise) {
    initPromise = init(input).then((result) => {
      initialized = true;
      return result;
    });
  }

  return initPromise;
}

export function parseJson(html, url) {
  ensureInitialized();
  return wasmParseJson(html, url);
}

export function parse(html, url) {
  ensureInitialized();
  return parseResponse(wasmParseJson(html, url));
}
