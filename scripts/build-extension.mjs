import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, "..");
const extensionDir = resolve(rootDir, "extension");
const extensionWasmDir = resolve(extensionDir, "wasm");

function resetDir(path) {
  rmSync(path, { recursive: true, force: true });
  mkdirSync(path, { recursive: true });
}

function copyPath(source, target) {
  if (!existsSync(source)) {
    throw new Error(`Missing expected path: ${source}`);
  }

  mkdirSync(dirname(target), { recursive: true });
  cpSync(source, target, { recursive: true });
}

resetDir(extensionWasmDir);
copyPath(resolve(rootDir, "packages/defuddle-wasm/index.js"), resolve(extensionWasmDir, "index.js"));
copyPath(resolve(rootDir, "packages/defuddle-wasm/index.d.ts"), resolve(extensionWasmDir, "index.d.ts"));
copyPath(resolve(rootDir, "packages/defuddle-wasm/pkg"), resolve(extensionWasmDir, "pkg"));
