# Streaming Output — Async Execution Design

## Problem

When the LLM streams output, the token parser currently blocks on
`w 200ms` and `wf "text" 3s`. This means:

- LLM output stalls for the duration of every wait
- A `wf` with 3s timeout blocks token parsing for 3 seconds
- Total latency = LLM generation time + sum of all wait times
- The LLM can't "think ahead" while waiting

```
Current (synchronous):

LLM tokens:  k w-p ··· w 200ms ········· k "README" ··· k ret
Parser:      ▓▓▓▓▓     ░░░░░░░░░░░░░░░░  ▓▓▓▓▓▓▓▓▓     ▓▓▓▓
Execution:   ▓         ░░░ blocked 200ms  ▓              ▓
                        ↑ parser AND executor both blocked
```

## Solution: Async Command Queue

Separate token parsing from command execution. Parser never blocks.
Commands go into a queue; executor runs them in order with timing.

```
Proposed (async queue):

LLM tokens:  k w-p · w 200ms · k "README" · k ret     (all parsed instantly)
Parser:      ▓▓▓▓▓   ▓▓▓▓▓▓▓   ▓▓▓▓▓▓▓▓▓   ▓▓▓▓      (never blocks)
                ↓        ↓          ↓          ↓
Queue:       [k w-p] [w 200ms] [k "README"] [k ret]
                ↓
Executor:    ▓────── ░░░200ms░░ ▓────────── ▓──         (runs in order)
```

## Design

### Token Parser Thread (never blocks)
```
LLM SSE stream → accumulate tokens → parse complete lines → push to queue
```

### Command Queue (crossbeam channel or VecDeque + Mutex)
```
[Cmd::KeyTap, Cmd::Wait(200ms), Cmd::TypeString, Cmd::KeyTap, ...]
```

### Executor Thread (RT priority, runs commands in order)
```
loop {
    cmd = queue.pop();
    match cmd {
        KeyTap/MouseMove/... → execute instantly (<1ms)
        Wait(d)              → sleep(d), then continue
        WaitFor(q, timeout)  → poll AX tree until match, then continue
    }
}
```

## What changes

| Command | Current | Proposed |
|---|---|---|
| `k a` | Execute + block parser ~1ms | Execute instantly, parser unblocked |
| `w 200ms` | Block parser 200ms | Queue delay, parser unblocked |
| `wf "text" 3s` | Block parser up to 3s | Queue polls async, parser unblocked |
| `m 400 300` | Execute + block parser ~1ms | Execute instantly, parser unblocked |

## Sequential vs Parallel

### Default: Sequential (queued in order)
```
k w-p
w 200ms
k "README.md"
k ret
```
Commands run in order: press Cmd+P → wait 200ms → type → enter.
This is the natural, safe behavior.

### Explicit parallel: `|` operator
```
m ~500 300 1s | p ls 0.5 0.0
```
Both commands start simultaneously. Useful for:
- Moving mouse while holding a gamepad stick
- Smooth mouse lerp while pressing keys

### Explicit sequential: `;` on same line
```
k w-p; w 200ms; k "README.md"; k ret
```
Same as separate lines — sequential. Just more compact.

## Latency Improvement

### Example: "open README in VSCode"

**Current (blocking):**
```
LLM TTFT:     200ms
k w-p:          1ms
w 200ms:      200ms  ← parser blocked!
k "README.md":  5ms
k ret:          1ms
wf change 3s: ???ms  ← parser blocked up to 3s!
─────────────────────
Total:        ~407ms + wf time (parser blocked entire time)
```

**Proposed (async queue):**
```
LLM TTFT:     200ms
All tokens:    50ms  ← parser reads all tokens in ~50ms
Queue runs:
  k w-p:        1ms
  w 200ms:    200ms  (executor sleeps, parser already done)
  k "README":   5ms
  k ret:        1ms
─────────────────────
Total:        ~250ms parse + 207ms execute = 457ms
But parser finished at 250ms — ready for next turn!
```

The key win: **parser finishes 200ms+ earlier**, so the next LLM turn
can start sooner. Over multiple turns, this compounds.

## Echo / Feedback Timing

With async execution, echo happens when commands actually execute,
not when parsed:

```
LLM output:   k w-p\n w 200ms\n k "README.md"\n k ret\n
Parser echo:  (nothing — just queues)
Executor:
  T+0ms:   execute k w-p      → echo "[    250ms] > k w-p"
  T+1ms:   start w 200ms
  T+201ms: execute k "README"  → echo "[    451ms] > k README.md"
  T+206ms: execute k ret       → echo "[    456ms] > k ret"
```

Timestamps in the echo show actual execution time, which is what
matters for debugging timing issues.

## AX Tree Diff Timing

Currently the AX tree is re-read after all commands in a turn.
With async queue, we can be smarter:

**Option A**: Read AX tree after queue drains (same as now, but
the tree read doesn't block the parser — it happens on executor thread).

**Option B**: Auto-read AX tree after any `wf` completes, since
that's when the UI has settled.

**Option C**: Background AX tree poller (every 500ms) that detects
changes independently of command execution. Changes are sent to the
LLM as `[AX CHANGED]` events in the signal stream.

Option A is simplest for MVP. Option C is the full SENSE.md vision.

## Implementation Plan

1. Add `crossbeam-channel` dependency (already in Cargo.toml plan)
2. Split `run_agent_loop` into:
   - Parser closure (in `stream_chat` callback) → sends to channel
   - Executor thread → receives from channel, executes, echoes
3. Executor thread runs at elevated priority (`SCHED_FIFO` or similar)
4. Echo goes back to stderr (for overlay) from executor thread
5. AX tree re-read happens on executor thread after queue drains

## Future: Bidirectional Streaming

The async queue naturally supports bidirectional streaming with
realtime models (Gemini Live API):

```
┌─────────────┐     ┌──────────┐     ┌──────────┐
│ LLM stream  │────→│ Parser   │────→│ Command  │
│ (tokens in) │     │ (never   │     │ Queue    │
│             │     │  blocks) │     │          │
│ ← feedback  │←────│          │←────│ Executor │
│   (AX diff, │     │          │     │ (echoes  │
│    echoes)  │     │          │     │  results)│
└─────────────┘     └──────────┘     └──────────┘
```

The parser and executor run independently. Feedback from the executor
(echoes, AX diffs) feeds back into the LLM's input stream for the
next turn — or into a live WebSocket session for realtime models.
