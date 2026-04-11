import { spawnSync } from 'node:child_process';

import {
  DEMO_DB_PATH,
  FSGDB_BIN,
  SERVO_DIR,
  VSCODE_DEMO_CMD,
  VSCODE_EXTENSION_DIR,
} from './config';

export interface VSCodeDriverStep {
  label: string;
  command: string;
  notes: string;
}

export function buildVSCodeSteps(): VSCodeDriverStep[] {
  return [
    {
      label: 'Run isolated VS Code query editor scene',
      command:
        `DEMO_DB_PATH=${DEMO_DB_PATH} ` +
        `DEMO_WORKSPACE_PATH=${SERVO_DIR} ` +
        `DEMO_FSGDB_BIN=${FSGDB_BIN} ` +
        `DEMO_VSCODE_GREP="DEMO FLOW: opens Cypher Query Editor" ` +
        `${VSCODE_DEMO_CMD}`,
      notes: 'This launches the isolated VS Code harness and records the query editor segment only.',
    },
    {
      label: 'Run isolated VS Code graph visualization scene',
      command:
        `DEMO_DB_PATH=${DEMO_DB_PATH} ` +
        `DEMO_WORKSPACE_PATH=${SERVO_DIR} ` +
        `DEMO_FSGDB_BIN=${FSGDB_BIN} ` +
        `DEMO_VSCODE_GREP="DEMO FLOW: opens Graph Visualization" ` +
        `${VSCODE_DEMO_CMD}`,
      notes: 'This launches the isolated VS Code harness and records the graph visualization segment only.',
    },
  ];
}

export function runVSCodeStep(step: VSCodeDriverStep): void {
  console.log(`\n=== ${step.label} ===`);
  console.log(step.notes);
  const result = spawnSync('/bin/bash', ['-lc', step.command], {
    cwd: VSCODE_EXTENSION_DIR,
    encoding: 'utf8',
    stdio: 'inherit',
  });
  if (result.status !== 0) {
    throw new Error(`VS Code demo step failed (${result.status}): ${step.command}`);
  }
}

export function runVSCodeScenes(): void {
  const steps = buildVSCodeSteps();
  for (const step of steps) {
    runVSCodeStep(step);
  }
}
