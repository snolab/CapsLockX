#!/usr/bin/env python3
"""Re-run recorded PTT voice segments through multiple ASR models and compare.

Since 2026-06, otoji saves every push-to-talk segment as a `<stem>.wav` next to
its transcript in the otoji notes store (`~/Library/Application Support/otoji/`).
This script collects those recordings and replays each one through several
SenseVoice model variants via `otoji transcribe`, so the *same* spoken audio can
be compared across models offline — the loop you can't run live.

Usage:
    scripts/ptt-model-bench.py                 # newest 20 PTT recordings, default models
    scripts/ptt-model-bench.py --limit 50
    scripts/ptt-model-bench.py --models int8-2024 int8-2025 2024 2025
    scripts/ptt-model-bench.py --reference refs.json   # CER vs hand-corrected refs

`--reference refs.json` is `{ "<stem>": "ground truth text", ... }`. When given,
the script reports CER (CJK) / WER (latin) against it; otherwise it just prints
the transcripts side by side for eyeballing (the stored note text — itself a
model output — is shown as the baseline column).

Output: a markdown table on stdout plus a CSV next to it (--csv path).
"""
from __future__ import annotations
import argparse, json, os, subprocess, sys, csv as csvmod
from pathlib import Path

OTOJI_DIR = Path(os.path.expanduser("~/Library/Application Support/otoji"))
MODEL_CACHE = Path(os.path.expanduser("~/.cache/otoji"))

# Short alias -> sherpa model directory name. Extend freely.
MODEL_ALIASES = {
    "int8-2024": "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17",
    "int8-2025": "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09",
    "2024":      "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17",
    "2025":      "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2025-09-09",
    "nano-int8": "sherpa-onnx-sense-voice-funasr-nano-int8-2025-12-17",
}
DEFAULT_MODELS = ["int8-2024", "int8-2025", "2024"]


def load_ptt_segments(limit: int, kinds: set[str]) -> list[dict]:
    """Newest-first voice segments (of the given kinds) that have a .wav sibling."""
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
    rows.reverse()  # notes.jsonl is append order (oldest first) -> newest first
    return rows[:limit]


def transcribe(wav: str, model_dir: Path) -> str:
    try:
        out = subprocess.run(
            ["otoji", "transcribe", wav, "--model", str(model_dir)],
            capture_output=True, text=True, timeout=120,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        return f"<error: {e}>"
    line = out.stdout.strip().splitlines()[-1] if out.stdout.strip() else ""
    try:
        return json.loads(line).get("text", "")
    except ValueError:
        return f"<no-json: {out.stderr.strip()[:60]}>"


def is_cjk(s: str) -> bool:
    return any("　" <= c <= "鿿" or "가" <= c <= "힣" for c in s)


def edit_distance(a: list[str], b: list[str]) -> int:
    dp = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        prev, dp[0] = dp[0], i
        for j, cb in enumerate(b, 1):
            prev, dp[j] = dp[j], min(dp[j] + 1, dp[j - 1] + 1, prev + (ca != cb))
    return dp[-1]


def err_rate(hyp: str, ref: str) -> float:
    """CER for CJK references, WER otherwise. 0..1 (or >1 with many insertions)."""
    h = list(hyp) if is_cjk(ref) else hyp.split()
    r = list(ref) if is_cjk(ref) else ref.split()
    if not r:
        return 0.0 if not h else 1.0
    return edit_distance(h, r) / len(r)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--limit", type=int, default=20, help="how many newest PTT recordings (default 20)")
    ap.add_argument("--models", nargs="+", default=DEFAULT_MODELS,
                    help=f"model aliases or absolute dirs (default: {DEFAULT_MODELS}). aliases: {list(MODEL_ALIASES)}")
    ap.add_argument("--reference", help="JSON {stem: ground-truth text} for CER/WER scoring")
    ap.add_argument("--kind", nargs="+", default=["ptt_final", "final"],
                    help="note kinds to include (default: ptt_final final)")
    ap.add_argument("--csv", default="/tmp/ptt-model-bench.csv", help="CSV output path")
    args = ap.parse_args()

    models = {}
    for m in args.models:
        d = MODEL_CACHE / MODEL_ALIASES.get(m, m) if not os.path.isabs(m) else Path(m)
        if not d.exists():
            sys.exit(f"model dir not found: {d}")
        models[m] = d

    refs = {}
    if args.reference:
        refs = json.loads(Path(args.reference).read_text())

    segs = load_ptt_segments(args.limit, set(args.kind))
    if not segs:
        sys.exit("no recordings with .wav found yet — do a few Space+V dictations first")
    print(f"# PTT model benchmark — {len(segs)} segments × {len(models)} models\n", file=sys.stderr)

    # header
    cols = ["stem"] + (["reference"] if refs else ["note_text"]) + list(models)
    rows_out = []
    totals = {m: [] for m in models}

    for seg in segs:
        ref = refs.get(seg["stem"], seg["note_text"])
        row = {"stem": seg["stem"], "reference" if refs else "note_text": ref}
        for m, d in models.items():
            hyp = transcribe(seg["wav"], d)
            row[m] = hyp
            if refs:
                totals[m].append(err_rate(hyp, ref))
            print(f"  {seg['stem'][:19]}  {m:10}  {hyp[:60]}", file=sys.stderr)
        rows_out.append(row)

    # markdown table
    print("| " + " | ".join(cols) + " |")
    print("|" + "|".join(["---"] * len(cols)) + "|")
    for row in rows_out:
        print("| " + " | ".join(str(row.get(c, "")).replace("|", "\\|") for c in cols) + " |")

    if refs:
        print("\n## Mean error rate (CER cjk / WER latin), lower = better\n")
        print("| model | mean err | n |")
        print("|---|---|---|")
        for m in models:
            vals = totals[m]
            mean = sum(vals) / len(vals) if vals else float("nan")
            print(f"| {m} | {mean*100:.1f}% | {len(vals)} |")

    with open(args.csv, "w", newline="") as f:
        w = csvmod.DictWriter(f, fieldnames=cols)
        w.writeheader()
        w.writerows(rows_out)
    print(f"\nCSV written to {args.csv}", file=sys.stderr)


if __name__ == "__main__":
    main()
