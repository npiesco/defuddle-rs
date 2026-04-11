#!/usr/bin/env python3
"""Concat recorded segments, apply speed zones, synthesize TTS narration, mux final video."""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

DEMO_DIR = Path(__file__).resolve().parent
ROOT_DIR = DEMO_DIR.parents[1]
OUTPUT_DIR = ROOT_DIR / "demo" / "output"
TIMINGS_PATH = OUTPUT_DIR / "timings.json"

SEGMENTS = [
    OUTPUT_DIR / "seg_01_cli.mp4",
    OUTPUT_DIR / "seg_02_vscode.mp4",
    OUTPUT_DIR / "seg_03_webapp.mp4",
    OUTPUT_DIR / "seg_04_tauri.mp4",
    OUTPUT_DIR / "seg_05_copilot.mp4",
]

# Speed zones per segment: list of (start_sec, end_sec, factor). end=0 means to source end.
# Populated dynamically using timings.json for the Copilot segment.
CLI_SPEED: list[tuple[float, float, float]] = [
    (0.0, 0.0, 1.5),
]
VSCODE_SPEED: list[tuple[float, float, float]] = [
    (0.0, 0.0, 1.5),
]
WEBAPP_SPEED: list[tuple[float, float, float]] = [
    (0.0, 5.0, 1.0),   # connection screen — show at real speed
    (5.0, 0.0, 2.0),   # graph loading and exploration
]
TAURI_SPEED: list[tuple[float, float, float]] = [
    (0.0, 5.0, 1.0),   # connection screen — show at real speed
    (5.0, 0.0, 2.0),   # graph loading and exploration
]


def load_env() -> None:
    env_path = DEMO_DIR / ".env"
    if not env_path.exists():
        return
    for raw in env_path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        os.environ.setdefault(key.strip(), value.strip())


def run(args: list[str], timeout: int = 600) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, capture_output=True, text=True, timeout=timeout)
    if result.returncode != 0:
        raise RuntimeError(f"Command failed: {' '.join(args[:4])}\nstderr: {result.stderr[:500]}")
    return result


def get_duration(path: Path) -> float:
    result = run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration",
         "-of", "csv=p=0", str(path)],
        timeout=30,
    )
    return float(result.stdout.strip())


def build_copilot_speed_zones(seg_dur: float) -> list[tuple[float, float, float]]:
    timings: dict[str, float] = {}
    if TIMINGS_PATH.is_file():
        timings = json.loads(TIMINGS_PATH.read_text())

    mcp_seen = timings.get("mcp_tool_seen", 15.0)
    demo_done = timings.get("demo_done", seg_dur * 0.85)

    return [
        (0.0, mcp_seen, 2.0),          # prelude: startup → first MCP tool call
        (mcp_seen, demo_done, 1.5),    # agent working (slight speedup for length)
        (demo_done, 0.0, 8.0),         # post-completion idle: fast-forward to end
    ]


def speedup_segment(src: Path, dst: Path, zones: list[tuple[float, float, float]]) -> float:
    """Apply speed zones to a segment, produce silent MP4."""
    src_dur = get_duration(src)

    # Resolve zone end=0 to actual source duration
    resolved: list[tuple[float, float, float]] = []
    for zs, ze, zf in zones:
        if ze == 0.0 or ze > src_dur:
            ze = src_dur
        if ze > zs:
            resolved.append((zs, ze, zf))

    input_args: list[str] = []
    filter_parts: list[str] = []
    for i, (zs, ze, zf) in enumerate(resolved):
        input_args.extend(["-ss", f"{zs:.3f}", "-t", f"{ze - zs:.3f}", "-i", str(src)])
        if zf == 1.0:
            filter_parts.append(f"[{i}:v]setpts=PTS-STARTPTS[z{i}]")
        else:
            filter_parts.append(f"[{i}:v]setpts=(PTS-STARTPTS)/{zf}[z{i}]")

    zone_refs = "".join(f"[z{i}]" for i in range(len(resolved)))
    filter_parts.append(f"{zone_refs}concat=n={len(resolved)}:v=1:a=0[outv]")

    zone_desc = ", ".join(
        f"{zs:.0f}-{ze:.0f}s@{zf:.1f}x" for zs, ze, zf in resolved
    )
    print(f"  zones: {zone_desc}")
    run([
        "ffmpeg", "-y",
        *input_args,
        "-filter_complex", ";".join(filter_parts),
        "-map", "[outv]",
        "-c:v", "libx264", "-preset", "fast", "-crf", "18",
        "-tune", "stillimage", "-pix_fmt", "yuv420p",
        "-an", str(dst),
    ])
    dur = get_duration(dst)
    print(f"  → {dur:.1f}s (source was {src_dur:.1f}s)")
    return dur


def concat_segments(segment_paths: list[Path], dst: Path) -> float:
    """Concat silent MP4 segments using ffmpeg concat demuxer."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        list_path = Path(f.name)
        for p in segment_paths:
            f.write(f"file '{p}'\n")

    run([
        "ffmpeg", "-y",
        "-f", "concat", "-safe", "0", "-i", str(list_path),
        "-c:v", "copy", "-an",
        str(dst),
    ])
    list_path.unlink(missing_ok=True)
    dur = get_duration(dst)
    print(f"[concat] {dur:.1f}s total from {len(segment_paths)} segments")
    return dur


def synthesize_narration(target_seconds: float, out_path: Path) -> float:
    from narration import NARRATION_TEXT  # type: ignore[import-not-found]
    import azure.cognitiveservices.speech as speechsdk

    speech_key = os.environ.get("AZURE_SPEECH_KEY") or os.environ.get("FOUNDRY_API_KEY")
    speech_region = os.environ.get("AZURE_SPEECH_REGION") or os.environ.get("FOUNDRY_REGION")
    speech_voice = os.environ.get("FSGDB_DEMO_VOICE", "en-US-AndrewMultilingualNeural")

    if not speech_key or not speech_region:
        raise RuntimeError("AZURE_SPEECH_KEY/FOUNDRY_API_KEY and AZURE_SPEECH_REGION/FOUNDRY_REGION required")

    speech_config = speechsdk.SpeechConfig(subscription=speech_key, region=speech_region)
    speech_config.speech_synthesis_voice_name = speech_voice
    speech_config.set_speech_synthesis_output_format(
        speechsdk.SpeechSynthesisOutputFormat.Riff16Khz16BitMonoPcm,
    )

    def synth(text: str, path: Path) -> float:
        audio_config = speechsdk.audio.AudioOutputConfig(filename=str(path))
        synthesizer = speechsdk.SpeechSynthesizer(
            speech_config=speech_config, audio_config=audio_config,
        )
        result = synthesizer.speak_text_async(text).get()
        if result.reason != speechsdk.ResultReason.SynthesizingAudioCompleted:
            raise RuntimeError(f"TTS failed: {result.reason}")
        return get_duration(path)

    # Calibrate: measure words/sec
    cal_path = OUTPUT_DIR / "_cal.wav"
    cal_dur = synth(NARRATION_TEXT, cal_path)
    cal_path.unlink(missing_ok=True)

    words = NARRATION_TEXT.split()
    wps = len(words) / cal_dur
    print(f"[audio] calibration: {len(words)} words in {cal_dur:.1f}s = {wps:.2f} wps")

    target_words = int(target_seconds * wps)
    print(f"[audio] target: {target_seconds:.1f}s × {wps:.2f} wps = {target_words} words")

    if target_words <= len(words):
        final_text = " ".join(words[:target_words])
    else:
        repeats = (target_words // len(words)) + 1
        final_text = " ".join((words * repeats)[:target_words])

    dur = synth(final_text, out_path)
    print(f"[audio] narration: {dur:.1f}s (target was {target_seconds:.1f}s)")
    return dur


def mux_final(video_path: Path, audio_path: Path, final_path: Path) -> None:
    run([
        "ffmpeg", "-y",
        "-i", str(video_path),
        "-i", str(audio_path),
        "-c:v", "libx264", "-preset", "medium", "-crf", "18",
        "-tune", "stillimage", "-pix_fmt", "yuv420p",
        "-c:a", "aac", "-b:a", "192k",
        "-map", "0:v:0", "-map", "1:a:0",
        "-shortest", "-movflags", "+faststart",
        str(final_path),
    ])
    audio_check = run(
        ["ffprobe", "-v", "error", "-select_streams", "a",
         "-show_entries", "stream=codec_type", "-of", "csv=p=0", str(final_path)],
        timeout=20,
    )
    if not audio_check.stdout.strip():
        raise RuntimeError(f"Final video has no audio stream: {final_path}")
    size_mb = final_path.stat().st_size / (1024 * 1024)
    print(f"[mux] final: {final_path.name} ({size_mb:.1f} MB)")


def main() -> int:
    load_env()
    sys.path.insert(0, str(DEMO_DIR))

    for seg in SEGMENTS:
        if not seg.is_file():
            print(f"Missing segment: {seg}", file=sys.stderr)
            return 1

    copilot_dur = get_duration(SEGMENTS[4])
    copilot_zones = build_copilot_speed_zones(copilot_dur)

    sped: list[Path] = []
    for i, (seg, zones) in enumerate(
        zip(SEGMENTS, [CLI_SPEED, VSCODE_SPEED, WEBAPP_SPEED, TAURI_SPEED, copilot_zones]), 1
    ):
        dst = OUTPUT_DIR / f"_sped_{i:02d}.mp4"
        print(f"[video] segment {i} ({seg.name}):")
        speedup_segment(seg, dst, zones)
        sped.append(dst)

    combined_path = OUTPUT_DIR / "_combined.mp4"
    total_dur = concat_segments(sped, combined_path)

    narration_path = OUTPUT_DIR / "_narration.wav"
    synthesize_narration(total_dur, narration_path)

    final_path = OUTPUT_DIR / "fsgdb_demo_final.mp4"
    mux_final(combined_path, narration_path, final_path)

    for p in [*sped, combined_path, narration_path]:
        p.unlink(missing_ok=True)

    final_dur = get_duration(final_path)
    print(f"\n[DONE] {final_dur:.0f}s, {final_path.stat().st_size / (1024*1024):.1f} MB")
    print(f"Final video: {final_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
