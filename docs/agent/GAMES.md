# Games — Multi-Channel Agent Architecture

## The Core Problem

Games need **simultaneous continuous inputs** on multiple channels.
An LLM outputs text sequentially — one token at a time.
How do you get parallel analog control from a serial text stream?

## Solution: Hierarchical Agent Swarm

```
┌──────────────────────────────────────────────────────┐
│ SLOW THINKER — Strategy (1-5s cycle)                 │
│ Gemini 2.5 Pro / Claude Opus                         │
│ "Retreat to cover, heal, then flank from the right"  │
│                                                      │
│ fork/tell/kill sub-agents                            │
└───────┬──────────┬──────────┬────────────────────────┘
        │          │          │
   ┌────▼───┐ ┌────▼───┐ ┌───▼────┐ ┌──────────┐
   │ FAST   │ │ FAST   │ │ FAST   │ │ REFLEX   │
   │ aim    │ │ move   │ │ combat │ │ (local   │
   │ 50ms   │ │ 100ms  │ │ 100ms  │ │  rules)  │
   │ loop   │ │ loop   │ │ loop   │ │ 0ms      │
   │        │ │        │ │        │ │          │
   │ Groq   │ │ Groq   │ │ Groq   │ │ no LLM  │
   │ 8B     │ │ 8B     │ │ 8B     │ │          │
   │        │ │        │ │        │ │ on hp<30 │
   │m +2 -1 │ │p ls 1 0│ │p d:a   │ │ → heal  │
   │m +1 0  │ │p ls .5 0│ │w 100ms │ │on enemy │
   │m -1 +2 │ │p ls 0 0│ │p u:a   │ │ → dodge │
   └────────┘ └────────┘ └────────┘ └──────────┘
```

### Three thinking speeds

| Layer | Model | Cycle | Role | Cost/cycle |
|---|---|---|---|---|
| **Reflex** | None (local rules) | 0ms | Instant reactions: heal at low HP, dodge on alert | Free |
| **Fast** | Groq/Cerebras 8B | 50-200ms | Motor control: aim tracking, movement, combos | ~20 tokens |
| **Slow** | Gemini Pro / Claude | 1-5s | Strategy: plan routes, decide tactics, adapt | ~200 tokens |

### Fork semantics (cheap sub-agents)

```
# Fork inherits parent's full conversation history (KV cache shared)
# Only the role instruction is new — minimal extra tokens
fork aim "track nearest enemy with mouse. output m +dx +dy at 50ms intervals."
fork move "navigate to waypoint. output p ls x y at 100ms intervals."

# Main agent continues as strategist
tell aim "switch target to the sniper on the roof"
tell move "retreat behind the wall"
kill move
fork move "circle right to flank" --loop 100ms

# React rules (no LLM, instant)
on "hp" < 30 { k 5 }              # use health potion
on "ammo" == 0 { k r }            # auto-reload
on "enemy" dist < 50 { k s-space } # dodge roll
```

## Game Feasibility by Category

### Tier 1: Excellent (LLM is fast enough)

| Genre | Channels | Timing | Agent Config |
|---|---|---|---|
| Turn-Based RPG | 1 | Seconds+ | Single slow thinker |
| Turn-Based Strategy | 1 | None | Single slow thinker |
| Card Games | 1 | None | Single slow thinker |
| Visual Novel | 1 | None | Single slow thinker |
| Board Games | 1 | None | Single slow thinker |
| Idle/Clicker | 0-1 | None | Reflex rules only |
| Puzzle (untimed) | 1 | None | Single slow thinker |
| Tower Defense | 1-2 | ~1s | Slow thinker + reflex |
| City Builder | 1-2 | None | Single slow thinker |
| Life Sim | 1-2 | None | Single slow thinker |
| 4X Strategy | 1 | None | Single slow thinker |

### Tier 2: Good (needs fast agents)

| Genre | Channels | Timing | Agent Config |
|---|---|---|---|
| MOBA | 3-5 | ~100ms | Slow (strategy) + 2 fast (move, ability) |
| RTS | 3-5 | ~100ms | Slow (macro) + 2 fast (micro, build) |
| Action RPG | 4-6 | ~200ms | Slow (strategy) + 2 fast (combat, dodge) |
| Roguelite | 4-6 | ~200ms | Slow (route) + 2 fast (combat, move) |
| Survival | 5-7 | ~200ms | Slow (plan) + 2 fast (combat, gather) |
| MMO | 3-5 | ~200ms | Slow (strategy) + fast (rotation) |
| 3D Platformer | 3-4 | ~200ms | Slow (route) + fast (jump timing) |
| Horror | 3-7 | ~300ms | Slow (explore) + fast (combat/flee) |

### Tier 3: Marginal (needs swarm + reflexes)

| Genre | Channels | Timing | Agent Config |
|---|---|---|---|
| 2D Platformer | 3-4 | ~50ms | Fast (movement) + reflex (jump timing) |
| TPS | 5-7 | ~100ms | Slow + 3 fast (aim, move, abilities) |
| Battle Royale | 5-9 | ~50ms | Slow + 3 fast + reflexes |
| Sandbox | 4-6 | Varies | Slow (plan) + fast (build/combat) |
| Mobile Action | 2-6 | ~200ms | 2 fast (stick, buttons) |

### Tier 4: Not feasible with current LLMs

| Genre | Channels | Timing | Why |
|---|---|---|---|
| FPS (competitive) | 6-8 | ~16ms | Aim tracking needs <16ms, LLM min ~50ms |
| Fighting Games | 2-4 | ~16ms | Frame-perfect combos, reaction-based |
| Racing (sim) | 3-5 | Continuous | Analog steering needs 30Hz+ continuous |
| Rhythm Games | 2-10 | ~16-50ms | Frame-perfect timing is the entire game |
| Sports (sim) | 4-5 | ~50ms | Analog control + fast reactions |
| VR | 12-20+ | ~50ms | 6DOF tracking impossible via text |
| Flight Sim | 6-10+ | Continuous | Multi-axis analog + procedures |

## CLX Language Extensions for Games

### Fork / Tell / Kill

```
fork NAME "instruction" [--model MODEL] [--loop INTERVAL]
tell NAME "new instruction"
kill NAME
pause NAME
resume NAME
```

### React Rules (no LLM, local condition → action)

```
on CONDITION { COMMANDS }

# Examples:
on "hp" < 30 { k 5 }                    # use potion when low HP
on "ammo" == 0 { k r; w 500ms }         # reload when empty
on "enemy" dist < 100 { p d:b; w 200ms; p u:b }  # dodge when close
on "cooldown_q" ready { k q }           # use ability when available
```

Conditions are evaluated against the SENSE stream (AX tree, YOLO detections,
OCR text). The runtime checks conditions at the sensor polling rate.

### Continuous Output Mode

For fast agents controlling analog axes, output a stream of values:

```
# Instead of discrete commands:
p ls 0.5 0.0
w 50ms
p ls 0.6 0.1
w 50ms
p ls 0.7 0.2

# Continuous interpolation (runtime handles timing):
p ls ~0.5,0.0 ~0.7,0.2 500ms    # lerp stick from (0.5,0) to (0.7,0.2) over 500ms
m ~400,300 ~600,200 200ms        # lerp mouse smoothly
```

The runtime interpolates between waypoints at hardware rate (120-240Hz),
so the LLM only needs to output target positions, not individual steps.

## Concurrent Channel Architecture

```
┌────────────────────────────────────────┐
│ Channel Manager                        │
│                                        │
│ ch:aim   ← fast agent (m commands)     │
│ ch:move  ← fast agent (p ls commands)  │
│ ch:btn   ← fast agent (p d/u commands) │
│ ch:main  ← slow agent (fork/tell/kill) │
│ ch:reflex ← local rules (on conditions)│
│                                        │
│ Each channel has its own command queue  │
│ All channels execute in parallel       │
│ Channels can be paused/killed          │
└───────────┬────────────────────────────┘
            │
   ┌────────┼────────┐
   ▼        ▼        ▼
Keyboard  Mouse   Gamepad   MIDI
injector  injector emulator output
```

Each channel owns a device axis:
- `aim` → mouse dx/dy
- `move` → left stick x/y
- `camera` → right stick x/y
- `triggers` → LT/RT analog
- `buttons` → discrete press/release

Channels don't conflict because they control different physical axes.

## Cost Estimation

### Example: Playing an action RPG for 1 hour

| Agent | Model | Cycle | Tokens/cycle | Cycles/hour | Total tokens |
|---|---|---|---|---|---|
| Strategy | Gemini Flash | 5s | 200 | 720 | 144K |
| Combat | Groq 8B | 200ms | 20 | 18,000 | 360K |
| Movement | Groq 8B | 200ms | 15 | 18,000 | 270K |
| Reflexes | Local rules | — | 0 | — | 0 |
| SENSE input | — | — | — | — | ~500K |
| **Total** | | | | | **~1.3M tokens** |

Cost:
- Groq 8B: ~$0.05/M input + $0.08/M output → ~$0.08
- Gemini Flash: ~$0.10/M input + $0.40/M output → ~$0.10
- **Total: ~$0.20/hour** for an action RPG

### Example: Turn-based strategy (single agent)

| Agent | Model | Cycle | Tokens/cycle | Cycles/hour | Total tokens |
|---|---|---|---|---|---|
| Strategy | Gemini Flash | 5s | 300 | 720 | 216K |
| **Total** | | | | | **~216K tokens** |

Cost: **~$0.03/hour**

## See Also

- [STREAMING-OUTPUT.md](./STREAMING-OUTPUT.md) — async command queue design
- [MODELS.md](./MODELS.md) — model selection per speed tier
- [MODEL-FAST.md](./MODEL-FAST.md) — fast models for sub-agents
- [SENSE.md](./SENSE.md) — sensor input to agents
