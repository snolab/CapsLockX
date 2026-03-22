# Preprocessing vs Native LLM Input

Should we preprocess with YOLO/STT/OCR before the LLM, or let the LLM
handle raw input natively? **Answer: hybrid — preprocess the hot path,
go native for the hard stuff.**

## Decision Matrix

| Signal | Preprocess? | Why |
|---|---|---|
| **Object detection** | **Yes — YOLO** | 100-1000x faster, precise boxes, tracking. 5-30x token savings |
| **OCR (standard fonts)** | **Yes — local OCR** | Fast, reliable, 5-30x token savings |
| **OCR (stylized/game fonts)** | **No — LLM native** | LLMs handle fantasy/pixel fonts far better than Tesseract |
| **Voice commands** | **Depends** | Native audio for lowest latency; local STT for cost/noise |
| **Scene understanding** | **No — LLM native** | Zero-shot, semantic reasoning, context awareness |
| **Feature extraction (CLIP)** | **No** | Cannot feed embeddings to text LLM. Use for retrieval only |

## Object Detection: YOLO vs LLM Vision

### Speed
| | YOLO | LLM Vision |
|---|---|---|
| Per-frame latency | **1-10ms** | 400-2000ms |
| Sustainable FPS | **30-120+** | 1-3 |
| Factor | **100-1000x faster** | |

### Token cost per frame
- YOLO JSON output (5 objects): **~50-80 tokens**
- Raw screenshot to LLM: **~1000-1500 tokens**
- **Savings: 15-30x**

### Quality tradeoffs
- **YOLO wins**: Precise bounding boxes (sub-pixel), no hallucination, persistent object tracking (ByteTrack/BoTSORT), consistent across frames
- **LLM wins**: Zero-shot novel concepts ("the flask next to the minimap"), stylized game art, semantic reasoning ("which enemy is the biggest threat?")
- **YOLO weakness**: Needs fine-tuning for game-specific objects (500-5000 labeled images per class). COCO classes (person, car, etc.) don't cover fantasy game items

### YOLO on Apple Silicon
| Model | M1/M2 (ANE) | M2 Ultra |
|---|---|---|
| YOLOv8-nano | 8-12ms | 3-4ms |
| YOLOv8-small | 15-20ms | 5-7ms |
| YOLOv10-medium | 25-35ms | 8-12ms |

Plenty fast for real-time game perception on Mac.

### Object Tracking
YOLO + ByteTrack gives persistent IDs across frames (~0.5-2ms overhead).
LLMs cannot track objects between frames — each frame is independent.
For "follow that enemy" behavior, **tracking is essential**.

## STT: Local Whisper vs Native Audio API

### Latency
| Pipeline | Total latency |
|---|---|
| Whisper → text → LLM API | **800-2800ms** (two serial steps) |
| Gemini Live native audio | **500-1500ms** (one step) |

Native audio wins by eliminating a serial hop.

### Quality
- Whisper large-v3 WER: ~3-5% (clean speech), ~8-15% (noisy)
- Gemini Live: comparable on clean speech, possibly worse on noisy game audio
- **Text transcription discards tone/emphasis/emotion** — native audio preserves it

### Token cost
- 10s audio → Gemini: ~250 audio tokens
- 10s audio → Whisper text: ~35 text tokens
- **Audio is 7x more tokens**, but absolute cost is small on Flash

### Recommendation
- **Latency-critical** (game commands): use native audio API
- **Cost-critical** (long sessions): use local Whisper + text
- **Noisy environment** (game sounds + voice): local Whisper with VAD preprocessing is more robust

## OCR: Local vs LLM Native

### Speed
| Engine | Latency |
|---|---|
| Apple Vision Framework (ANE) | **30-80ms** |
| PaddleOCR v4 (GPU) | 20-60ms |
| Tesseract 5 (CPU) | 200-800ms |
| LLM from screenshot | 400-2000ms |

### Quality on game text
- **Standard fonts** (system UI, common HUD): Local OCR ~95%+ accuracy, LLM ~95%
- **Stylized fonts** (pixel art, fantasy, gothic): Local OCR ~30-60%, **LLM ~85-90%**
- **Context understanding** ("HP: 45/100" = health is low): Only LLM understands semantics

### Recommendation
- Known HUD with standard fonts → **local OCR** (fast, free, precise)
- Stylized/unknown text → **LLM from cropped image region**
- Apple Vision Framework on macOS is nearly free and excellent — **always run it**

## Feature Extraction (CLIP, Florence-2)

**Skip in the hot path.**

- CLIP vectors can't be fed to a text LLM (incompatible representation)
- Florence-2 scene descriptions are redundant with what the LLM extracts from images
- Adding a preprocessing step adds latency without adding information

**Use only for auxiliary systems**: memory/retrieval, frame indexing, similarity search.

## Recommended Architecture

```
┌─────────────────────────────────────────────────────┐
│  LOCAL PREPROCESSING (every frame, 30-60 FPS)       │
│                                                     │
│  Screen ─→ YOLO (objects, boxes, tracking IDs)      │
│         ─→ OCR  (HUD text from known regions)       │
│                                                     │
│  Audio  ─→ VAD  (voice activity detection)          │
│         ─→ Whisper (fallback transcript)            │
│                                                     │
│  Output: structured JSON, ~100-400 tokens/update    │
└──────────────────┬──────────────────────────────────┘
                   │ every 200ms-1s
┌──────────────────▼──────────────────────────────────┐
│  LLM API (1-5 Hz decision cycle)                    │
│                                                     │
│  Input:                                             │
│   - YOLO detections + OCR text (every cycle)        │
│   - Raw screenshot (every 5-10s, or on change)      │
│   - Native audio stream (if Gemini Live)            │
│   - Human input events (CLX format)                 │
│                                                     │
│  Output: CLX agent commands (streaming)             │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│  EXECUTOR (120-240 Hz, RT thread)                   │
│  Parse + inject keyboard/mouse/gamepad/MIDI         │
└─────────────────────────────────────────────────────┘
```

### Cost savings with preprocessing
- Without preprocessing: ~1500 tokens/frame × 2 fps = **3000 tok/s**
- With YOLO+OCR: ~200 tokens/update × 2 fps + 1500 tokens every 10s = **550 tok/s average**
- **~5x cost reduction**, faster API responses (smaller payloads), better spatial precision

### When to send raw screenshots anyway
1. Initial scene (first frame of new context)
2. Significant scene change detected (YOLO object count changes dramatically)
3. Every 5-10 seconds for grounding (prevent drift from structured-only input)
4. When the LLM asks for visual context (query command `?`)
5. Stylized text that local OCR fails on
