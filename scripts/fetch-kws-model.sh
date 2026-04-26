#!/usr/bin/env bash
# Fetch the sherpa-onnx KWS model used by CLX's wake-word listener and
# generate a default keywords.txt with "Hey CLX" as the wake phrase.
#
# Usage:
#   ./scripts/fetch-kws-model.sh                # default phrase "hey clx"
#   ./scripts/fetch-kws-model.sh "hey clx" "ok computer"
#
# Outputs:
#   $MODEL_DIR/         — sherpa KWS model (encoder/decoder/joiner + tokens)
#   $KEYWORDS_FILE      — BPE-encoded keywords.txt (one phrase per line)
#
# After running, set in CLX Preferences (or env):
#   wake_word_enabled        = true
#   wake_word_model_dir      = $MODEL_DIR
#   wake_word_keywords_file  = $KEYWORDS_FILE

set -euo pipefail

CACHE="${OTOJI_CACHE_DIR:-$HOME/.cache/otoji}"
MODEL_NAME="sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01"
MODEL_URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/kws-models/${MODEL_NAME}.tar.bz2"
MODEL_DIR="$CACHE/$MODEL_NAME"
CFG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/clx"
KEYWORDS_FILE="$CFG_DIR/kws-keywords.txt"

mkdir -p "$CACHE" "$CFG_DIR"

# ── 1. Download model if missing ────────────────────────────────────────
if [ ! -d "$MODEL_DIR" ]; then
    echo "[kws] downloading $MODEL_NAME (~3.3MB)"
    cd "$CACHE"
    curl -fL "$MODEL_URL" | tar -xj
    if [ ! -d "$MODEL_DIR" ]; then
        echo "[kws] error: extraction did not produce $MODEL_DIR" >&2
        exit 1
    fi
    echo "[kws] model installed at $MODEL_DIR"
else
    echo "[kws] model already present at $MODEL_DIR (skip download)"
fi

# ── 2. Build keywords.txt ────────────────────────────────────────────────
# The model ships a `keywords.txt` with common phrases (HEY SIRI, ALEXA, …).
# For instant try-out we copy that and optionally append custom phrases
# tokenized via sherpa_onnx.utils.text2token (needs `sentencepiece`).
if [ "$#" -eq 0 ]; then
    PHRASES=("hey clx")
else
    PHRASES=("$@")
fi

# Start from the bundled keywords (HEY SIRI etc.) so the user can test
# immediately by saying any of those before custom phrases are added.
cp "$MODEL_DIR/keywords.txt" "$KEYWORDS_FILE"

# Encode user-supplied phrases via sherpa_onnx.utils.text2token. The wheel
# only declares sentencepiece + pypinyin as soft deps — install on demand.
NEED_PIP=()
python3 -c "import sentencepiece" 2>/dev/null || NEED_PIP+=(sentencepiece)
python3 -c "import pypinyin"     2>/dev/null || NEED_PIP+=(pypinyin)
if [ "${#NEED_PIP[@]}" -gt 0 ]; then
    echo "[kws] installing python deps for BPE encoding: ${NEED_PIP[*]}"
    pip3 install --quiet "${NEED_PIP[@]}" || {
        echo "[kws] WARNING: pip install failed; custom phrases skipped" >&2
        PHRASES=()
    }
fi

if [ "${#PHRASES[@]}" -gt 0 ]; then
    TMP_OUT="$(mktemp)"
    UPPER=()
    for p in "${PHRASES[@]}"; do
        UPPER+=("$(echo "$p" | tr '[:lower:]' '[:upper:]')")
    done
    if PYTHONIOENCODING=utf-8 python3 - "$MODEL_DIR" "$TMP_OUT" "${UPPER[@]}" <<'PY'
import sys, sherpa_onnx.utils as u
model_dir, out_path, *texts = sys.argv[1:]
rows = u.text2token(
    texts=texts,
    tokens=f"{model_dir}/tokens.txt",
    tokens_type="bpe",
    bpe_model=f"{model_dir}/bpe.model",
)
with open(out_path, "w", encoding="utf-8") as f:
    for row in rows:
        f.write(" ".join(row) + "\n")
PY
    then
        echo "" >> "$KEYWORDS_FILE"
        echo "# --- custom phrases (added by fetch-kws-model.sh) ---" >> "$KEYWORDS_FILE"
        cat "$TMP_OUT" >> "$KEYWORDS_FILE"
        echo "[kws] appended ${#PHRASES[@]} custom phrase(s) to $KEYWORDS_FILE"
    else
        echo "[kws] WARNING: text2token failed; only bundled phrases active" >&2
    fi
    rm -f "$TMP_OUT"
fi

echo "[kws] keywords file: $KEYWORDS_FILE"
echo "[kws] active phrases:"
grep -v '^#' "$KEYWORDS_FILE" | grep -v '^$' | sed 's/^/    /'

# ── 4. Print env-var setup hint ──────────────────────────────────────────
cat <<EOF

[kws] done.

Quick test (env vars override prefs):
    export OTOJI_KWS_DIR='$MODEL_DIR'
    export OTOJI_KWS_KEYWORDS='$KEYWORDS_FILE'
    ./build.sh

Or open CLX preferences (Cmd+,) → Wake Word and paste:
    Model directory:  $MODEL_DIR
    Keywords file:    $KEYWORDS_FILE
EOF
