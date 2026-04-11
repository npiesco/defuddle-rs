/**
 * Agent demo driver.
 *
 * This does NOT use agent-tty. The real demo surface is the system terminal
 * launched via `exo-open --launch TerminalEmulator`.
 *
 * agent-tty was only useful as a reference for how to think about driving
 * a terminal-native agent session.
 */
import { chmodSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import {
  AGENT_CLI_CMD,
  AGENT_TERMINAL_LAUNCH_CMD,
  DEMO_DB_PATH,
  OUTPUT_DIR,
  REPO_ROOT,
  SERVO_DIR,
} from './config';

export interface AgentScenePlan {
  launchCommand: string;
  agentCommand: string;
  initialPrompt: string;
  notes: string[];
}

export const AGENT_SCENE_PLAN: AgentScenePlan = {
  launchCommand: AGENT_TERMINAL_LAUNCH_CMD,
  agentCommand: AGENT_CLI_CMD,
  initialPrompt:
    `We are evaluating whether Servo could become a viable Playwright automation target. ` +
    `Use the fsgdb MCP tools against the shared graph at ${DEMO_DB_PATH}. ` +
    `Treat the graph and MCP-backed structural analysis as your primary reasoning surface. ` +
    `Start with graph-backed search, module explanation, dependency tracing, call graph analysis, and test-impact tools before falling back to manual file reads. ` +
    `Identify likely startup, embedding, runtime control, and testing seams, then summarize the most promising hook points, ` +
    `the validation path, and the main risks. ` +
    `Be explicit about which MCP tools you used and why.`,
  notes: [
    'Launch the system terminal with exo-open --launch TerminalEmulator.',
    'Start Copilot inside that terminal.',
    'The first prompt in the session should be the prepared investigation prompt.',
    'Record the terminal window as the agent scene once the session is visible.',
  ],
};

export function emitAgentPlan(): {
  jsonPath: string;
  promptPath: string;
  launchScriptPath: string;
  statusPath: string;
  transcriptPath: string;
  launchCommand: string;
} {
  const jsonPath = path.join(OUTPUT_DIR, 'agent_scene.json');
  const promptPath = path.join(OUTPUT_DIR, 'agent_initial_prompt.txt');
  const launchScriptPath = path.join(OUTPUT_DIR, 'agent_scene_launch.sh');
  const statusPath = path.join(OUTPUT_DIR, 'agent_scene_status.json');
  const transcriptPath = path.join(OUTPUT_DIR, 'agent_scene_transcript.txt');

  writeFileSync(jsonPath, JSON.stringify(AGENT_SCENE_PLAN, null, 2));
  writeFileSync(promptPath, AGENT_SCENE_PLAN.initialPrompt + '\n');

  const launchScript = `#!/usr/bin/env bash
set -euo pipefail

STATUS_PATH=${JSON.stringify(statusPath)}
PROMPT_PATH=${JSON.stringify(promptPath)}
TRANSCRIPT_PATH=${JSON.stringify(transcriptPath)}
PROJECT_ROOT=${JSON.stringify(REPO_ROOT)}
SERVO_DIR=${JSON.stringify(SERVO_DIR)}

cd "$PROJECT_ROOT"
rm -f "$STATUS_PATH"
touch "$TRANSCRIPT_PATH"

echo "[agent scene] pwd=$(pwd)" | tee -a "$TRANSCRIPT_PATH"
if [ "$(pwd)" != "$PROJECT_ROOT" ]; then
  cat > "$STATUS_PATH" <<JSON
{
  "startedAt": "$(date -Iseconds)",
  "endedAt": "$(date -Iseconds)",
  "durationMs": 0,
  "exitCode": 98,
  "cwd": "$(pwd)",
  "expectedCwd": "$PROJECT_ROOT",
  "transcriptPath": "$TRANSCRIPT_PATH",
  "error": "Terminal launched in unexpected working directory"
}
JSON
  echo "[agent scene] unexpected cwd, aborting" | tee -a "$TRANSCRIPT_PATH"
  exec bash -i
fi

start_ms=$(python - <<'PY'
import time
print(int(time.time() * 1000))
PY
)
start_iso=$(date -Iseconds)

prompt=$(cat "$PROMPT_PATH")
set +e
printf '/exit\\n' | ${AGENT_CLI_CMD} -i "$prompt" --no-color --add-dir "$PROJECT_ROOT" --add-dir "$SERVO_DIR" 2>&1 | tee "$TRANSCRIPT_PATH"
rc=\${PIPESTATUS[1]}
set -e

end_ms=$(python - <<'PY'
import time
print(int(time.time() * 1000))
PY
)
end_iso=$(date -Iseconds)
duration_ms=$((end_ms - start_ms))

cat > "$STATUS_PATH" <<JSON
{
  "startedAt": "$start_iso",
  "endedAt": "$end_iso",
  "durationMs": $duration_ms,
  "exitCode": $rc,
  "cwd": "$PROJECT_ROOT",
  "transcriptPath": "$TRANSCRIPT_PATH"
}
JSON

echo
echo "[agent scene complete] status written to $STATUS_PATH"
echo "[agent scene complete] transcript written to $TRANSCRIPT_PATH"
exec bash -i
`;

  writeFileSync(launchScriptPath, launchScript);
  chmodSync(launchScriptPath, 0o755);

  const launchCommand = `${AGENT_TERMINAL_LAUNCH_CMD} --working-directory ${JSON.stringify(REPO_ROOT)} /bin/bash ${JSON.stringify(launchScriptPath)}`;

  return { jsonPath, promptPath, launchScriptPath, statusPath, transcriptPath, launchCommand };
}
