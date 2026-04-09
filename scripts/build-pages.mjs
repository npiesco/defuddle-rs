import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, "..");
const distDir = resolve(rootDir, "dist");

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

resetDir(distDir);
copyPath(resolve(rootDir, "app/index.html"), resolve(distDir, "index.html"));
copyPath(resolve(rootDir, "app/styles.css"), resolve(distDir, "styles.css"));
copyPath(resolve(rootDir, "app/app.js"), resolve(distDir, "app.js"));
copyPath(resolve(rootDir, "packages/defuddle-wasm/index.js"), resolve(distDir, "wasm/index.js"));
copyPath(resolve(rootDir, "packages/defuddle-wasm/index.d.ts"), resolve(distDir, "wasm/index.d.ts"));
copyPath(resolve(rootDir, "packages/defuddle-wasm/pkg"), resolve(distDir, "wasm/pkg"));
writeFileSync(resolve(distDir, ".nojekyll"), "");
