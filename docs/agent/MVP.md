# MVP — Minimum Viable Agent Loop

## Goal

A working voice+screen→LLM→action loop:
1. **Voice** (STT) + **Accessibility tree** (screen text+positions) → context
2. **Fast LLM** → streams CLX commands
3. **Parser** → parses streaming tokens into commands
4. **Actor** → executes commands (keyboard/mouse)
5. **Echo** → command results stream back to LLM as feedback

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                        SENSE (input to LLM)                    │
│                                                                │
│  ┌─────────────┐    ┌──────────────────────────┐               │
│  │ Voice (STT) │    │ Accessibility Tree       │               │
│  │ Whisper/    │    │ AX API → text elements   │               │
│  │ SenseVoice  │    │ with center positions    │               │
│  └──────┬──────┘    │ as indented tree text    │               │
│         │           └────────────┬─────────────┘               │
│         │                        │                             │
│         └────────┬───────────────┘                             │
│                  │                                             │
│         ┌────────▼────────┐                                    │
│         │ Context Builder │                                    │
│         │ system prompt + │                                    │
│         │ AX tree + voice │                                    │
│         │ + echo feedback │◄──────────────── echo ◄────────┐   │
│         └────────┬────────┘                                │   │
└──────────────────┼─────────────────────────────────────────┼───┘
                   │                                         │
            ┌──────▼──────┐                                  │
            │  LLM API    │                                  │
            │  (Gemini    │                                  │
            │   Flash /   │                                  │
            │   Groq)     │                                  │
            │  streaming  │                                  │
            └──────┬──────┘                                  │
                   │ SSE tokens                              │
            ┌──────▼──────┐                                  │
            │  Parser     │                                  │
            │  (winnow    │                                  │
            │   Partial)  │                                  │
            └──────┬──────┘                                  │
                   │ Command                                 │
            ┌──────▼──────┐                                  │
            │  Actor      │──── echo result ─────────────────┘
            │  (execute   │
            │   key/mouse)│
            └─────────────┘
```

## MVP Scope (what to build)

### In scope
- [x] Accessibility tree reader (macOS AX API → text tree)
- [ ] winnow streaming parser for `k`, `m`, `w` commands
- [ ] LLM API streaming client (Gemini Flash, OpenAI-compat)
- [ ] Actor: execute parsed commands via existing CapsLockX platform
- [ ] Echo: stream command execution results back to LLM context
- [ ] CLX+A hotkey to activate agent with voice prompt
- [ ] System prompt with CLX language reference + AX tree context

### Out of scope (Phase 2+)
- Gamepad emulation (`p` commands)
- MIDI output (`M` commands)
- Screen capture / vision
- YOLO preprocessing
- Sense control (`S` commands)
- Variables, loops, labels

## Implementation Plan

### Step 1: Accessibility Tree Reader

Read the focused app's UI tree via macOS Accessibility API.
Output as indented text with element type, label, and center position.

```
[AX] app="Safari" title="GitHub"
  window "GitHub - snolab/CapsLockX" 0,38 1440x862
    toolbar 0,38 1440,52
      button "Back" 45,64
      button "Forward" 75,64
      textfield "Address" 400,64 "https://github.com/snolab/CapsLockX"
      button "Reload" 750,64
    webarea 0,90 1440x810
      heading "CapsLockX" 200,150
      link "Code" 100,180
      link "Issues" 160,180
      link "Pull requests" 250,180
      text "A productivity tool..." 200,220
```

This is ~20-50 tokens for a typical window — extremely token-efficient
compared to screenshots (~1000 tokens).

**Implementation**: `AXUIElementCopyAttributeValue` tree walk.
Already partially implemented in CapsLockX (`output.rs` has AX helpers).

### Step 2: Streaming Parser (winnow)

Minimal grammar for MVP:

```
command  = key_cmd | mouse_cmd | wait_cmd | comment
key_cmd  = "k" SP (string | mods? keyname)
mouse_cmd = "m" SP (coords | click | scroll)
wait_cmd = "w" SP duration
comment  = "#" text
```

File: `rs/core/src/agent/lang.rs`

### Step 3: LLM Streaming Client

Connect to OpenAI-compatible SSE endpoint.
Feed tokens into parser as they arrive.

```rust
// Pseudo-code
let stream = llm_client.chat_stream(messages).await;
while let Some(token) = stream.next().await {
    parser.feed(&token);
    while let Some(cmd) = parser.next_command() {
        let result = actor.execute(cmd);
        echo_tx.send(result);
    }
}
```

File: `rs/core/src/agent/stream.rs`

### Step 4: Actor (Command Executor)

Map parsed commands to existing CapsLockX platform methods:

```rust
fn execute(&self, cmd: Command) -> EchoResult {
    match cmd {
        Command::KeyTap { key, mods } => {
            self.platform.key_tap_with_mods(key, &mods, 1);
            EchoResult::ok("k", &format!("{}", key))
        }
        Command::MouseMove { x, y } => {
            self.platform.mouse_move(x, y);
            EchoResult::ok("m", &format!("{} {}", x, y))
        }
        Command::MouseClick { button } => {
            self.platform.mouse_click(button);
            EchoResult::ok("m", "c")
        }
        Command::Wait { duration } => {
            std::thread::sleep(duration);
            EchoResult::ok("w", &format!("{}ms", duration.as_millis()))
        }
        _ => EchoResult::skip()
    }
}
```

File: `rs/core/src/agent/actor.rs`

### Step 5: Echo (Result Feedback)

After each command executes, echo the result back to the LLM:

```
[OK t=0.23] k a                    # key 'a' tapped successfully
[OK t=0.45] m 400 300              # mouse moved to 400,300
[OK t=0.46] m c                    # clicked
[ERR t=0.50] k nonexistent         # unknown key name
[AX t=0.70] focused="textfield"    # new focus state after click
```

The echo is appended to the LLM's conversation as an assistant message
or injected into the next user message. This closes the loop — the LLM
sees the effect of its actions.

**Key insight**: After a click, re-read the AX tree focused element and
echo it. This tells the LLM what it clicked on and whether focus moved.

### Step 6: Hotkey + Voice Activation

`CLX+A` triggers the agent:
1. Capture AX tree of focused app
2. Listen for voice prompt (existing STT)
3. Build context: system prompt + AX tree + voice transcript
4. Start LLM streaming → parser → actor → echo loop
5. `ESC` cancels

### Step 7: System Prompt

```
You are CLX Agent. You control the computer by outputting CLX commands.
Commands execute IMMEDIATELY as you stream them.

## Commands
k a          — tap key 'a'
k c-c        — Ctrl+C
k "text"     — type string
m 400 300    — move mouse to (400,300)
m c          — left click
m c:r        — right click
w 200ms      — wait 200ms

## Current Screen (Accessibility Tree)
[injected AX tree here]

## User Request
[voice transcript here]

## Rules
1. Output ONLY CLX commands, no prose.
2. After clicking, wait 200ms for UI to update.
3. Use the element positions from the accessibility tree.
4. You will see [OK] echoes confirming each command executed.
5. If something goes wrong, try a different approach.
```

## File Structure

```
rs/core/src/agent/
  mod.rs          — AgentInterpreter, hotkey handler, main loop
  lang.rs         — winnow streaming parser
  actor.rs        — command executor, echo generation
  stream.rs       — LLM SSE streaming client
  ax_tree.rs      — accessibility tree reader (macOS)
```

## Success Criteria

The MVP is working when you can:
1. Press `CLX+A`
2. Say "click on the Issues tab"
3. The agent reads the AX tree, finds "Issues" link at position (160,180)
4. Streams: `m 160 180\nw 100ms\nm c\n`
5. Mouse moves and clicks the Issues tab
6. Echo confirms: `[OK] m 160 180`, `[OK] m c`, `[AX] focused="link Issues"`
7. The page navigates to Issues
