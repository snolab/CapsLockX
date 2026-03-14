; Manual QA test for CapsLockX Shift+HJKL selection
; Shows instructions, waits for user input, verifies results
#SingleInstance Force

pass := 0
fail := 0
total := 0

Log(msg) {
    FileAppend, %msg%`n, %A_ScriptDir%\test-results.txt
}

FileDelete, %A_ScriptDir%\test-results.txt
Log("=== CLX Manual QA Test ===")

; --- Create test window ---
Gui, QA:New, +Resize, CLX Manual QA
Gui, QA:Font, s12, Consolas
Gui, QA:Add, Text, vInstructions w600 h60,
Gui, QA:Add, Edit, vTestEdit w600 h100 Multi,
Gui, QA:Add, Text, vStatus w600 h30 cBlue,
Gui, QA:Add, Text, vScore w600 h30,
Gui, QA:Add, Button, gNextTest vBtnNext w200 h35, Ready (Enter)
Gui, QA:Show, w640 h320
Sleep, 300

; Map Enter to the button
Gui, QA:+LastFound
WinGet, qaHwnd, ID
#IfWinActive ahk_id %qaHwnd%
; no hotkeys needed, button handles it

; ── Test definitions ──
tests := []

t := {}
t.name := "TEST 1: CLX+L (cursor right)"
t.setup := "hello world"
t.cursorPos := 0
t.instructions := "Hold SPACE, tap L a few times, release SPACE.`nCursor should move RIGHT without selecting."
t.expectSelection := false
tests.Push(t)

t := {}
t.name := "TEST 2: CLX+H (cursor left)"
t.setup := "hello world"
t.cursorPos := 11
t.instructions := "Hold SPACE, tap H a few times, release SPACE.`nCursor should move LEFT without selecting."
t.expectSelection := false
tests.Push(t)

t := {}
t.name := "TEST 3: CLX+Shift+L (select right)"
t.setup := "hello world test"
t.cursorPos := 0
t.instructions := "Hold SPACE, hold SHIFT, tap L a few times, release all.`nText should be SELECTED rightward (highlighted)."
t.expectSelection := true
tests.Push(t)

t := {}
t.name := "TEST 4: Shift+CLX+L (shift first)"
t.setup := "hello world test"
t.cursorPos := 0
t.instructions := "Hold SHIFT first, then hold SPACE, tap L a few times, release all.`nText should be SELECTED rightward."
t.expectSelection := true
tests.Push(t)

t := {}
t.name := "TEST 5: CLX+Shift+H (select left)"
t.setup := "hello world test"
t.cursorPos := 10
t.instructions := "Hold SPACE, hold SHIFT, tap H a few times, release all.`nText should be SELECTED leftward."
t.expectSelection := true
tests.Push(t)

t := {}
t.name := "TEST 6: CLX+K (cursor up)"
t.setup := "line one`r`nline two"
t.cursorPos := 18
t.instructions := "Hold SPACE, tap K, release SPACE.`nCursor should move UP without selecting."
t.expectSelection := false
tests.Push(t)

t := {}
t.name := "TEST 7: CLX+J (cursor down)"
t.setup := "line one`r`nline two"
t.cursorPos := 0
t.instructions := "Hold SPACE, tap J, release SPACE.`nCursor should move DOWN without selecting."
t.expectSelection := false
tests.Push(t)

t := {}
t.name := "TEST 8: Shift not stuck"
t.setup := ""
t.cursorPos := 0
t.instructions := "Type 'test' normally (no CLX).`nIt should appear lowercase, NOT 'TEST'."
t.expectSelection := false
t.checkLowercase := true
tests.Push(t)

currentTest := 0
SetupNextTest()
return

; ── Test flow ──

SetupNextTest() {
    global tests, currentTest, pass, fail, total
    currentTest++

    if (currentTest > tests.Count()) {
        ; All done
        total := pass + fail
        result := pass "/" total " passed"
        GuiControl, QA:, Instructions, All tests complete!
        GuiControl, QA:, Status, % result
        GuiControl, QA:, Score, % "PASS=" pass " FAIL=" fail
        GuiControl, QA:, BtnNext, Close
        Log("")
        Log("=== RESULTS: " result " ===")
        return
    }

    t := tests[currentTest]
    GuiControl, QA:, Instructions, % t.name "`n" t.instructions
    GuiControl, QA:, TestEdit, % t.setup
    GuiControl, QA:, Status, % "Perform the action above, then click Ready"
    GuiControl, QA:, Score, % "Progress: " (currentTest - 1) "/" tests.Count() "  PASS=" pass " FAIL=" fail

    ; Set cursor position
    GuiControl, QA:Focus, TestEdit
    Sleep, 100
    SendMessage, 0xB1, % t.cursorPos, % t.cursorPos, Edit1, CLX Manual QA ; EM_SETSEL
    Sleep, 100
}

NextTest:
    global tests, currentTest, pass, fail

    if (currentTest > tests.Count()) {
        Gui, QA:Destroy
        ExitApp
    }

    t := tests[currentTest]

    if (t.HasKey("checkLowercase")) {
        ; Special test: check what user typed
        GuiControlGet, typed, QA:, TestEdit
        if (typed = "test") {
            Log(t.name ": PASS (typed lowercase)")
            pass++
            GuiControl, QA:, Status, PASS - typed lowercase correctly
        } else {
            Log(t.name ": FAIL (got: " typed ")")
            fail++
            GuiControl, QA:, Status, % "FAIL - expected 'test', got '" typed "'"
        }
    } else {
        ; Check if text is selected
        GuiControl, QA:Focus, TestEdit
        Clipboard :=
        Send, ^c
        ClipWait, 0.5
        hasSelection := (Clipboard != "")

        if (t.expectSelection) {
            if (hasSelection) {
                Log(t.name ": PASS (selected: " Clipboard ")")
                pass++
                GuiControl, QA:, Status, % "PASS - selected: " Clipboard
            } else {
                Log(t.name ": FAIL (no selection)")
                fail++
                GuiControl, QA:, Status, FAIL - no text was selected
            }
        } else {
            if (!hasSelection) {
                Log(t.name ": PASS (no selection)")
                pass++
                GuiControl, QA:, Status, PASS - no selection (cursor moved only)
            } else {
                Log(t.name ": FAIL (unexpected selection: " Clipboard ")")
                fail++
                GuiControl, QA:, Status, % "FAIL - unexpected selection: " Clipboard
            }
        }
    }

    Sleep, 1000
    SetupNextTest()
return

QAGuiClose:
    total := pass + fail
    Log("")
    Log("=== RESULTS: " pass "/" total " passed (aborted early) ===")
    Gui, QA:Destroy
    ExitApp
return
