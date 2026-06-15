#!/usr/bin/env python3
"""Accuracy × cost matrix benchmark for STT models over recorded PTT audio.

Replays the PTT recordings saved by otoji (`<stem>.wav` in the notes store)
through several ASR models — SenseVoice variants via `otoji transcribe`, and
Whisper variants via the standalone `sherpa-onnx-offline` binary (otoji's
transcribe path is SenseVoice-only) — and reports, per model:

    mean latency (cost)   ·   mean CER/WER (accuracy, if references given)   ·
    model size on disk    ·   a cost-adjusted ranking

so you can pick the best accuracy-per-cost combination for your voice.

Usage:
    scripts/stt-matrix-bench.py --limit 20
    scripts/stt-matrix-bench.py --models int8-2024 whisper-turbo whisper-large-v3
    scripts/stt-matrix-bench.py --reference refs.json     # enables CER/WER + ranking

`refs.json` = { "<stem>": "ground truth text", ... }. Without it you still get
the latency/size matrix and side-by-side transcripts (accuracy columns blank).

Whisper needs the standalone sherpa-onnx binary; point to it with:
    --sherpa-offline ~/work/sensevoice-bench/sherpa-onnx-v1.13.2-osx-arm64-shared
(default tries that path).
"""
from __future__ import annotations
import argparse, json, os, subprocess, sys, time
from pathlib import Path

OTOJI_DIR = Path(os.path.expanduser("~/Library/Application Support/otoji"))
MODEL_CACHE = Path(os.path.expanduser("~/.cache/otoji"))
DEFAULT_SHERPA = Path(os.path.expanduser(
    "~/work/sensevoice-bench/sherpa-onnx-v1.13.2-osx-arm64-shared"))

# alias -> model cache dir name. kind is inferred from the directory contents.
MODEL_ALIASES = {
    "int8-2024":        "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17",
    "int8-2025":        "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09",
    "2024":             "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17",
    "2025":             "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2025-09-09",
    "nano-int8":        "sherpa-onnx-sense-voice-funasr-nano-int8-2025-12-17",
    "whisper-turbo":    "sherpa-onnx-whisper-turbo",
    "whisper-large-v3": "sherpa-onnx-whisper-large-v3",
    # whisper.cpp (Metal-accelerated, quantized) — an "optimized whisper".
    # Value is an absolute .bin path, detected as kind "whispercpp".
    "whispercpp-turbo": "/opt/homebrew/share/whisper-cpp/ggml-large-v3-turbo-q5_0.bin",
}
DEFAULT_MODELS = ["int8-2024", "whisper-turbo", "whispercpp-turbo"]


def model_kind(d: Path) -> str:
    """sensevoice / whisper (sherpa) / whispercpp, inferred from the path."""
    if d.is_file() and d.suffix == ".bin":
        return "whispercpp"
    names = {p.name for p in d.iterdir()} if d.is_dir() else set()
    if any("encoder" in n and "whisper" not in n.lower() for n in names) and \
       any(n.endswith("-encoder.int8.onnx") for n in names):
        return "whisper"
    if any("encoder" in n for n in names) and any("decoder" in n for n in names):
        return "whisper"
    return "sensevoice"


def dir_size_mb(d: Path) -> float:
    if d.is_file():
        return d.stat().st_size / 1e6
    total = sum(p.stat().st_size for p in d.rglob("*") if p.is_file())
    return total / 1e6


def run_whispercpp(wav: str, model_bin: Path) -> tuple[str, float]:
    t0 = time.monotonic()
    try:
        out = subprocess.run(["whisper-cli", "-m", str(model_bin), "-f", wav, "-nt", "-np"],
                             capture_output=True, text=True, timeout=300)
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        return f"<error: {e}>", time.monotonic() - t0
    return out.stdout.strip().replace("\n", " "), time.monotonic() - t0


def load_segments(limit: int, kinds: set[str]) -> list[dict]:
    jsonl = OTOJI_DIR / "notes.jsonl"
    if not jsonl.exists():
        sys.exit(f"no notes store at {jsonl}")
    rows = []
    for line in jsonl.read_text().splitlines():
        try:
            n = json.loads(line)
        except ValueError:
            continue
        if n.get("kind") not in kinds:
            continue
        wav = OTOJI_DIR / f"{n['stem']}.wav"
        if wav.exists():
            rows.append({"stem": n["stem"], "wav": str(wav), "note_text": n.get("text", "")})
    rows.reverse()
    return rows[:limit]


def run_sensevoice(wav: str, model_dir: Path) -> tuple[str, float]:
    env = {**os.environ, "OTOJI_REBUILDING": "1", "OTOJI_RELAUNCHED": "1"}
    t0 = time.monotonic()
    try:
        out = subprocess.run(["otoji", "transcribe", wav, "--model", str(model_dir)],
                             capture_output=True, text=True, timeout=180, env=env)
    except subprocess.TimeoutExpired:
        return "<timeout>", time.monotonic() - t0
    dt = time.monotonic() - t0
    for ln in reversed(out.stdout.strip().splitlines()):
        ln = ln.strip()
        if ln.startswith("{"):
            try:
                return json.loads(ln).get("text", "").strip(), dt
            except ValueError:
                break
    return "<no-json>", dt


def run_whisper(wav: str, model_dir: Path, sherpa: Path) -> tuple[str, float]:
    binp = sherpa / "bin" / "sherpa-onnx-offline"
    if not binp.exists():
        return "<no sherpa-onnx-offline>", 0.0
    enc = next(model_dir.glob("*encoder*.onnx"), None)
    dec = next(model_dir.glob("*decoder*.onnx"), None)
    tok = next(model_dir.glob("*tokens*.txt"), None)
    if not (enc and dec and tok):
        return "<whisper files missing>", 0.0
    env = {**os.environ, "DYLD_LIBRARY_PATH": str(sherpa / "lib")}
    t0 = time.monotonic()
    try:
        out = subprocess.run(
            [str(binp), f"--whisper-encoder={enc}", f"--whisper-decoder={dec}",
             f"--tokens={tok}", "--num-threads=4", wav],
            capture_output=True, text=True, timeout=300, env=env)
    except subprocess.TimeoutExpired:
        return "<timeout>", time.monotonic() - t0
    dt = time.monotonic() - t0
    for ln in out.stdout.splitlines() + out.stderr.splitlines():
        ln = ln.strip()
        if ln.startswith("{") and '"text"' in ln:
            try:
                return json.loads(ln).get("text", "").strip(), dt
            except ValueError:
                continue
    return "<no-json>", dt


def is_cjk(s: str) -> bool:
    return any("　" <= c <= "鿿" or "가" <= c <= "힣" for c in s)


def edit_distance(a, b) -> int:
    dp = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        prev, dp[0] = dp[0], i
        for j, cb in enumerate(b, 1):
            prev, dp[j] = dp[j], min(dp[j] + 1, dp[j - 1] + 1, prev + (ca != cb))
    return dp[-1]


def err_rate(hyp: str, ref: str) -> float:
    h = list(hyp) if is_cjk(ref) else hyp.lower().split()
    r = list(ref) if is_cjk(ref) else ref.lower().split()
    if not r:
        return 0.0 if not h else 1.0
    return edit_distance(h, r) / len(r)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--limit", type=int, default=20)
    ap.add_argument("--models", nargs="+", default=DEFAULT_MODELS)
    ap.add_argument("--reference", help="JSON {stem: ground truth}")
    ap.add_argument("--kind", nargs="+", default=["ptt_final", "final"])
    ap.add_argument("--sherpa-offline", default=str(DEFAULT_SHERPA),
                    help="sherpa-onnx shared dir (for whisper)")
    ap.add_argument("--csv", default="/tmp/stt-matrix.csv")
    args = ap.parse_args()

    sherpa = Path(os.path.expanduser(args.sherpa_offline))
    models = {}
    for m in args.models:
        d = MODEL_CACHE / MODEL_ALIASES.get(m, m) if not os.path.isabs(m) else Path(m)
        if not d.exists():
            print(f"skip {m}: dir not found ({d})", file=sys.stderr)
            continue
        models[m] = {"dir": d, "kind": model_kind(d), "size_mb": dir_size_mb(d)}

    refs = json.loads(Path(args.reference).read_text()) if args.reference else {}
    segs = load_segments(args.limit, set(args.kind))
    if not segs:
        sys.exit("no recordings with .wav found")

    print(f"# STT matrix — {len(segs)} segments × {len(models)} models"
          f"{' (scored)' if refs else ' (no refs → latency/size only)'}\n", file=sys.stderr)

    per_seg = []
    agg = {m: {"lat": [], "cer": []} for m in models}
    for seg in segs:
        ref = refs.get(seg["stem"])
        row = {"stem": seg["stem"], "reference": ref or seg["note_text"]}
        for m, info in models.items():
            if info["kind"] == "whisper":
                text, dt = run_whisper(seg["wav"], info["dir"], sherpa)
            elif info["kind"] == "whispercpp":
                text, dt = run_whispercpp(seg["wav"], info["dir"])
            else:
                text, dt = run_sensevoice(seg["wav"], info["dir"])
            row[m] = text
            agg[m]["lat"].append(dt)
            if ref:
                agg[m]["cer"].append(err_rate(text, ref))
            print(f"  {seg['stem'][:19]} {m:16} {dt:5.1f}s  {text[:55]}", file=sys.stderr)
        per_seg.append(row)

    # transcript matrix
    cols = ["stem", "reference"] + list(models)
    print("## Transcripts\n")
    print("| " + " | ".join(cols) + " |")
    print("|" + "|".join(["---"] * len(cols)) + "|")
    for row in per_seg:
        print("| " + " | ".join(str(row.get(c, "")).replace("|", "\\|") for c in cols) + " |")

    # summary matrix
    print("\n## Model matrix (accuracy × cost)\n")
    print("| model | kind | size MB | mean latency | mean err | n |")
    print("|---|---|---|---|---|---|")
    summary = []
    for m, info in models.items():
        lat = sum(agg[m]["lat"]) / len(agg[m]["lat"]) if agg[m]["lat"] else float("nan")
        cer = (sum(agg[m]["cer"]) / len(agg[m]["cer"])) if agg[m]["cer"] else None
        summary.append((m, info, lat, cer))
        cer_s = f"{cer*100:.1f}%" if cer is not None else "—"
        print(f"| {m} | {info['kind']} | {info['size_mb']:.0f} | {lat:.2f}s | {cer_s} | {len(agg[m]['lat'])} |")

    if refs:
        print("\n## Ranking\n")
        scored = [(m, lat, cer) for (m, _i, lat, cer) in summary if cer is not None]
        best_acc = min(scored, key=lambda x: x[2], default=None)
        # cost-adjusted: error penalised, latency penalised (1 pt err ≈ 0.1s).
        best_cp = min(scored, key=lambda x: x[2] * 100 + x[1] * 0.1, default=None)
        if best_acc:
            print(f"- Most accurate: **{best_acc[0]}** (err {best_acc[2]*100:.1f}%, {best_acc[1]:.2f}s)")
        if best_cp:
            print(f"- Best accuracy-per-cost: **{best_cp[0]}** (err {best_cp[2]*100:.1f}%, {best_cp[1]:.2f}s)")

    import csv as csvmod
    with open(args.csv, "w", newline="") as f:
        w = csvmod.DictWriter(f, fieldnames=cols)
        w.writeheader(); w.writerows(per_seg)
    print(f"\nCSV: {args.csv}", file=sys.stderr)


if __name__ == "__main__":
    main()
