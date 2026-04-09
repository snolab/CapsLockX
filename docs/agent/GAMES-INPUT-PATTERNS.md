# Game Input Patterns — Complete Reference

## Summary Table

| Genre | Channels | Inputs/sec | Timing | Analog | LLM Tier |
|---|---|---|---|---|---|
| FPS | 6-8 | 8-20 | ~16ms | High | 4 (not feasible) |
| TPS | 5-7 | 6-15 | ~100ms | High | 3 (marginal) |
| MOBA | 3-5 | 3-12 | ~100ms | Medium | 2 (good) |
| Battle Royale | 5-9 | 2-20 | ~50ms | High | 3 (marginal) |
| Fighting | 2-4 | 5-20 | ~16ms | Low | 4 (not feasible) |
| Racing | 3-5 | 10-30 | ~50ms | **Critical** | 4 (not feasible) |
| 2D Platformer | 3-4 | 5-20 | ~50ms | Low | 3 (marginal) |
| 3D Platformer | 3-4 | 4-10 | ~200ms | Medium | 2 (good) |
| Action RPG | 4-6 | 4-12 | ~200ms | Medium | 2 (good) |
| Turn-Based RPG | 1-2 | 0.5-3 | Seconds | None | 1 (excellent) |
| RTS | 3-5 | 5-20 | ~100ms | Low | 2 (good) |
| Turn-Based Strategy | 1 | 0.5-2 | None | None | 1 (excellent) |
| Tower Defense | 1-2 | 1-8 | ~1s | None | 1 (excellent) |
| Roguelite | 4-6 | 8-15 | ~200ms | Medium | 2 (good) |
| Survival | 5-7 | 2-10 | ~200ms | Medium | 2 (good) |
| Rhythm | 2-10 | 5-20+ | ~16ms | Low | 4 (not feasible) |
| Puzzle | 1-3 | 0.5-12 | None-100ms | None | 1 (excellent) |
| Card Game | 1 | 0.3-1 | None | None | 1 (excellent) |
| Sports | 4-5 | 5-12 | ~50ms | **Critical** | 4 (not feasible) |
| City Builder | 1-2 | 0.5-3 | None | None | 1 (excellent) |
| Life Sim | 1-2 | 1-3 | None | None | 1 (excellent) |
| Flight Sim | 6-10+ | Continuous | Procedural | **Critical** | 4 (not feasible) |
| MMO | 3-5 | 2-8 | ~200ms | Low | 2 (good) |
| Visual Novel | 1 | 0.3-1 | None | None | 1 (excellent) |
| Sandbox | 4-6 | 1-8 | Forgiving | Low | 2 (good) |
| Horror | 3-7 | 1-8 | ~300ms | Medium | 2 (good) |
| Idle/Clicker | 0-1 | 0-15 | None | None | 1 (excellent) |
| Board Game | 1 | 0.2-1 | None | None | 1 (excellent) |
| VR | 12-20+ | Continuous | ~50ms | **Critical** | 4 (not feasible) |
| Mobile | 1-6 | 1-15 | ~200ms | Medium | 2-3 |

## Detailed Input Patterns by Genre

### FPS — 6-8 simultaneous channels

```
Channel 1: WASD movement       (digital, hold)
Channel 2: Mouse aim           (analog 2D, continuous 120Hz+)
Channel 3: Left click          (digital, fire)
Channel 4: Right click         (digital, ADS/scope)
Channel 5: Sprint/Crouch       (digital, hold Shift/Ctrl)
Channel 6: Jump/Dodge          (digital, tap Space)
Channel 7: Ability keys        (digital, tap Q/E/F)
Channel 8: Weapon switch       (digital, scroll/number)

Concurrent example (during firefight):
  W held + A held + Shift held    (move forward-left sprinting)
  + Mouse continuous aim           (tracking enemy)
  + Left click hold                (full-auto fire)
  + Right click held               (ADS)
  = 6 simultaneous inputs
```
**Examples:** Counter-Strike 2, Valorant, Apex Legends, Overwatch 2, CoD

### Racing — 3-5 analog channels

```
Channel 1: Steering          (analog, continuous -1.0 to +1.0)
Channel 2: Throttle           (analog, continuous 0.0 to 1.0)
Channel 3: Brake              (analog, continuous 0.0 to 1.0)
Channel 4: Gear shift         (digital, up/down)
Channel 5: Handbrake/Nitro    (digital, tap)

Concurrent example (cornering):
  Steering at 0.6              (partial right turn)
  + Throttle at 0.3            (trail-braking exit)
  + Brake at 0.7               (trail-braking entry)
  = 3 simultaneous analog axes
```
**Examples:** Assetto Corsa, iRacing, Forza Motorsport, Gran Turismo 7

### Fighting Games — 2-4 channels, frame-perfect timing

```
Channel 1: Direction           (digital 8-way, hold/tap sequences)
Channel 2: Attack buttons      (digital, 1-3 simultaneous)

Combo input (must execute within ~300ms):
  ↓ ↘ → + Punch               (quarter-circle forward + punch = hadouken)
  Frame 1: ↓ held
  Frame 2-3: ↘ held
  Frame 4-5: → held
  Frame 6: → + LP pressed      (must be same frame or +1)

1-frame link (16.67ms window):
  MP on hit → wait exactly 1 frame → MP again
```
**Examples:** Street Fighter 6, Tekken 8, Guilty Gear Strive

### Rhythm Games — 2-10 channels, ultra-precise timing

```
Channel 1-4/7/10: Key lanes    (digital, tap at exact timing)

osu!mania 4-key:
  D F J K                      (4 lanes)
  Hit window: ±16ms = Marvelous, ±50ms = Perfect, ±100ms = Great

Beat Saber:
  Left hand: 3D position + rotation (6DOF continuous)
  Right hand: 3D position + rotation (6DOF continuous)
  Hit timing: ±50ms for full score
```
**Examples:** osu!, Beat Saber, DJMAX, Guitar Hero

### Action RPG — 4-6 channels

```
Channel 1: WASD movement       (digital, hold/tap for dodge)
Channel 2: Mouse aim/camera    (analog 2D)
Channel 3: Attack/Block        (digital, left/right click)
Channel 4: Dodge roll          (digital, tap Shift/Space)
Channel 5: Ability             (digital, tap Q/E/R)
Channel 6: Potion/Item         (digital, tap number)

Boss fight pattern:
  1. Dodge roll (Shift tap, ~200ms i-frame window)
  2. Run behind boss (W + mouse aim)
  3. Heavy attack combo (right click × 3, ~300ms between each)
  4. Dodge away before boss turns
  5. Repeat
```
**Examples:** Elden Ring, Diablo IV, Monster Hunter, Path of Exile

### Turn-Based Strategy — 1 channel, no timing

```
Channel 1: Mouse click         (select unit → click destination → confirm)

Typical turn:
  1. Click unit A (500ms thinking)
  2. Click move destination (1s thinking)
  3. Click "attack" ability (200ms)
  4. Click enemy target (500ms)
  5. Click "end turn" (200ms)

  Total: ~5 inputs over ~3-5 seconds. No simultaneous inputs ever.
```
**Examples:** XCOM 2, Civilization VI, Fire Emblem, Into the Breach

### VR — 12-20+ channels (most complex)

```
Left hand:  position XYZ (3 analog, continuous 90Hz)
            rotation RPY (3 analog, continuous 90Hz)
            trigger (1 analog)
            grip (1 analog/digital)
            thumbstick (2 analog)
            buttons A/B (2 digital)

Right hand: position XYZ (3 analog)
            rotation RPY (3 analog)
            trigger (1 analog)
            grip (1 analog/digital)
            thumbstick (2 analog)
            buttons A/B (2 digital)

Head:       position XYZ (3 analog, passive tracking)
            rotation RPY (3 analog, passive tracking)

Total: ~26 continuous channels
```
**Examples:** Half-Life: Alyx, Beat Saber, Pavlov VR

## Agent Configuration Templates

### Template: Turn-Based (single agent, no fork)
```
# Just one slow thinker
S screen mode ocr fps 0.5
# LLM analyzes board state, outputs clicks
m 400 300 c
w 500ms
m 200 400 c
```

### Template: Action RPG (slow + 2 fast)
```
fork combat "dodge when boss attacks, counter-attack after. output p/k commands." --model groq-8b --loop 200ms
fork move "navigate to objective, avoid hazards. output p ls commands." --model groq-8b --loop 200ms
on "hp" < 30 { k 5 }
# Main agent: strategy decisions every 3-5s
tell combat "boss is in phase 2, use ranged attacks"
tell move "stay at medium range, circle left"
```

### Template: MOBA (slow + fast + reflexes)
```
fork micro "last-hit minions, dodge skillshots. output m/k commands." --model groq-8b --loop 150ms
on "hp" < 25% { k d }          # flash away
on "ally_hp" < 20% { k r }     # heal ally (support)
# Main agent: macro strategy
tell micro "push lane, ward river at @800,400"
tell micro "group mid for teamfight"
```

### Template: Platformer (fast only)
```
fork run "navigate the level. hold right, jump gaps, avoid enemies." --model groq-8b --loop 100ms
on "spike" dist < 30 { k space }   # auto-jump spikes
on "enemy" dist < 50 { k space }   # auto-jump enemies
```
