#!/usr/bin/env python3
"""Concat defuddle demo segments, fit narration per segment, and mux final video."""
from __future__ import annotations

import os
import json
import subprocess
import sys
import tempfile
from pathlib import Path

DEMO_DIR = Path(__file__).resolve().parent
ROOT_DIR = DEMO_DIR.parents[1]
OUTPUT_DIR = ROOT_DIR / "demo" / "output"
RECORDING_PATH = OUTPUT_DIR / "defuddle_demo_silent.mp4"
NARRATION_PATH = OUTPUT_DIR / "narration.wav"
FINAL_VIDEO_PATH = OUTPUT_DIR / "defuddle_demo_final.mp4"

SEGMENT_SPEEDS = {
    "seg_01_hook.mp4": [(0.0, 0.0, 1.5)],
    "seg_02_extension.mp4": [(0.0, 6.0, 1.0), (6.0, 0.0, 3.0)],
    "seg_03_rust.mp4": [(0.0, 8.0, 6.0), (8.0, 18.0, 1.5), (18.0, 0.0, 22.0)],
    "seg_04_python.mp4": [(0.0, 8.0, 3.0), (8.0, 0.0, 7.0)],
    "seg_05_mcp.mp4": [(0.0, 33.6, 10.0), (33.6, 47.9, 4.0), (47.9, 207.3, 14.0), (207.3, 0.0, 90.0)],
    "seg_06_close.mp4": [(0.0, 0.0, 2.5)],
}


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
        raise RuntimeError(f"Command failed: {' '.join(args[:6])}\nstderr: {result.stderr[:800]}")
    return result


def get_duration(path: Path) -> float:
    result = run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)],
        timeout=30,
    )
    return float(result.stdout.strip())


def get_dimensions(path: Path) -> tuple[int, int]:
    result = run(
        [
            "ffprobe",
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0:s=x",
            str(path),
        ],
        timeout=30,
    )
    width_s, height_s = result.stdout.strip().split("x", 1)
    return int(width_s), int(height_s)


def find_segments() -> list[Path]:
    from narration import SEGMENT_NARRATION  # type: ignore[import-not-found]

    segments: list[Path] = []
    missing: list[str] = []
    for item in SEGMENT_NARRATION:
        path = OUTPUT_DIR / item["file"]
        if path.is_file():
            segments.append(path)
        else:
            missing.append(item["file"])
    if missing:
        raise RuntimeError(f"Missing expected segments: {', '.join(missing)}")
    return segments


def get_canvas_size(segments: list[Path]) -> tuple[int, int]:
    max_width = 0
    max_height = 0
    for segment in segments:
        width, height = get_dimensions(segment)
        max_width = max(max_width, width)
        max_height = max(max_height, height)
    if max_width % 2:
        max_width += 1
    if max_height % 2:
        max_height += 1
    return max_width, max_height


def build_atempo_filter(tempo: float) -> str:
    if tempo <= 1.0:
        return "anull"
    factors: list[str] = []
    remaining = tempo
    while remaining > 2.0:
        factors.append("atempo=2.0")
        remaining /= 2.0
    factors.append(f"atempo={remaining:.4f}")
    return ",".join(factors)


def speedup_segment(
    src: Path,
    dst: Path,
    zones: list[tuple[float, float, float]],
    canvas_width: int,
    canvas_height: int,
) -> float:
    src_dur = get_duration(src)
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
            filter_parts.append(
                f"[{i}:v]setpts=PTS-STARTPTS,"
                f"scale={canvas_width}:{canvas_height}:force_original_aspect_ratio=decrease,"
                f"pad={canvas_width}:{canvas_height}:(ow-iw)/2:(oh-ih)/2:black,"
                f"setsar=1[z{i}]"
            )
        else:
            filter_parts.append(
                f"[{i}:v]setpts=(PTS-STARTPTS)/{zf},"
                f"scale={canvas_width}:{canvas_height}:force_original_aspect_ratio=decrease,"
                f"pad={canvas_width}:{canvas_height}:(ow-iw)/2:(oh-ih)/2:black,"
                f"setsar=1[z{i}]"
            )

    zone_refs = "".join(f"[z{i}]" for i in range(len(resolved)))
    filter_parts.append(f"{zone_refs}concat=n={len(resolved)}:v=1:a=0[outv]")

    run([
        "ffmpeg", "-y",
        *input_args,
        "-filter_complex", ";".join(filter_parts),
        "-map", "[outv]",
        "-c:v", "libx264", "-preset", "fast", "-crf", "18",
        "-tune", "stillimage", "-pix_fmt", "yuv420p",
        "-an", str(dst),
    ])
    return get_duration(dst)


def concat_media(inputs: list[Path], dst: Path, *, audio: bool) -> float:
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        list_path = Path(f.name)
        for p in inputs:
            f.write(f"file '{p}'\n")

    codec_args = ["-c:a", "pcm_s16le", "-ar", "16000", "-ac", "1"] if audio else ["-c:v", "copy", "-an"]
    run([
        "ffmpeg", "-y",
        "-f", "concat", "-safe", "0", "-i", str(list_path),
        *codec_args,
        str(dst),
    ])
    list_path.unlink(missing_ok=True)
    return get_duration(dst)


def synthesize_text(text: str, out_path: Path) -> float:
    provider = os.environ.get("DEMO_TTS_PROVIDER", "").strip().lower()
    openai_key = os.environ.get("OPENAI_API_KEY")
    if provider == "openai" or (provider == "" and openai_key):
        if not openai_key:
            raise RuntimeError("OPENAI_API_KEY required for OpenAI TTS")
        voice = os.environ.get("DEMO_OPENAI_VOICE", "alloy")
        model = os.environ.get("DEMO_OPENAI_TTS_MODEL", "gpt-4o-mini-tts")
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False, encoding="utf-8") as f:
            payload_path = Path(f.name)
            json.dump(
                {
                    "model": model,
                    "voice": voice,
                    "input": text,
                    "response_format": "wav",
                },
                f,
            )
        result = subprocess.run(
            [
                "curl",
                "-sS",
                "-o",
                str(out_path),
                "-w",
                "%{http_code}",
                "https://api.openai.com/v1/audio/speech",
                "-H",
                f"Authorization: Bearer {openai_key}",
                "-H",
                "Content-Type: application/json",
                "--data-binary",
                f"@{payload_path}",
            ],
            capture_output=True,
            text=True,
            timeout=600,
        )
        payload_path.unlink(missing_ok=True)
        if result.returncode != 0:
            raise RuntimeError(f"OpenAI TTS failed: {result.stderr[:800]}")
        status = result.stdout.strip()
        if status != "200":
            error_text = out_path.read_text(encoding="utf-8", errors="replace")[:800]
            raise RuntimeError(f"OpenAI TTS failed: HTTP {status}: {error_text}")
        return get_duration(out_path)

    import azure.cognitiveservices.speech as speechsdk

    speech_key = os.environ.get("AZURE_SPEECH_KEY") or os.environ.get("FOUNDRY_API_KEY")
    speech_region = os.environ.get("AZURE_SPEECH_REGION") or os.environ.get("FOUNDRY_REGION")
    speech_voice = os.environ.get("DEMO_SPEECH_VOICE", "en-US-AndrewMultilingualNeural")

    if not speech_key or not speech_region:
        raise RuntimeError("AZURE_SPEECH_KEY/FOUNDRY_API_KEY and AZURE_SPEECH_REGION/FOUNDRY_REGION required")

    speech_config = speechsdk.SpeechConfig(subscription=speech_key, region=speech_region)
    speech_config.speech_synthesis_voice_name = speech_voice
    speech_config.set_speech_synthesis_output_format(
        speechsdk.SpeechSynthesisOutputFormat.Riff16Khz16BitMonoPcm,
    )
    audio_config = speechsdk.audio.AudioOutputConfig(filename=str(out_path))
    synthesizer = speechsdk.SpeechSynthesizer(speech_config=speech_config, audio_config=audio_config)
    result = synthesizer.speak_text_async(text).get()
    close_fn = getattr(synthesizer, "close", None)
    if callable(close_fn):
        close_fn()
    if result.reason != speechsdk.ResultReason.SynthesizingAudioCompleted:
        raise RuntimeError(f"TTS failed: {result.reason}")
    return get_duration(out_path)


def fit_narration_to_segment(text: str, target_seconds: float, raw_path: Path, fitted_path: Path) -> float:
    raw_duration = synthesize_text(text, raw_path)
    tempo = max(1.0, raw_duration / max(target_seconds, 0.1))
    af_parts: list[str] = []
    if tempo > 1.0:
        af_parts.append(build_atempo_filter(tempo))
    af_parts.append(f"apad=whole_dur={target_seconds:.3f}")
    run([
        "ffmpeg", "-y",
        "-i", str(raw_path),
        "-af", ",".join(af_parts),
        "-ar", "16000",
        "-ac", "1",
        str(fitted_path),
    ])
    return get_duration(fitted_path)


def trim_existing_narration_segment(
    source_audio: Path,
    start_seconds: float,
    source_duration: float,
    target_seconds: float,
    raw_slice_path: Path,
    trimmed_path: Path,
    fitted_path: Path,
) -> float:
    run([
        "ffmpeg", "-y",
        "-ss", f"{start_seconds:.3f}",
        "-t", f"{source_duration:.3f}",
        "-i", str(source_audio),
        "-ar", "16000",
        "-ac", "1",
        str(raw_slice_path),
    ])
    run([
        "ffmpeg", "-y",
        "-i", str(raw_slice_path),
        "-af", "silenceremove=stop_periods=-1:stop_duration=0.3:stop_threshold=-50dB",
        "-ar", "16000",
        "-ac", "1",
        str(trimmed_path),
    ])
    trimmed_duration = get_duration(trimmed_path)
    tempo = max(1.0, trimmed_duration / max(target_seconds, 0.1))
    af_parts: list[str] = []
    if tempo > 1.0:
        af_parts.append(build_atempo_filter(tempo))
    af_parts.append(f"apad=whole_dur={target_seconds:.3f}")
    run([
        "ffmpeg", "-y",
        "-i", str(trimmed_path),
        "-af", ",".join(af_parts),
        "-ar", "16000",
        "-ac", "1",
        str(fitted_path),
    ])
    return get_duration(fitted_path)


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


def main() -> int:
    load_env()
    sys.path.insert(0, str(DEMO_DIR))
    reuse_existing_narration = os.environ.get("DEFUDDLE_REUSE_NARRATION") == "1"

    from narration import SEGMENT_NARRATION  # type: ignore[import-not-found]

    segments = find_segments()
    canvas_width, canvas_height = get_canvas_size(segments)

    sped: list[Path] = []
    audio_segments: list[Path] = []
    narration_offsets: list[tuple[float, float]] = []
    cumulative = 0.0
    for segment in segments:
        original_duration = get_duration(segment)
        narration_offsets.append((cumulative, original_duration))
        cumulative += original_duration

    for index, segment in enumerate(segments, 1):
        zones = SEGMENT_SPEEDS.get(segment.name, [(0.0, 0.0, 1.0)])
        sped_path = OUTPUT_DIR / f"_sped_{index:02d}.mp4"
        video_duration = speedup_segment(segment, sped_path, zones, canvas_width, canvas_height)
        sped.append(sped_path)

        narration_item = next(item for item in SEGMENT_NARRATION if item["file"] == segment.name)
        raw_wav = OUTPUT_DIR / f"_narr_{index:02d}_raw.wav"
        trimmed_wav = OUTPUT_DIR / f"_narr_{index:02d}_trimmed.wav"
        fitted_wav = OUTPUT_DIR / f"_narr_{index:02d}.wav"

        if reuse_existing_narration:
            if not NARRATION_PATH.is_file():
                raise RuntimeError(f"Existing narration not found: {NARRATION_PATH}")
            start_seconds, source_duration = narration_offsets[index - 1]
            fitted_duration = trim_existing_narration_segment(
                NARRATION_PATH,
                start_seconds,
                source_duration,
                video_duration,
                raw_wav,
                trimmed_wav,
                fitted_wav,
            )
        else:
            fitted_duration = fit_narration_to_segment(
                narration_item["text"],
                video_duration,
                raw_wav,
                fitted_wav,
            )
        raw_wav.unlink(missing_ok=True)
        trimmed_wav.unlink(missing_ok=True)
        audio_segments.append(fitted_wav)
        print(f"[fit] {segment.name}: video={video_duration:.1f}s narration={fitted_duration:.1f}s")

    total_video_duration = concat_media(sped, RECORDING_PATH, audio=False)
    total_audio_duration = concat_media(audio_segments, NARRATION_PATH, audio=True)
    mux_final(RECORDING_PATH, NARRATION_PATH, FINAL_VIDEO_PATH)

    for path in [*sped, *audio_segments]:
        path.unlink(missing_ok=True)

    final_duration = get_duration(FINAL_VIDEO_PATH)
    print(f"[concat] video={total_video_duration:.1f}s audio={total_audio_duration:.1f}s final={final_duration:.1f}s")
    print(f"Final video: {FINAL_VIDEO_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
