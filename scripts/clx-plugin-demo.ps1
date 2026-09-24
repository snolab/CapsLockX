# A CLX plugin, in full.
#
# The entire contract is: print effects to stdout. No SDK, no linking, no ABI,
# no manifest. This file is a plugin because it prints — it could just as well
# be Python, Go, or a shell script, and CLX would not know the difference.
#
#   clx plugin powershell -ExecutionPolicy Bypass -File scripts\clx-plugin-demo.ps1
#
# Point the cursor at a text field first: the effects land wherever focus is.
#
# What it demonstrates, in order:
#   1. typing                  — the whole of a simple plugin
#   2. streaming revision      — a draft corrected in place, the shape every
#                                speech recogniser produces
#   3. keys and timing         — the rest of the vocabulary
#
# Note what is *not* here: nothing tells CLX what this plugin is for. It emits
# effects; CLX performs them; neither knows anything about the other's purpose.

param(
    [double]$Speed = 1.0   # multiplier on the pauses, for demoing slowly
)

function Pause-Beat([int]$ms) { Start-Sleep -Milliseconds ([int]($ms * $Speed)) }

# ── 1. typing ───────────────────────────────────────────────────────────────
# `k "…"` types a string and begins a revisable run.
'k "CLX plugin demo: "'
Pause-Beat 400

# ── 2. streaming revision ───────────────────────────────────────────────────
# A recogniser emits a rough draft, then improves it. `retype` replaces the
# current run in place: CLX diffs against what it last typed and erases only
# the differing suffix, so this costs a few keystrokes rather than a rewrite.
#
# Watch the text change without flickering through a full delete.
'retype "CLX plugin demo: the quick braun"'
Pause-Beat 500
'retype "CLX plugin demo: the quick brown fox"'
Pause-Beat 500
'retype "CLX plugin demo: The quick brown fox."'
Pause-Beat 500

# `commit` ends the run. After this, a `retype` has nothing to revise and
# simply types — which is what stops one plugin backspacing over another's
# output.
'commit'

# ── 3. the rest of the vocabulary ───────────────────────────────────────────
'k enter'
'k "Keys: "'
'w 200ms'
'k "shift makes this "'
'k s-a'          # Shift+A — a chord
'k " and waits are effects too."'
'w 300ms'
'k enter'
'k "Done. Nothing here knew what this plugin was for."'

# A plugin exits when it is finished. CLX closes the revisable run for it,
# however it ends — including badly — so a plugin that dies mid-sentence
# cannot leave the next one revising text that is no longer its own.
