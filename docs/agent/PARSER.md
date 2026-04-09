# Streaming Parser Design

## Technology Choice: **winnow** with `Partial<&str>`

### Why winnow?

| Criteria | winnow | nom | pest | hand-written |
|---|---|---|---|---|
| Native streaming | Yes (`Partial`) | Yes (streaming modules) | No | Manual |
| Performance | **627 MB/s** | 213 MB/s | 57 MB/s | Varies |
| API ergonomics | Clean, modern | Good but older | Grammar files | Full control |
| Maturity | Good (nom successor) | Excellent | Good | N/A |
| Error recovery | Built-in | Manual | Good | Manual |

winnow is 3x faster than nom, has a cleaner streaming API via `Partial`,
and is the maintained successor to nom. For our use case (tiny DSL
statements, streaming from LLM), it's the clear winner.

### Alternatives considered and rejected

- **pest**: No streaming support. Requires full input. Too slow.
- **tree-sitter**: Incremental but edit-based, not stream-based. Overkill.
- **chumsky**: No streaming. Good errors but wrong model.
- **lalrpop**: No streaming. LR(1) doesn't expose statement boundaries.

## Streaming Parse Loop

```rust
use winnow::stream::Partial;
use winnow::error::ErrMode;

fn parse_loop(rx: Receiver<String>, cmd_tx: Sender<Command>) {
    let mut buffer = String::new();

    while let Ok(chunk) = rx.recv() {
        buffer.push_str(&chunk);

        loop {
            let input = Partial::new(buffer.as_str());
            match parse_statement(input) {
                Ok((remaining, cmd)) => {
                    // Statement complete — execute immediately
                    cmd_tx.send(cmd).unwrap();
                    let consumed = buffer.len() - remaining.len();
                    buffer.drain(..consumed);
                }
                Err(ErrMode::Incomplete(_)) => {
                    // Need more data — wait for next chunk
                    break;
                }
                Err(_) => {
                    // Parse error — skip to next newline
                    if let Some(nl) = buffer.find('\n') {
                        eprintln!("[agent] skip bad line: {}", &buffer[..nl]);
                        buffer.drain(..=nl);
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
```

### Key properties:

1. **Zero-copy where possible** — winnow operates on `&str` slices
2. **Immediate execution** — each complete statement dispatches instantly
3. **Error resilience** — bad lines are skipped, never crash
4. **No re-parsing overhead** — consumed input is drained from buffer
5. **Backpressure** — if executor is busy, channel back-pressures naturally

## Grammar Implementation Sketch

```rust
use winnow::prelude::*;
use winnow::combinator::{alt, preceded, opt, repeat};
use winnow::token::{take_while, one_of};
use winnow::ascii::{float, space0, space1, line_ending};
use winnow::stream::Partial;

type Stream<'a> = Partial<&'a str>;

fn statement<'a>(input: &mut Stream<'a>) -> PResult<Command> {
    let _ = space0.parse_next(input)?;
    alt((
        preceded('#', take_while(0.., |c| c != '\n')).map(|_| Command::Nop),
        key_cmd,
        mouse_cmd,
        pad_cmd,
        midi_cmd,
        wait_cmd,
        repeat_cmd,
    )).parse_next(input)?;
    // Consume terminator
    alt((line_ending, ";")).parse_next(input)?;
    Ok(cmd)
}

fn key_cmd<'a>(input: &mut Stream<'a>) -> PResult<Command> {
    let _ = 'k'.parse_next(input)?;
    let _ = space1.parse_next(input)?;
    // ... parse modifiers, key name, d:/u: prefix, string literal
}

fn mouse_cmd<'a>(input: &mut Stream<'a>) -> PResult<Command> {
    let _ = 'm'.parse_next(input)?;
    let _ = space1.parse_next(input)?;
    // ... parse coordinates (absolute/relative/lerp), click, scroll
}
```

## Command Enum

```rust
enum Command {
    Nop,
    // Keyboard
    KeyTap { mods: Mods, key: Key },
    KeyDown { key: Key },
    KeyUp { key: Key },
    TypeString { text: String },
    // Mouse
    MouseMove { x: f64, y: f64, mode: MoveMode },
    MouseClick { button: MouseButton },
    MouseDown { button: MouseButton },
    MouseUp { button: MouseButton },
    MouseScroll { delta: i32 },
    // Gamepad
    PadStick { stick: Stick, x: f32, y: f32, lerp: Option<Duration> },
    PadTrigger { trigger: Trigger, value: f32 },
    PadButton { button: PadBtn, state: BtnState },
    PadDpad { direction: DpadDir },
    // MIDI
    MidiNoteOn { note: u8, velocity: u8 },
    MidiNoteOff { note: u8 },
    MidiCC { controller: u8, value: u8 },
    MidiPitchBend { value: u16 },
    MidiProgramChange { program: u8 },
    // Timing
    Wait { duration: Duration },
    Repeat { count: u32, body: Vec<Command> },
}

enum MoveMode {
    Absolute,                    // m 500 300
    Relative,                    // m +10 -5
    Lerp { duration: Duration }, // m ~500 300 200ms
}
```

## Threading Model

```
[LLM SSE Stream] ──(async)──→ [Stream Accumulator]
                                       │
                                 (crossbeam channel)
                                       │
                              [Parser Thread] ──(crossbeam)──→ [Executor Thread]
                                                                   (RT priority)
                                                                       │
                                                    ┌──────────┬───────┼───────┐
                                                    ▼          ▼       ▼       ▼
                                                Keyboard    Mouse    Pad     MIDI
```

- **Async task**: Reads SSE/WebSocket, forwards text chunks
- **Parser thread**: Normal priority, runs parse loop
- **Executor thread**: Real-time priority, injects input events
  - Uses `spin_sleep` for sub-ms timing
  - Handles lerp interpolation (smooth mouse/stick movement)
  - Pre-allocated event buffers
