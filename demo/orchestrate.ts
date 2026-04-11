/**
 * defuddle Demo Orchestrator
 *
 * This copied scaffold now targets defuddle's six-scene flow:
 * hook, browser extension, Rust crate, Python bindings, MCP, and close.
 */
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, unlinkSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';

import {
  AGENT_TERMINAL_LAUNCH_CMD,
  AGENT_SCENE_TIMEOUT_MS,
  DEMO_FLAGS,
  DEMO_DB_DIR,
  DEMO_DB_PATH,
  FINAL_VIDEO_PATH,
  FSGDB_BIN,
  NARRATION_PATH,
  OUTPUT_DIR,
  RECORDING_PATH,
  SEGMENTS,
  SERVO_DIR,
  SPEECH_KEY,
  SPEECH_REGION,
  SPEECH_VOICE,
  TIMELINE_PATH,
  VSCODE_EXTENSION_DIR,
  WINDOW_TITLES,
} from './config';
import { NARRATION_SCRIPT, SEGMENT_GROUPS, buildSegmentSSML } from './narration';
import { buildTerminalSteps, runTerminalStep, terminalPreflight } from './terminal-driver';
import { AGENT_SCENE_PLAN, emitAgentPlan } from './agent-driver';
import { buildVSCodeSteps, runVSCodeStep } from './vscode-driver';
import { CandycamClient } from './candycam-client';

interface TimelineStep {
  label: string;
  segment: string;
  notes: string;
  command?: string;
  expectedOutputs?: string[];
}

interface ExecutedTimelineStep extends TimelineStep {
  status: 'completed' | 'failed' | 'skipped';
  startedAt: string;
  endedAt: string;
  durationMs: number;
  error?: string;
}

interface TimelineDocument {
  generatedAt: string;
  dbPath: string;
  segments: typeof SEGMENTS;
  narrationPhases: typeof NARRATION_SCRIPT;
  segmentGroups: typeof SEGMENT_GROUPS;
  plannedSteps: TimelineStep[];
  executedSteps: ExecutedTimelineStep[];
}

function buildPlannedSteps(): TimelineStep[] {
  const terminalSteps = buildTerminalSteps();
  const vscodeSteps = buildVSCodeSteps();
  return [
    {
      label: terminalSteps[0].label,
      segment: SEGMENTS.SETUP_SCAN,
      notes: terminalSteps[0].notes,
      command: terminalSteps[0].command,
      expectedOutputs: [],
    },
    {
      label: terminalSteps[1].label,
      segment: SEGMENTS.SETUP_SCAN,
      notes: terminalSteps[1].notes,
      command: terminalSteps[1].command,
      expectedOutputs: [SERVO_DIR],
    },
    {
      label: terminalSteps[2].label,
      segment: SEGMENTS.SETUP_SCAN,
      notes: terminalSteps[2].notes,
      command: terminalSteps[2].command,
      expectedOutputs: [DEMO_DB_DIR],
    },
    {
      label: terminalSteps[3].label,
      segment: SEGMENTS.SETUP_SCAN,
      notes: terminalSteps[3].notes,
      command: terminalSteps[3].command,
      expectedOutputs: [DEMO_DB_PATH],
    },
    {
      label: terminalSteps[4].label,
      segment: SEGMENTS.CLI,
      notes: terminalSteps[4].notes,
      command: terminalSteps[4].command,
      expectedOutputs: [DEMO_DB_PATH],
    },
    {
      label: terminalSteps[5].label,
      segment: SEGMENTS.CLI,
      notes: terminalSteps[5].notes,
      command: terminalSteps[5].command,
      expectedOutputs: [DEMO_DB_PATH],
    },
    {
      label: vscodeSteps[0].label,
      segment: SEGMENTS.VSCODE_QUERY,
      notes: vscodeSteps[0].notes,
      command: `cd ${VSCODE_EXTENSION_DIR} && ${vscodeSteps[0].command}`,
    },
    {
      label: vscodeSteps[1].label,
      segment: SEGMENTS.VSCODE_GRAPH,
      notes: vscodeSteps[1].notes,
      command: `cd ${VSCODE_EXTENSION_DIR} && ${vscodeSteps[1].command}`,
    },
    {
      label: 'Agent and conclusion scene',
      segment: SEGMENTS.AGENT,
      notes: 'Launch the system terminal, start Copilot, and seed the prepared first prompt.',
      command: AGENT_TERMINAL_LAUNCH_CMD,
    },
  ];
}

function createTimelineDocument(): TimelineDocument {
  return {
    generatedAt: new Date().toISOString(),
    dbPath: DEMO_DB_PATH,
    segments: SEGMENTS,
    narrationPhases: NARRATION_SCRIPT,
    segmentGroups: SEGMENT_GROUPS,
    plannedSteps: buildPlannedSteps(),
    executedSteps: [],
  };
}

function persistTimeline(doc: TimelineDocument): void {
  writeFileSync(TIMELINE_PATH, JSON.stringify(doc, null, 2));
}

function preflight(): void {
  console.log('=== defuddle Demo Orchestrator ===\n');

  mkdirSync(OUTPUT_DIR, { recursive: true });
  console.log(`[✓] Output directory: ${OUTPUT_DIR}`);

  if (!existsSync(FSGDB_BIN)) {
    throw new Error(`fsgdb binary not found: ${FSGDB_BIN}`);
  }
  console.log(`[✓] fsgdb binary: ${FSGDB_BIN}`);

  if (!existsSync(VSCODE_EXTENSION_DIR)) {
    throw new Error(`VS Code extension directory not found: ${VSCODE_EXTENSION_DIR}`);
  }
  console.log(`[✓] VS Code extension dir: ${VSCODE_EXTENSION_DIR}`);

  terminalPreflight();

  if (DEMO_FLAGS.RUN_AGENT) {
    const copilotVersion = spawnSync('copilot', ['--version'], { encoding: 'utf8', timeout: 10_000 });
    if (copilotVersion.status !== 0) {
      throw new Error('copilot CLI not found or not executable');
    }
    console.log(`[✓] copilot CLI: ${copilotVersion.stdout.trim()}`);
  }

  const ffmpegResult = spawnSync('ffmpeg', ['-version'], { encoding: 'utf8', timeout: 5_000 });
  if (ffmpegResult.status !== 0) {
    throw new Error('ffmpeg not found on PATH');
  }
  console.log('[✓] ffmpeg: available');

  if (SPEECH_KEY) {
    console.log(`[✓] Azure Speech TTS configured: region=${SPEECH_REGION}, voice=${SPEECH_VOICE}`);
  } else {
    console.log('[!] Azure Speech key not configured yet — narration synthesis will need credentials');
  }

  if (DEMO_FLAGS.DRY_RUN) {
    console.log('[✓] Dry run mode enabled: recording, TTS, and final merge are skipped');
  }

  console.log('');
}

function cleanOutput(): void {
  mkdirSync(OUTPUT_DIR, { recursive: true });
  for (const entry of readdirSync(OUTPUT_DIR)) {
    rmSync(path.join(OUTPUT_DIR, entry), { force: true, recursive: true });
  }
  mkdirSync(DEMO_DB_DIR, { recursive: true });
}

function emitTimeline(doc: TimelineDocument): void {
  persistTimeline(doc);
  console.log(`[✓] Timeline scaffold: ${TIMELINE_PATH}\n`);
}

function recordExecutedStep(
  doc: TimelineDocument,
  step: TimelineStep,
  startedAtMs: number,
  endedAtMs: number,
  status: ExecutedTimelineStep['status'],
  error?: string,
): void {
  doc.executedSteps.push({
    ...step,
    status,
    startedAt: new Date(startedAtMs).toISOString(),
    endedAt: new Date(endedAtMs).toISOString(),
    durationMs: Math.max(0, endedAtMs - startedAtMs),
    error,
  });
  persistTimeline(doc);
}

function formatDuration(durationMs: number): string {
  if (durationMs < 1_000) return `${durationMs}ms`;
  return `${(durationMs / 1_000).toFixed(1)}s`;
}

function executeTimedStep(
  doc: TimelineDocument,
  step: TimelineStep,
  runner: () => void,
): void {
  const startedAtMs = Date.now();
  console.log(`--- Timing: ${step.segment} :: ${step.label} ---`);
  try {
    runner();
    const endedAtMs = Date.now();
    recordExecutedStep(doc, step, startedAtMs, endedAtMs, 'completed');
    console.log(`  completed in ${formatDuration(endedAtMs - startedAtMs)}\n`);
  } catch (error) {
    const endedAtMs = Date.now();
    const message = error instanceof Error ? error.message : String(error);
    recordExecutedStep(doc, step, startedAtMs, endedAtMs, 'failed', message);
    console.log(`  failed after ${formatDuration(endedAtMs - startedAtMs)}\n`);
    throw error;
  }
}

async function executeTimedAsyncStep(
  doc: TimelineDocument,
  step: TimelineStep,
  runner: () => Promise<void>,
): Promise<void> {
  const startedAtMs = Date.now();
  console.log(`--- Timing: ${step.segment} :: ${step.label} ---`);
  try {
    await runner();
    const endedAtMs = Date.now();
    recordExecutedStep(doc, step, startedAtMs, endedAtMs, 'completed');
    console.log(`  completed in ${formatDuration(endedAtMs - startedAtMs)}\n`);
  } catch (error) {
    const endedAtMs = Date.now();
    const message = error instanceof Error ? error.message : String(error);
    recordExecutedStep(doc, step, startedAtMs, endedAtMs, 'failed', message);
    console.log(`  failed after ${formatDuration(endedAtMs - startedAtMs)}\n`);
    throw error;
  }
}

function synthesizeNarrationPlan(): void {
  console.log('--- Narration plan ---');
  console.log(`  Scenes: ${NARRATION_SCRIPT.length}`);
  console.log(`  Segment groups: ${SEGMENT_GROUPS.length}`);

  for (const group of SEGMENT_GROUPS) {
    const ssml = buildSegmentSSML(group.phases, SPEECH_VOICE);
    const preview = ssml.split('\n').slice(0, 4).join(' ');
    console.log(`  ${group.segPrefix}: phases=${group.phases.join(', ')} :: ${preview.substring(0, 120)}...`);
  }

  console.log('');
}

function summarizeExpectedArtifacts(): void {
  const artifacts = [
    RECORDING_PATH,
    NARRATION_PATH,
    FINAL_VIDEO_PATH,
    TIMELINE_PATH,
    path.join(OUTPUT_DIR, SEGMENTS.SETUP_SCAN),
    path.join(OUTPUT_DIR, SEGMENTS.CLI),
    path.join(OUTPUT_DIR, SEGMENTS.VSCODE_QUERY),
    path.join(OUTPUT_DIR, SEGMENTS.VSCODE_GRAPH),
    path.join(OUTPUT_DIR, SEGMENTS.AGENT),
    path.join(OUTPUT_DIR, SEGMENTS.CLOSE),
    path.join(OUTPUT_DIR, 'agent_scene.json'),
    path.join(OUTPUT_DIR, 'agent_initial_prompt.txt'),
  ];

  console.log('Expected artifacts:');
  for (const artifact of artifacts) {
    console.log(`  - ${artifact}`);
  }
  if (DEMO_FLAGS.DRY_RUN) {
    console.log('  - dry run mode: segment/video/audio artifacts may be intentionally absent');
  }
  console.log('');
}

function summarizeCurrentOutput(): void {
  console.log('Current output state:');
  const entries = existsSync(OUTPUT_DIR) ? readdirSync(OUTPUT_DIR) : [];
  if (entries.length === 0) {
    console.log('  (empty)');
    console.log('');
    return;
  }

  for (const entry of entries) {
    const fullPath = path.join(OUTPUT_DIR, entry);
    const sizeKB = Math.round(statSync(fullPath).size / 1024);
    console.log(`  - ${entry} (${sizeKB} KB)`);
  }
  console.log('');
}

function findSegments(): string[] {
  const segments: string[] = [];
  if (!existsSync(OUTPUT_DIR)) return segments;
  for (const file of readdirSync(OUTPUT_DIR)) {
    if (file.startsWith('seg_') && file.endsWith('.mp4')) {
      segments.push(path.join(OUTPUT_DIR, file));
    }
  }
  segments.sort();
  return segments;
}

function isValidMp4(filePath: string): boolean {
  const probe = spawnSync(
    'ffprobe',
    ['-v', 'error', '-show_entries', 'format=duration', '-of', 'csv=p=0', filePath],
    { encoding: 'utf8', timeout: 10_000 },
  );
  return probe.status === 0 && parseFloat(probe.stdout?.trim() ?? '0') > 0;
}

function concatSegments(): void {
  if (DEMO_FLAGS.DRY_RUN) {
    console.log('--- Segment concat skipped in dry run mode ---\n');
    return;
  }

  console.log('--- Concatenating video segments ---');

  const segments = findSegments();
  if (segments.length === 0) {
    throw new Error('No video segments found in demo/output');
  }

  const invalid = segments.filter((segment) => !isValidMp4(segment));
  if (invalid.length > 0) {
    throw new Error(`Invalid segments: ${invalid.map((segment) => path.basename(segment)).join(', ')}`);
  }

  const intermediates: string[] = [];
  for (let i = 0; i < segments.length; i++) {
    const intermediate = path.join(OUTPUT_DIR, `_concat_${i}.mp4`);
    intermediates.push(intermediate);
    const result = spawnSync(
      'ffmpeg',
      ['-y', '-i', segments[i], '-an', '-c:v', 'copy', intermediate],
      { encoding: 'utf8', timeout: 60_000 },
    );
    if (result.status !== 0) {
      throw new Error(`ffmpeg failed while preparing ${path.basename(segments[i])}`);
    }
  }

  const concatList = path.join(OUTPUT_DIR, '_concat_list.txt');
  writeFileSync(
    concatList,
    intermediates.map((file) => `file '${file.replace(/\\/g, '/')}'`).join('\n'),
    'utf8',
  );

  const concatResult = spawnSync(
    'ffmpeg',
    ['-y', '-f', 'concat', '-safe', '0', '-i', concatList, '-c:v', 'copy', '-an', '-movflags', '+faststart', RECORDING_PATH],
    { encoding: 'utf8', timeout: 120_000 },
  );

  for (const file of intermediates) {
    try { unlinkSync(file); } catch { /* ignore */ }
  }
  try { unlinkSync(concatList); } catch { /* ignore */ }

  if (concatResult.status !== 0) {
    throw new Error(`ffmpeg concat failed: ${concatResult.stderr?.substring(0, 400)}`);
  }

  if (!isValidMp4(RECORDING_PATH)) {
    throw new Error(`Concatenated recording is invalid: ${RECORDING_PATH}`);
  }

  const sizeKB = Math.round(statSync(RECORDING_PATH).size / 1024);
  console.log(`[✓] Silent recording: ${RECORDING_PATH} (${sizeKB} KB)\n`);
}

async function synthesizeNarration(): Promise<void> {
  if (DEMO_FLAGS.DRY_RUN) {
    console.log('--- Narration synthesis skipped in dry run mode ---\n');
    return;
  }

  if (!DEMO_FLAGS.SYNTH_NARRATION) {
    console.log('--- Narration synthesis skipped ---\n');
    return;
  }

  if (!SPEECH_KEY) {
    throw new Error('Azure Speech key not configured but DEMO_SYNTH_NARRATION=1');
  }

  console.log('--- Synthesizing narration ---');
  const sdk = await import('microsoft-cognitiveservices-speech-sdk');
  const speechConfig = sdk.SpeechConfig.fromSubscription(SPEECH_KEY, SPEECH_REGION);
  speechConfig.speechSynthesisOutputFormat =
    sdk.SpeechSynthesisOutputFormat.Riff16Khz16BitMonoPcm;

  const segments = findSegments();
  const durations = new Map<string, number>();
  for (const segment of segments) {
    const base = path.basename(segment);
    const prefix = base.replace(/(\.mp4)$/, '').split('_').slice(0, 2).join('_');
    const duration = parseFloat(
      spawnSync(
        'ffprobe',
        ['-v', 'error', '-show_entries', 'format=duration', '-of', 'csv=p=0', segment],
        { encoding: 'utf8', timeout: 10_000 },
      ).stdout?.trim() ?? '0',
    );
    durations.set(prefix, duration);
  }

  const paddedWavs: string[] = [];
  for (const group of SEGMENT_GROUPS) {
    const ssml = buildSegmentSSML(group.phases, SPEECH_VOICE);
    const synthesizer = new sdk.SpeechSynthesizer(speechConfig, null as any);
    const result = await new Promise<any>((resolve, reject) => {
      synthesizer.speakSsmlAsync(
        ssml,
        (res: any) => { synthesizer.close(); resolve(res); },
        (err: any) => { synthesizer.close(); reject(err); },
      );
    });

    if (result.reason !== sdk.ResultReason.SynthesizingAudioCompleted) {
      throw new Error(`TTS failed for ${group.segPrefix}`);
    }

    const rawWav = path.join(OUTPUT_DIR, `_narr_${group.segPrefix}_raw.wav`);
    writeFileSync(rawWav, Buffer.from(result.audioData));

    const videoDuration = durations.get(group.segPrefix) ?? 15;
    const rawDuration = parseFloat(
      spawnSync(
        'ffprobe',
        ['-v', 'error', '-show_entries', 'format=duration', '-of', 'csv=p=0', rawWav],
        { encoding: 'utf8', timeout: 10_000 },
      ).stdout?.trim() ?? '0',
    );

    const paddedWav = path.join(OUTPUT_DIR, `_narr_${group.segPrefix}.wav`);
    const tempo = rawDuration > videoDuration + 0.5
      ? Math.min(2.0, rawDuration / videoDuration)
      : 1.0;
    const af = tempo > 1.0
      ? `atempo=${tempo.toFixed(4)},apad=whole_dur=${videoDuration.toFixed(3)}`
      : `apad=whole_dur=${videoDuration.toFixed(3)}`;

    const padResult = spawnSync(
      'ffmpeg',
      ['-y', '-i', rawWav, '-af', af, '-ar', '16000', '-ac', '1', paddedWav],
      { encoding: 'utf8', timeout: 30_000 },
    );
    if (padResult.status !== 0) {
      throw new Error(`ffmpeg audio fit failed for ${group.segPrefix}`);
    }

    paddedWavs.push(paddedWav);
    try { unlinkSync(rawWav); } catch { /* ignore */ }
  }

  const audioList = path.join(OUTPUT_DIR, '_narr_concat.txt');
  writeFileSync(
    audioList,
    paddedWavs.map((file) => `file '${file.replace(/\\/g, '/')}'`).join('\n'),
    'utf8',
  );

  const concatAudioResult = spawnSync(
    'ffmpeg',
    ['-y', '-f', 'concat', '-safe', '0', '-i', audioList, '-c:a', 'pcm_s16le', '-ar', '16000', '-ac', '1', NARRATION_PATH],
    { encoding: 'utf8', timeout: 30_000 },
  );

  for (const file of paddedWavs) {
    try { unlinkSync(file); } catch { /* ignore */ }
  }
  try { unlinkSync(audioList); } catch { /* ignore */ }

  if (concatAudioResult.status !== 0) {
    throw new Error(`ffmpeg narration concat failed: ${concatAudioResult.stderr?.substring(0, 400)}`);
  }

  const sizeKB = Math.round(statSync(NARRATION_PATH).size / 1024);
  console.log(`[✓] Narration: ${NARRATION_PATH} (${sizeKB} KB)\n`);
}

function mergeAudioVideo(): void {
  if (DEMO_FLAGS.DRY_RUN) {
    console.log('--- Final merge skipped in dry run mode ---\n');
    return;
  }

  if (!DEMO_FLAGS.MERGE_VIDEO) {
    console.log('--- Final merge skipped ---\n');
    return;
  }

  if (!existsSync(RECORDING_PATH)) {
    throw new Error(`Silent recording missing: ${RECORDING_PATH}`);
  }
  if (!existsSync(NARRATION_PATH)) {
    throw new Error(`Narration missing: ${NARRATION_PATH}`);
  }

  console.log('--- Merging narration + video ---');
  const result = spawnSync(
    'ffmpeg',
    [
      '-y',
      '-i', RECORDING_PATH,
      '-i', NARRATION_PATH,
      '-c:v', 'libx264',
      '-preset', 'medium',
      '-crf', '18',
      '-pix_fmt', 'yuv420p',
      '-c:a', 'aac',
      '-b:a', '192k',
      '-map', '0:v:0',
      '-map', '1:a:0',
      '-shortest',
      '-movflags', '+faststart',
      FINAL_VIDEO_PATH,
    ],
    { encoding: 'utf8', timeout: 600_000 },
  );

  if (result.status !== 0) {
    throw new Error(`ffmpeg merge failed: ${result.stderr?.substring(0, 400)}`);
  }

  if (!isValidMp4(FINAL_VIDEO_PATH)) {
    throw new Error(`Final video invalid: ${FINAL_VIDEO_PATH}`);
  }

  const sizeKB = Math.round(statSync(FINAL_VIDEO_PATH).size / 1024);
  console.log(`[✓] Final video: ${FINAL_VIDEO_PATH} (${sizeKB} KB)\n`);
}

async function maybeRecordWindowByTitle(
  cam: CandycamClient,
  filename: string,
  titleSubstring: string,
): Promise<boolean> {
  if (!titleSubstring.trim()) {
    console.log(`  recording skipped for ${filename}: no window title configured`);
    return false;
  }
  await cam.recordWindow(filename, titleSubstring);
  return true;
}

async function runTerminalPhase(cam: CandycamClient, doc: TimelineDocument): Promise<void> {
  if (!DEMO_FLAGS.RUN_TERMINAL) return;
  const steps = buildTerminalSteps();

  const recordedSetup = await maybeRecordWindowByTitle(cam, SEGMENTS.SETUP_SCAN, WINDOW_TITLES.TERMINAL);
  for (const step of steps.slice(0, 4)) {
    executeTimedStep(doc, {
      label: step.label,
      segment: SEGMENTS.SETUP_SCAN,
      notes: step.notes,
      command: step.command,
    }, () => runTerminalStep(step));
  }
  if (recordedSetup) await cam.stopSegment();

  const recordedCli = await maybeRecordWindowByTitle(cam, SEGMENTS.CLI, WINDOW_TITLES.TERMINAL);
  for (const step of steps.slice(4)) {
    executeTimedStep(doc, {
      label: step.label,
      segment: SEGMENTS.CLI,
      notes: step.notes,
      command: step.command,
    }, () => runTerminalStep(step));
  }
  if (recordedCli) await cam.stopSegment();
}

async function runVSCodePhase(cam: CandycamClient, doc: TimelineDocument): Promise<void> {
  if (!DEMO_FLAGS.RUN_VSCODE) return;
  const steps = buildVSCodeSteps();

  const recordedQuery = await maybeRecordWindowByTitle(cam, SEGMENTS.VSCODE_QUERY, WINDOW_TITLES.VSCODE);
  executeTimedStep(doc, {
    label: steps[0].label,
    segment: SEGMENTS.VSCODE_QUERY,
    notes: steps[0].notes,
    command: `cd ${VSCODE_EXTENSION_DIR} && ${steps[0].command}`,
  }, () => runVSCodeStep(steps[0]));
  if (recordedQuery) await cam.stopSegment();

  const recordedGraph = await maybeRecordWindowByTitle(cam, SEGMENTS.VSCODE_GRAPH, WINDOW_TITLES.VSCODE);
  executeTimedStep(doc, {
    label: steps[1].label,
    segment: SEGMENTS.VSCODE_GRAPH,
    notes: steps[1].notes,
    command: `cd ${VSCODE_EXTENSION_DIR} && ${steps[1].command}`,
  }, () => runVSCodeStep(steps[1]));
  if (recordedGraph) await cam.stopSegment();
}

async function runAgentPhase(cam: CandycamClient, doc: TimelineDocument): Promise<void> {
  const agentPlan = emitAgentPlan();
  console.log(`[✓] Agent scene plan: ${agentPlan.jsonPath}`);
  console.log(`[✓] Agent initial prompt: ${agentPlan.promptPath}`);
  console.log(`[✓] Agent launch script: ${agentPlan.launchScriptPath}`);
  if (!DEMO_FLAGS.RUN_AGENT) return;
  const recorded = await maybeRecordWindowByTitle(cam, SEGMENTS.AGENT, WINDOW_TITLES.AGENT);
  await executeTimedAsyncStep(doc, {
    label: 'Agent and conclusion scene',
    segment: SEGMENTS.AGENT,
    notes: 'Launch the system terminal, start Copilot, and seed the prepared first prompt.',
    command: agentPlan.launchCommand,
  }, async () => {
    console.log(`\n=== Agent scene placeholder ===`);
    console.log(`Launch command: ${agentPlan.launchCommand}`);
    console.log(`Agent command: ${AGENT_SCENE_PLAN.agentCommand}`);
    console.log(`Seed prompt from: ${agentPlan.promptPath}`);
    const launchResult = spawnSync('/bin/bash', ['-lc', agentPlan.launchCommand], {
      encoding: 'utf8',
      stdio: 'inherit',
    });
    if (launchResult.status !== 0) {
      throw new Error(`Failed to launch agent terminal (${launchResult.status})`);
    }

    const deadline = Date.now() + AGENT_SCENE_TIMEOUT_MS;
    while (!existsSync(agentPlan.statusPath)) {
      if (Date.now() > deadline) {
        throw new Error(`Timed out waiting for agent status file: ${agentPlan.statusPath}`);
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    const status = JSON.parse(readFileSync(agentPlan.statusPath, 'utf8')) as {
      durationMs: number;
      exitCode: number;
      transcriptPath?: string;
    };
    console.log(`Agent scene runtime: ${formatDuration(status.durationMs)}`);
    if (status.transcriptPath) {
      console.log(`Agent transcript: ${status.transcriptPath}`);
    }
    if (status.exitCode !== 0) {
      throw new Error(`Agent scene failed with exit code ${status.exitCode}`);
    }
  });
  if (recorded) await cam.stopSegment();
}

function printTimingSummary(doc: TimelineDocument): void {
  console.log('Timing summary:');
  if (doc.executedSteps.length === 0) {
    console.log('  (no executed steps)');
    console.log('');
    return;
  }

  let currentSegment = '';
  for (const step of doc.executedSteps) {
    if (step.segment !== currentSegment) {
      currentSegment = step.segment;
      console.log(`  ${currentSegment}`);
    }
    console.log(`    ${step.label}: ${formatDuration(step.durationMs)} [${step.status}]`);
  }
  console.log('');
}

async function main(): Promise<void> {
  preflight();
  cleanOutput();
  const timeline = createTimelineDocument();
  emitTimeline(timeline);
  synthesizeNarrationPlan();
  summarizeExpectedArtifacts();
  const cam = new CandycamClient();
  await cam.start();
  try {
    await runTerminalPhase(cam, timeline);
    await runVSCodePhase(cam, timeline);
    await runAgentPhase(cam, timeline);
  } finally {
    await cam.quit();
  }
  concatSegments();
  await synthesizeNarration();
  mergeAudioVideo();
  summarizeCurrentOutput();
  printTimingSummary(timeline);

  console.log('Next implementation steps:');
  console.log('  1. Record the six defuddle segments with their final output filenames');
  console.log('  2. Feed those segment durations into the per-segment narration fit pass');
  console.log('  3. Run concat, TTS, and final mux on the real segment set');
  console.log(
    `  4. Mode: dryRun=${DEMO_FLAGS.DRY_RUN} narration=${DEMO_FLAGS.SYNTH_NARRATION} merge=${DEMO_FLAGS.MERGE_VIDEO}`,
  );
}

main().catch((error) => {
  console.error('\n[orchestrate] FAILED');
  console.error(error instanceof Error ? error.stack ?? error.message : String(error));
  process.exit(1);
});
