#!/usr/bin/env python3
"""Concat defuddle demo segments, synthesize one continuous narration, mux final video.

Pattern follows kagmus: concat the silent segments as-is into one video, generate ONE
TTS over the full narration text, then slow the video (if needed) to match audio length
and mux. No per-segment speedups, no per-segment TTS — that's what produced the dead-air
gaps in the previous build.
"""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

DEMO_DIR = Path(__file__).resolve().parent.parent
SCRIPTS_DIR = Path(__file__).resolve().parent
OUTPUT_DIR = DEMO_DIR / "output"
RECORDING_PATH = OUTPUT_DIR / "defuddle_demo_silent.mp4"
NARRATION_PATH = OUTPUT_DIR / "narration.wav"
FINAL_VIDEO_PATH = OUTPUT_DIR / "defuddle_demo_final.mp4"


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


def find_segments() -> list[Path]:
    sys.path.insert(0, str(SCRIPTS_DIR))
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


def _probe_dimensions(path: Path) -> tuple[int, int]:
    result = run(
        [
            "ffprobe", "-v", "error", "-select_streams", "v:0",
            "-show_entries", "stream=width,height", "-of", "csv=p=0:s=x",
            str(path),
        ],
        timeout=30,
    )
    w, h = result.stdout.strip().split("x")
    return int(w), int(h)


def concat_segments(segments: list[Path], dst: Path) -> float:
    """Concatenate segments onto a common canvas using the concat *filter*.

    The demo segments are recorded at different resolutions (terminals, browser,
    VS Code). Using the concat demuxer with ``-c copy`` forces every segment to
    inherit the first segment's stream dimensions, which causes the player to
    stretch/squash later segments and warps the text. Instead, scale each
    segment to fit inside a shared canvas while preserving aspect ratio, then
    pad with black bars. This guarantees no warping.
    """
    dims = [_probe_dimensions(seg) for seg in segments]
    target_w = max(w for w, _ in dims)
    target_h = max(h for _, h in dims)
    target_w += target_w % 2
    target_h += target_h % 2

    cmd: list[str] = ["ffmpeg", "-y"]
    for seg in segments:
        cmd += ["-i", str(seg)]

    filter_parts: list[str] = []
    labels: list[str] = []
    for idx in range(len(segments)):
        label = f"v{idx}"
        filter_parts.append(
            f"[{idx}:v]scale={target_w}:{target_h}:"
            f"force_original_aspect_ratio=decrease:flags=lanczos,"
            f"pad={target_w}:{target_h}:(ow-iw)/2:(oh-ih)/2:color=black,"
            f"setsar=1,fps=30,format=yuv420p[{label}]"
        )
        labels.append(f"[{label}]")
    filter_parts.append(
        f"{''.join(labels)}concat=n={len(segments)}:v=1:a=0[outv]"
    )
    filter_complex = ";".join(filter_parts)

    cmd += [
        "-filter_complex", filter_complex,
        "-map", "[outv]", "-an",
        "-c:v", "libx264", "-crf", "18", "-preset", "medium",
        "-pix_fmt", "yuv420p", "-movflags", "+faststart",
        str(dst),
    ]
    run(cmd, timeout=1800)
    return get_duration(dst)


def synthesize_narration(text: str, out_path: Path) -> float:
    """One TTS pass over the full narration text. Writes a single WAV."""
    import azure.cognitiveservices.speech as speechsdk

    speech_region = (
        os.environ.get("AZURE_SPEECH_REGION")
        or os.environ.get("FOUNDRY_REGION")
        or "eastus2"
    )
    speech_voice = os.environ.get("DEMO_SPEECH_VOICE", "en-US-AndrewMultilingualNeural")

    sp_tenant = os.environ.get("AZURE_SP_TENANT_ID")
    sp_client = os.environ.get("AZURE_SP_CLIENT_ID")
    sp_secret = os.environ.get("AZURE_SP_CLIENT_SECRET")

    if sp_tenant and sp_client and sp_secret:
        from azure.identity import ClientSecretCredential

        credential = ClientSecretCredential(
            tenant_id=sp_tenant,
            client_id=sp_client,
            client_secret=sp_secret,
        )
        token = credential.get_token("https://cognitiveservices.azure.com/.default")
        # Speech SDK requires AAD tokens in the form: aad#<resource_id>#<jwt>
        resource_id = os.environ.get(
            "AZURE_SPEECH_RESOURCE_ID",
            "/subscriptions/8ae535e3-3fd0-4228-b0b6-f1b7b9e82d51"
            "/resourceGroups/posixlake-rg/providers/Microsoft.CognitiveServices"
            "/accounts/posixlake-speech",
        )
        speech_config = speechsdk.SpeechConfig(
            auth_token=f"aad#{resource_id}#{token.token}",
            region=speech_region,
        )
    else:
        speech_key = os.environ.get("AZURE_SPEECH_KEY") or os.environ.get("FOUNDRY_API_KEY")
        if not speech_key:
            raise RuntimeError(
                "Azure Speech credentials missing. Set AZURE_SP_TENANT_ID/"
                "AZURE_SP_CLIENT_ID/AZURE_SP_CLIENT_SECRET (preferred) or "
                "AZURE_SPEECH_KEY in demo/.env."
            )
        speech_config = speechsdk.SpeechConfig(subscription=speech_key, region=speech_region)

    speech_config.speech_synthesis_voice_name = speech_voice
    speech_config.set_speech_synthesis_output_format(
        speechsdk.SpeechSynthesisOutputFormat.Riff16Khz16BitMonoPcm,
    )

    audio_config = speechsdk.audio.AudioOutputConfig(filename=str(out_path))
    synthesizer = speechsdk.SpeechSynthesizer(
        speech_config=speech_config, audio_config=audio_config,
    )
    result = synthesizer.speak_text_async(text).get()
    close_fn = getattr(synthesizer, "close", None)
    if callable(close_fn):
        close_fn()

    if result.reason != speechsdk.ResultReason.SynthesizingAudioCompleted:
        details = ""
        if result.reason == speechsdk.ResultReason.Canceled:
            cancel = getattr(result, "cancellation_details", None)
            if cancel is not None:
                details = f" reason={cancel.reason} error={cancel.error_details}"
        raise RuntimeError(f"TTS failed: {result.reason}{details}")

    return get_duration(out_path)


def merge_final_video(video_path: Path, audio_path: Path, final_path: Path) -> None:
    """Mux audio over video. Preserves the full video length: pads audio with silence
    if narration is shorter; slows video if narration is longer."""
    vid_dur = get_duration(video_path)
    nar_dur = get_duration(audio_path)

    if nar_dur > vid_dur + 1:
        factor = nar_dur / vid_dur
        print(f"[merge] slowing video {factor:.2f}x ({vid_dur:.1f}s → {nar_dur:.1f}s)")
        slowed = OUTPUT_DIR / "_slowed.mp4"
        run([
            "ffmpeg", "-y", "-i", str(video_path),
            "-filter:v", f"setpts={factor:.4f}*PTS",
            "-an", "-c:v", "libx264", "-crf", "18", "-r", "30",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart",
            str(slowed),
        ])
        video_input = slowed
        target_dur = nar_dur
        audio_filter = "anull"
    else:
        pad = vid_dur - nar_dur
        print(f"[merge] video {vid_dur:.1f}s, narration {nar_dur:.1f}s — padding audio with {pad:.1f}s silence")
        video_input = video_path
        target_dur = vid_dur
        audio_filter = f"apad=whole_dur={vid_dur:.3f}"

    run([
        "ffmpeg", "-y",
        "-i", str(video_input),
        "-i", str(audio_path),
        "-filter_complex", f"[1:a]{audio_filter}[aout]",
        "-map", "0:v:0", "-map", "[aout]",
        "-c:v", "copy", "-c:a", "aac", "-b:a", "192k",
        "-t", f"{target_dur:.3f}",
        "-movflags", "+faststart",
        str(final_path),
    ])

    if video_input != video_path:
        video_input.unlink(missing_ok=True)


def main() -> int:
    load_env()
    sys.path.insert(0, str(SCRIPTS_DIR))
    from narration import NARRATION_TEXT  # type: ignore[import-not-found]

    segments = find_segments()
    print(f"[concat] joining {len(segments)} segments")
    silent_dur = concat_segments(segments, RECORDING_PATH)
    print(f"[concat] silent video: {silent_dur:.1f}s → {RECORDING_PATH}")

    print("[tts] synthesizing one continuous narration")
    nar_dur = synthesize_narration(NARRATION_TEXT, NARRATION_PATH)
    print(f"[tts] narration: {nar_dur:.1f}s → {NARRATION_PATH}")

    merge_final_video(RECORDING_PATH, NARRATION_PATH, FINAL_VIDEO_PATH)
    final_dur = get_duration(FINAL_VIDEO_PATH)
    print(f"[done] final: {final_dur:.1f}s → {FINAL_VIDEO_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
