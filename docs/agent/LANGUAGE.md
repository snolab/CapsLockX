# CLX Agent Language Specification

## Design Goals

1. **Minimal tokens** — LLMs pay per token; every character counts
2. **Streamable** — commands execute on newline/semicolon, no lookahead
3. **Human-readable** — terse but not cryptic
4. **Composable** — concurrent actions via channels, sequences via lines

## Grammar (EBNF-like)

```
program     = { statement NL }
statement   = command | comment | block
comment     = "#" { any }
block       = "{" NL { statement NL } "}"
command     = channel? action args? duration?
channel     = "@" IDENT ":"          // named device/channel
action      = IDENT                   // verb: k, m, p, M, w, r, etc.
args        = { arg }
arg         = NUMBER | STRING | IDENT | "+" NUMBER | "-" NUMBER | "~" NUMBER
duration    = NUMBER ("ms" | "s" | "f")   // milliseconds, seconds, frames
NL          = ";" | "\n"
```

## Core Commands

### Keyboard — `k` (key)

```
k a             # tap 'a' (down+up, ~50ms default)
k A             # tap Shift+A
k ret           # tap Return/Enter
k d:a           # key down 'a' (hold)
k u:a           # key up 'a' (release)
k c-c           # Ctrl+C  (modifier-key combo)
k a-tab         # Alt+Tab
k s-a           # Shift+A (same as k A)
k c-s-z         # Ctrl+Shift+Z
k w-space       # Win/Cmd+Space
k "hello"       # type string (auto key sequence)
```

**Modifier prefixes**: `c-` Ctrl, `s-` Shift, `a-` Alt/Option, `w-` Win/Cmd

**Special keys**: `ret` `esc` `tab` `space` `bksp` `del` `up` `down`
`left` `right` `home` `end` `pgup` `pgdn` `f1`-`f24` `caps` `ins`
`prtsc` `pause` `menu`

### Mouse — `m` (mouse)

```
m 500 300           # move to absolute (500, 300)
m +10 -5            # move relative (+10, -5)
m ~800 400 200ms    # lerp to (800, 400) over 200ms
m c                 # left click (down+up)
m c:r               # right click
m c:m               # middle click
m d                 # left button down (hold)
m u                 # left button up (release)
m d:r               # right button down
m s +3              # scroll up 3 notches
m s -2              # scroll down 2 notches
m 500 300 c         # move to (500,300) and click
```

### Gamepad / Stick — `p` (pad)

```
p ls 0.5 -0.3       # left stick: x=0.5 y=-0.3 (range -1.0 to 1.0)
p rs 0.0 1.0        # right stick: x=0 y=1 (full forward)
p lt 0.8            # left trigger: 0.8 (range 0.0 to 1.0)
p rt 1.0            # right trigger: full
p a                  # tap A button
p d:x                # hold X button
p u:x                # release X button
p dp up              # d-pad up
p dp dr              # d-pad down-right (diagonal)
p ls ~0.0 0.0 300ms  # lerp stick back to center over 300ms
```

**Buttons**: `a` `b` `x` `y` `lb` `rb` `lt` `rt` `ls` `rs`
`start` `back` `guide`
**D-pad**: `dp` + `up` `down` `left` `right` `ul` `ur` `dl` `dr`

### MIDI — `M` (MIDI)

```
M n 60 100           # note on: middle C, velocity 100
M o 60               # note off: middle C
M cc 1 64            # control change: modwheel at 64
M pb 8192            # pitch bend: center (0-16383)
M pc 5               # program change: patch 5
@midi2: M n 48 80    # send to named MIDI channel
```

### Wait / Timing — `w` (wait)

```
w 100ms              # wait 100 milliseconds
w 16f                # wait 16 frames (at current fps)
w 1s                 # wait 1 second
```

### Wait For — `wf` (wait for condition, like Playwright)

Blocks until a condition is met on the AX tree, with timeout.
Returns the matched element info. Essential for reliable automation —
never guess timing, always wait for the actual UI state.

```
wf "Quick Open" 3s           # wait until "Quick Open" appears in AX tree, 3s timeout
wf btn "Save" 5s             # wait for a button labeled "Save"
wf field "Search" 2s         # wait for a text field labeled "Search"
wf window "Untitled" 5s      # wait for a window with title containing "Untitled"
wf !loading 10s              # wait until "loading" disappears (! = not present)
wf txt "Success" 5s          # wait for static text containing "Success"
```

**Behavior:**
- Polls AX tree every 200ms until condition matches or timeout
- On match: returns `[OK wf] matched: btn "Save" @400,300` (with position!)
- On timeout: returns `[TIMEOUT wf] "Save" not found after 5s`
- The matched position can be used in the next command

**Examples in context:**
```
# Reliable file open (instead of guessing wait times)
k w-p
wf field "Search" 2s         # wait for Quick Open to actually appear
k "test-cycle.rs"
wf txt "test-cycle" 2s       # wait for search results
k ret
wf window "test-cycle" 3s    # wait for file to actually open

# Click a button that might take time to appear
m 400 300 c
wf btn "Confirm" 5s          # wait for confirmation dialog
m @last c                    # click the matched element (@last = last wf result)

# Wait for page load
k ret
wf !txt "Loading" 10s        # wait until "Loading" text disappears
wf txt "Dashboard" 5s        # then wait for Dashboard to appear
```

**Why this is better than `w 2000ms`:**
- Precise: acts immediately when ready (not after fixed delay)
- Reliable: doesn't fail if UI is slow (waits up to timeout)
- Informative: tells the LLM exactly what appeared and where
- Token-efficient: replaces multi-turn retry loops with one command

### Repeat — `r` (repeat)

```
r 5 { k a; w 50ms }  # tap 'a' 5 times, 50ms apart
r 0 { ... }           # repeat forever (until cancelled)
```

### Parallel / Concurrent — `|` (pipe)

```
m ~500 300 1s | p ls 0.5 0.0   # move mouse AND set stick simultaneously
```

### Variables / Labels (optional, for complex sequences)

```
$x = 500              # variable assignment
m $x 300              # use variable
:loop                  # label
k a; w 50ms; j loop    # jump to label
```

## Token Efficiency Examples

### Comparison: JSON vs CLX Agent Language

**Move mouse to (500, 300), click, type "hello", press Enter:**

JSON (typical LLM tool-call format) — **~45 tokens**:
```json
{"actions":[{"type":"mouse_move","x":500,"y":300},{"type":"mouse_click"},{"type":"type","text":"hello"},{"type":"key","key":"enter"}]}
```

CLX Agent Language — **~10 tokens**:
```
m 500 300 c
k "hello"
k ret
```

**Game input: move stick, press button, wait, release:**

JSON — **~50 tokens**:
```json
{"actions":[{"type":"gamepad_stick","stick":"left","x":1.0,"y":0.0},{"type":"gamepad_button","button":"a","state":"press"},{"type":"wait","ms":500},{"type":"gamepad_button","button":"a","state":"release"}]}
```

CLX Agent Language — **~8 tokens**:
```
p ls 1.0 0.0
p d:a
w 500ms
p u:a
```

**~5x token savings** on average.

## Streaming Behavior

Each line is an independently executable command. The parser:

1. Accumulates characters from the LLM stream
2. On newline or semicolon, parses the complete statement
3. Immediately dispatches to the command scheduler
4. Continues accumulating the next statement

This means the first command executes as soon as the first line is
complete in the stream — typically within 2-5 tokens of the LLM starting
its response.

## Error Recovery

If a line fails to parse:
- Log a warning with the malformed line
- Skip to next newline/semicolon
- Continue parsing

LLMs occasionally produce malformed output. The system must be resilient
and never crash or hang on bad input.

### Sense Control — `S` (sense)

The LLM controls its own perception. Every sensor starts at the lowest
detail level by default. The LLM actively raises frequency/detail when
needed and lowers it when done.

```
S screen off               # stop sending screenshots
S screen on                # resume at current settings
S screen fps 2             # set screenshot rate to 2 fps
S screen fps 0.2           # once every 5 seconds
S screen res 256            # set capture resolution (256x256)
S screen res 768            # higher detail
S screen mode ocr           # text-only mode (cheapest)
S screen mode image         # raw screenshots
S screen mode yolo          # YOLO detections only
S screen mode full          # YOLO + OCR + raw image
S screen region 0 0 500 300 # only capture this region

S key off                  # stop reporting keyboard events
S key on                   # resume
S key mode raw             # every key down/up
S key mode cooked          # combine into typed strings
S key mode event           # only modifiers and special keys

S mouse off                # stop reporting mouse
S mouse on
S mouse mode event         # only clicks/scroll (default)
S mouse mode pos 10        # position updates at 10 Hz
S mouse mode delta          # relative deltas only
S mouse mode moved          # just report "moved" or "idle" (cheapest)

S pad off                  # stop reporting gamepad
S pad on
S pad mode event           # only button presses (default)
S pad mode full 30         # all axes + buttons at 30 Hz
S pad mode stick 10        # stick positions at 10 Hz

S midi off
S midi on

S voice off                # stop reporting voice
S voice on
S voice mode stt           # text transcription (default)
S voice mode audio         # raw audio stream to LLM (Gemini Live)
S voice mode level         # just audio level (cheapest)

S audio off                # game/system audio
S audio on
S audio mode level 10      # audio level at 10 Hz
S audio mode fft           # frequency features

S app off                  # stop reporting active app changes
S app on

S clip off                 # stop monitoring clipboard
S clip on

S all off                  # mute all sensors
S all on                   # restore all to defaults
S all low                  # all sensors to lowest detail (default startup)
S all high                 # all sensors to highest detail
```

**Default startup state** — everything on, minimum detail:
```
S screen mode ocr fps 0.5 res 256    # OCR at 0.5fps, tiny resolution
S key mode event                      # only special keys
S mouse mode moved                    # just moved/idle flag
S pad off                             # off until needed
S midi off                            # off until needed
S voice mode stt                      # STT transcription
S audio off                           # off until needed
S app on                              # always report app changes
S clip off                            # off until needed
```

**Example: LLM escalates perception for a task**
```
# LLM starts a game — ramp up perception
S screen mode yolo fps 5 res 512
S pad mode full 30
S key off
# ... play the game ...
# Game paused — save tokens
S screen fps 0.5
S pad mode event
# Back to menu — switch to desktop mode
S all low
S screen mode ocr fps 1
S mouse mode event
```

### Query — `?` (ask for sensor data on demand)

Instead of continuous streaming, the LLM can request a single reading:

```
? screen                    # take one screenshot now
? screen ocr               # run OCR and return text
? screen yolo              # run YOLO and return detections
? mouse                    # current mouse position
? key                      # currently held keys
? pad                      # current gamepad state (all axes + buttons)
? app                      # active application info
? clip                     # clipboard contents
? time                     # current time
? sys                      # system state (cpu, mem, battery)
```

This is the most token-efficient way to get sensor data — the LLM only
pays tokens for data it actually needs, exactly when it needs it.

## Reserved for Future

- `v` — voice/audio output (TTS)
- `!` — system command (run shell)
- `@` — channel routing prefix (already used)
