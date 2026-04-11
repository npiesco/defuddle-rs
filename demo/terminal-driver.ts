/**
 * Terminal driver for the fsgdb demo.
 *
 * This is the first executable scene driver: it prepares Servo, builds the
 * shared graph, and emits a small set of legible CLI queries for the demo.
 *
 * The goal here is not full automation yet. The goal is a deterministic,
 * readable command plan that the orchestrator can call and later attach to
 * recording hooks.
 */
import { existsSync, mkdirSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';

import {
  DEMO_DB_DIR,
  DEMO_DB_PATH,
  FSGDB_BIN,
  SERVO_DIR,
  SERVO_REPO_URL,
} from './config';

export interface DriverStep {
  label: string;
  command: string;
  notes: string;
}

function runShell(command: string, cwd?: string): void {
  const result = spawnSync('/bin/bash', ['-lc', command], {
    cwd,
    encoding: 'utf8',
    stdio: 'inherit',
  });
  if (result.status !== 0) {
    throw new Error(`Command failed (${result.status}): ${command}`);
  }
}

function ensureFreshTempCloneTarget(repoDir: string): void {
  const resolved = path.resolve(repoDir);
  const tmpRoot = path.resolve('/tmp') + path.sep;
  if (!resolved.startsWith(tmpRoot)) {
    throw new Error(`Refusing to delete non-/tmp demo repo path: ${resolved}`);
  }
  if (existsSync(resolved)) {
    rmSync(resolved, { recursive: true, force: true });
  }
}

export function buildTerminalSteps(): DriverStep[] {
  return [
    {
      label: 'Reset temp clone target',
      command: `rm -rf ${SERVO_DIR}`,
      notes: 'Delete the existing temp repo first so clone timing is always real and repeatable.',
    },
    {
      label: 'Clone Servo fresh',
      command: `git clone ${SERVO_REPO_URL} ${SERVO_DIR}`,
      notes: 'Keep the clone trigger visible. This gives the demo a real clone duration every run.',
    },
    {
      label: 'Ensure demo db directory exists',
      command: `mkdir -p ${DEMO_DB_DIR}`,
      notes: 'Shared database path must be explicit and visible.',
    },
    {
      label: 'Scan Servo into the shared graph',
      command:
        `${FSGDB_BIN} --database ${DEMO_DB_PATH} scan ${SERVO_DIR} ` +
        `--enable-code-analysis ` +
        `--enable-git-analysis ` +
        `--exclude 'tests/**' ` +
        `--exclude 'third_party/**' ` +
        `--exclude 'resources/**'`,
      notes:
        'Use code analysis and git analysis for the demo, while skipping large non-core Servo trees to keep scan timing reasonable.',
    },
    {
      label: 'Show graph stats',
      command: `${FSGDB_BIN} --database ${DEMO_DB_PATH} stats`,
      notes: 'Use this as proof of life after scan completes.',
    },
    {
      label: 'Count indexed files',
      command: `${FSGDB_BIN} --database ${DEMO_DB_PATH} query --query "MATCH (f:File) RETURN count(f) AS files"`,
      notes: 'Simple readable query for the audience.',
    },
  ];
}

export function runTerminalStep(step: DriverStep): void {
  console.log(`\n=== ${step.label} ===`);
  console.log(step.notes);
  if (step.label === 'Reset temp clone target') {
    ensureFreshTempCloneTarget(SERVO_DIR);
    return;
  }
  runShell(step.command);
}

export function runTerminalScenes(): void {
  mkdirSync(DEMO_DB_DIR, { recursive: true });

  const steps = buildTerminalSteps();
  for (const step of steps) {
    runTerminalStep(step);
  }
}

export function terminalPreflight(): void {
  if (!existsSync(FSGDB_BIN)) {
    throw new Error(`fsgdb binary not found: ${FSGDB_BIN}`);
  }
}
