/**
 * Demo configuration — paths, commands, and output artifacts.
 *
 * Modeled after the Duckcells demo config, but scoped to fsgdb's
 * terminal + VS Code + MCP investigation flow.
 */
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const envPath = path.join(__dirname, '.env');
if (existsSync(envPath)) {
  for (const line of readFileSync(envPath, 'utf8').split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const eq = trimmed.indexOf('=');
    if (eq < 0) continue;
    const key = trimmed.slice(0, eq);
    const value = trimmed.slice(eq + 1);
    if (!process.env[key]) process.env[key] = value;
  }
}

export const REPO_ROOT = path.resolve(__dirname, '..');
export const OUTPUT_DIR = path.join(__dirname, 'output');

export const SERVO_REPO_URL = process.env.DEMO_SERVO_REPO_URL ?? 'https://github.com/servo/servo';
export const SERVO_DIR = process.env.DEMO_SERVO_DIR ?? '/tmp/servo';
export const DEMO_DB_DIR = process.env.DEMO_DB_DIR ?? '/tmp/fsgdb-servo-demo';
export const DEMO_DB_PATH = process.env.DEMO_DB_PATH ?? path.join(DEMO_DB_DIR, 'graph.db');

export const FSGDB_BIN = process.env.DEMO_FSGDB_BIN
  ?? path.join(REPO_ROOT, 'target', 'release', 'fsgdb');

export const VSCODE_EXTENSION_DIR = path.join(REPO_ROOT, 'vscode-extension');
export const VSCODE_ISOLATED_CMD = process.env.DEMO_VSCODE_ISOLATED_CMD
  ?? 'timeout 120 npx @vscode/test-cli --config .vscode-test-isolated.js';
export const VSCODE_DEMO_CMD = process.env.DEMO_VSCODE_DEMO_CMD
  ?? 'timeout 180 npx @vscode/test-cli --config .vscode-test-demo.js';
export const AGENT_TERMINAL_LAUNCH_CMD = process.env.DEMO_AGENT_TERMINAL_LAUNCH_CMD
  ?? 'exo-open --launch TerminalEmulator';
export const AGENT_CLI_CMD = process.env.DEMO_AGENT_CLI_CMD
  ?? 'copilot';
export const AGENT_SCENE_TIMEOUT_MS = Number(process.env.DEMO_AGENT_TIMEOUT_MS ?? '600000');

export const RECORDING_PATH = path.join(OUTPUT_DIR, 'fsgdb_demo_silent.mp4');
export const NARRATION_PATH = path.join(OUTPUT_DIR, 'narration.wav');
export const FINAL_VIDEO_PATH = path.join(OUTPUT_DIR, 'fsgdb_demo_final.mp4');
export const TIMELINE_PATH = path.join(OUTPUT_DIR, 'timeline.json');

export const SEGMENTS = {
  SETUP_SCAN: 'seg_01_setup_and_scan.mp4',
  CLI: 'seg_02_cli.mp4',
  VSCODE_QUERY: 'seg_03_vscode_query.mp4',
  VSCODE_GRAPH: 'seg_04_vscode_graph.mp4',
  AGENT: 'seg_05_agent_and_conclusion.mp4',
} as const;

export const SPEECH_KEY = process.env.AZURE_SPEECH_KEY
  ?? process.env.FOUNDRY_API_KEY
  ?? '';

export const SPEECH_REGION = process.env.AZURE_SPEECH_REGION
  ?? process.env.FOUNDRY_REGION
  ?? 'eastus2';

export const SPEECH_VOICE = process.env.DEMO_SPEECH_VOICE
  ?? 'en-US-AndrewMultilingualNeural';

export const WINDOW_TITLES = {
  TERMINAL: process.env.DEMO_TERMINAL_TITLE ?? '',
  VSCODE: process.env.DEMO_VSCODE_TITLE ?? 'Visual Studio Code',
  AGENT: process.env.DEMO_AGENT_TITLE ?? '',
} as const;

export const DEMO_FLAGS = {
  DRY_RUN: process.env.DEMO_DRY_RUN === '1',
  RUN_TERMINAL: process.env.DEMO_RUN_TERMINAL !== '0',
  RUN_VSCODE: process.env.DEMO_RUN_VSCODE !== '0',
  RUN_AGENT: process.env.DEMO_RUN_AGENT === '1',
  SYNTH_NARRATION: process.env.DEMO_SYNTH_NARRATION === '1',
  MERGE_VIDEO: process.env.DEMO_MERGE_VIDEO === '1',
} as const;

export const PAUSE = {
  SHORT: 1_500,
  MEDIUM: 2_500,
  LONG: 4_000,
} as const;
