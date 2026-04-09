# Demo Targets — Real-Time Games for Agent Showcase

Games selected for: complex real-time strategy, multiple simultaneous
channels, free/easy setup on macOS, and "wow factor" when an LLM plays them.

## Priority Targets

### 1. Surviv.io — 2D Battle Royale (Top Pick)

**URL:** https://surviv.io
**Platform:** Web browser (Chrome)
**Why:** Shows the FULL agent stack — movement + aiming + shooting + looting
+ healing + strategic positioning. Multiplayer against real humans.
"AI wins a battle royale" is a headline.

**Input channels (5-6 simultaneous):**
```
Channel 1: WASD movement        (digital, hold)
Channel 2: Mouse aim             (analog 2D, continuous)
Channel 3: Left click            (digital, shoot)
Channel 4: Right click           (digital, interact/pickup)
Channel 5: Number keys           (digital, weapon switch)
Channel 6: F/R/E                 (digital, interact/reload/use)
```

**Timing:** ~200-400ms reactions sufficient (2D overhead view, predictable)

**Agent configuration:**
```
# Slow thinker: strategic decisions (where to go, when to fight)
# Fast agents: motor control

fork aim "track nearest enemy, lead shots for moving targets. output m commands." --model groq-8b --loop 100ms
fork move "navigate toward objective, stay in safe zone, use cover. output k wasd." --model groq-8b --loop 150ms

# Reflexes
on "hp" < 30 { k 4 }              # use bandage
on "ammo" == 0 { k r }            # reload
on "enemy" dist < 50 { k space }  # dodge roll
on "loot" nearby { k f }          # auto-pickup

# Main agent: strategy every 2-3s
S screen mode yolo fps 5 res 512
tell move "safe zone is shrinking northeast, rotate now"
tell aim "ignore far targets, focus the one behind the tree"
```

**Screen parsing:** Top-down 2D, simple sprites, health/ammo HUD in corners.
YOLO can detect players, items, buildings. OCR reads health/ammo numbers.

---

### 2. Agar.io — Multiplayer Cell Eating

**URL:** https://agar.io
**Platform:** Web browser
**Why:** Continuous mouse control + split-second strategic decisions.
Beat humans at their own game. Simple visuals (colored circles).

**Input channels (2-3):**
```
Channel 1: Mouse position        (analog 2D, continuous — cell follows cursor)
Channel 2: Space                 (digital, split)
Channel 3: W                     (digital, eject mass)
```

**Timing:** ~200-500ms, continuous movement decisions

**Agent configuration:**
```
fork steer "navigate toward food, flee from larger cells, chase smaller ones. output m commands." --model groq-8b --loop 100ms

# Reflexes
on "bigger_enemy" dist < 200 { tell steer "FLEE from @enemy_pos" }
on "smaller_enemy" dist < 100 { tell steer "CHASE @enemy_pos" }

# Strategy (when to split, when to eject)
# Main agent decides split timing:
# "Enemy at 400,300 is half my size and close — split to eat them"
k space  # split!
```

**Screen parsing:** Circles on solid background. Color + size = threat level.
YOLO or simple color blob detection works perfectly.

---

### 3. Slither.io — Multiplayer Snake

**URL:** https://slither.io
**Platform:** Web browser
**Why:** Continuous mouse steering + boost timing + trap-laying strategy.
Length = score = visually obvious success. Beat humans.

**Input channels (2):**
```
Channel 1: Mouse position        (analog 2D, continuous — snake follows cursor)
Channel 2: Left click / Space    (digital, boost)
```

**Timing:** ~200-400ms for direction changes

**Agent configuration:**
```
fork steer "grow by eating dots, coil around smaller snakes, avoid hitting others. output m commands." --model groq-8b --loop 100ms

# Reflexes
on "collision_imminent" { tell steer "TURN AWAY from @obstacle_pos" }

# Strategy: trap opponents by circling them
# Main agent: "enemy at 500,300 is small, circle around them"
tell steer "spiral around @500,300 radius 100, tightening"
```

---

### 4. Krunker.io — Browser FPS

**URL:** https://krunker.io
**Platform:** Web browser
**Why:** FPS in browser, multiplayer. Even getting 1-2 kills against
humans is extremely impressive. "LLM plays FPS" is viral-worthy.
Blocky art style is easier for vision.

**Input channels (5-6):**
```
Channel 1: WASD movement        (digital, hold)
Channel 2: Mouse aim             (analog 2D, continuous)
Channel 3: Left click            (digital, shoot)
Channel 4: Right click           (digital, ADS)
Channel 5: Space                 (digital, jump)
Channel 6: R/number keys         (digital, reload/switch)
```

**Timing:** ~100-200ms (hardest on the list)

**Agent configuration:**
```
fork aim "track enemies, snap to heads. output m commands." --model groq-8b --loop 50ms
fork move "strafe unpredictably, move toward objectives. output k wasd." --model groq-8b --loop 100ms

on "enemy_visible" { m @enemy_pos c }   # snap aim + shoot
on "hp" < 30 { tell move "retreat behind cover" }
on "ammo" == 0 { k r }
```

**Difficulty:** Hardest target. ~100ms reactions needed. Demo may show
partial success (some kills, deaths too) which is still impressive.

---

### 5. Diep.io — Tank Shooter MMO

**URL:** https://diep.io
**Platform:** Web browser
**Why:** Movement + aiming + shooting + upgrade decisions. Shows
strategic resource allocation (upgrade tree). Continuous action.

**Input channels (4):**
```
Channel 1: WASD movement        (digital, hold)
Channel 2: Mouse aim             (analog 2D, continuous)
Channel 3: Left click            (digital, auto-shoot hold)
Channel 4: Number keys 1-8       (digital, upgrade stats)
```

**Timing:** ~200-400ms

**Agent configuration:**
```
fork combat "aim at nearest enemy/polygon, keep shooting. output m + click." --model groq-8b --loop 100ms
fork move "farm polygons when safe, flee when damaged. output k wasd." --model groq-8b --loop 200ms

# Strategy: upgrade decisions
on "level_up" { k 3; k 3; k 3 }  # max bullet damage first
tell move "farm the pentagon nest in center"
tell combat "focus the red triangle player"
```

---

### 6. Super Mario Bros — NES via RetroArch

**Install:** `brew install --cask retroarch` + FCEUmm core + ROM
**Platform:** macOS native (RetroArch)
**Why:** Most iconic game ever. "AI plays Mario" is instantly understood.
Clearing World 1-1 to 1-4 would be stunning.

**Input channels (3-4):**
```
Channel 1: Right arrow           (digital, hold — run forward)
Channel 2: B button (Z key)      (digital, hold — sprint)
Channel 3: A button (X key)      (digital, tap — jump)
Channel 4: Left/Down             (digital, occasional — backtrack/duck)
```

**Timing:** ~200-400ms for most jumps, ~100ms for tight sections

**Agent configuration:**
```
fork run "run right, jump over gaps and enemies. output k right/x/z." --model groq-8b --loop 100ms

# Reflexes (these handle the critical timing)
on "gap" dist < 40 { k x }           # jump over gap
on "goomba" dist < 60 { k x }        # jump on enemy
on "pipe" dist < 30 { k x }          # jump over pipe
on "pit" dist < 50 { k x; w 50ms }   # long jump over pit

# Strategy: main agent monitors progress
S screen mode yolo fps 10 res 256
# "stuck at pipe — need to jump earlier"
tell run "jump 10px earlier for pipes"
```

**Screen parsing:** NES pixel art is very consistent. YOLO can be trained
on a few hundred frames. Alternatively, RAM state (via RetroArch API)
gives perfect game state — position, enemies, gaps.

---

### 7. StarCraft: Brood War — Free RTS

**Install:** Free from Blizzard (StarCraft Anthology, free since 2017)
**Platform:** macOS (via Wine/CrossOver) or native old build
**Why:** RTS with micro + macro. Even basic play against AI is impressive.
Shows the full swarm: macro agent + micro agents per unit group.

**Input channels (3-5):**
```
Channel 1: Mouse position        (analog 2D, continuous)
Channel 2: Left click            (digital, select)
Channel 3: Right click           (digital, command)
Channel 4: Hotkeys               (digital, 1-0 for groups, A-attack, etc.)
Channel 5: Shift                 (digital, queue commands)
```

**Timing:** ~100-500ms for individual actions, sustained APM matters

**Agent configuration:**
```
fork macro "build workers, expand bases, research upgrades. output m clicks + k hotkeys." --model gemini-flash --loop 2s
fork micro "control army in combat, focus fire, retreat wounded units." --model groq-8b --loop 200ms
fork scout "send a unit to scout enemy base, report findings." --model groq-8b --loop 1s

# Reflexes
on "enemy_attack" at "base" { tell micro "defend base NOW" }
on "minerals" > 400 { tell macro "spend money, build more units" }

# Strategy (slow thinker)
# "Enemy is going mass air. Switch to anti-air units."
tell macro "build 5 goliaths, research range upgrade"
tell micro "focus the mutalisks first"
```

**Difficulty:** Complex, but playing against Easy AI is feasible. Winning
a game against AI on easy difficulty would be extremely impressive.

---

### 8. Superhot (Web Demo) — Time-Moves-When-You-Move FPS

**URL:** Search "superhot prototype" or https://superhotgame.com/play-prototype/
**Platform:** Web browser
**Why:** Time only moves when you move — **perfect for LLM agents**.
The agent can "think" for as long as it wants when stationary.
Effectively turns a fast FPS into a strategy game.

**Input channels (3-4):**
```
Channel 1: WASD movement        (digital — time advances with movement)
Channel 2: Mouse aim             (analog 2D — time advances with aim)
Channel 3: Left click            (digital, shoot/punch)
Channel 4: E                     (digital, pickup weapon)
```

**Timing:** Effectively unlimited — agent controls the pace.

**Agent configuration:**
```
# Single agent is sufficient — no need for fast sub-agents
# because time freezes when the agent stops outputting commands

S screen mode yolo fps 2 res 512

# Plan: observe frozen scene, identify threats, plan sequence
# Execute: move + aim + shoot in rapid burst
# Pause: stop outputting to freeze time and re-evaluate

m 400 300         # aim at enemy 1
m 400 300 c       # shoot
w 50ms            # let time advance slightly
# Stop outputting — time freezes — re-read screen
# Next turn: handle enemy 2
```

**Why this is the best "wow" demo:** It looks like the agent is playing
a fast FPS (bullets, explosions) but it actually has unlimited think time.
Spectators don't realize time is frozen — they just see the agent dodge
bullets and headshot enemies.

---

## Demo Difficulty Ranking

| Game | Difficulty for Agent | Wow Factor | Setup Effort |
|---|---|---|---|
| Superhot (web) | Easy (time freezes) | Very High | Open URL |
| Agar.io | Easy-Medium | High (beat humans) | Open URL |
| Slither.io | Easy-Medium | High (beat humans) | Open URL |
| Snake | Easy | Medium | Open URL |
| Diep.io | Medium | High | Open URL |
| Surviv.io | Medium-Hard | Very High | Open URL |
| Super Mario Bros | Medium | Very High | RetroArch setup |
| StarCraft:BW | Hard | Extremely High | Install game |
| Krunker.io | Very Hard | Extremely High | Open URL |

## Recommended Demo Order

1. **Superhot web demo** — "Look, the AI is playing an FPS!" (actually easy)
2. **Agar.io** — "It's beating real humans!" (medium, multiplayer)
3. **Surviv.io** — "Full battle royale with looting and combat!" (hard, impressive)
4. **Super Mario Bros** — "Iconic. Everyone gets it." (medium, nostalgic)

## See Also

- [GAMES.md](./GAMES.md) — Multi-channel agent swarm architecture
- [GAMES-INPUT-PATTERNS.md](./GAMES-INPUT-PATTERNS.md) — Input patterns per genre
- [MODELS.md](./MODELS.md) — Model selection for fast/slow agents
