; E2E test for CapsLockX cursor/selection keys
; Creates its own test window (Edit control) — no Notepad dependency
; Uses SPACE as CLX trigger
#SingleInstance Force
SetKeyDelay, 30, 30

pass := 0
fail := 0

Log(msg) {
    FileAppend, %msg%`n, %A_ScriptDir%\test-results.txt
}

FileDelete, %A_ScriptDir%\test-results.txt
Log("=== CLX E2E Test Started ===")

; --- Create test window with Edit control ---
Gui, TestWin:New, +Resize, CLX Test Window
Gui, TestWin:Font, s14, Consolas
Gui, TestWin:Add, Edit, vTestEdit w600 h200 Multi,
Gui, TestWin:Show, w640 h240
Sleep, 500

; Focus the edit control
GuiControl, TestWin:Focus, TestEdit
Sleep, 200

SimDelay := 100

; Helper: reset text and cursor
ResetText(text) {
    GuiControl, TestWin:, TestEdit, %text%
    Sleep, 100
    Send, ^a
    Sleep, 50
    Send, {Home}
    Sleep, 100
}

; Helper: get selected text via clipboard
GetSelected() {
    Clipboard :=
    Send, ^c
    ClipWait, 1
    return Clipboard
}

; ============================================================
; TEST 1: Space+L should move cursor right (no selection)
; ============================================================
Log("")
Log("--- TEST 1: Space+L (cursor right, no selection) ---")

ResetText("hello world test")

Send, {Space down}
Sleep, %SimDelay%
Send, {l down}
Sleep, 300
Send, {l up}
Sleep, %SimDelay%
Send, {Space up}
Sleep, 500

sel := GetSelected()
if (sel = "") {
    Log("PASS: No text selected")
    pass++
} else {
    Log("FAIL: Text was selected: " sel)
    fail++
}

; ============================================================
; TEST 2: Space+H should move cursor left (no selection)
; ============================================================
Log("")
Log("--- TEST 2: Space+H (cursor left, no selection) ---")

ResetText("hello world test")
Send, {End}
Sleep, 100

Send, {Space down}
Sleep, %SimDelay%
Send, {h down}
Sleep, 300
Send, {h up}
Sleep, %SimDelay%
Send, {Space up}
Sleep, 500

sel := GetSelected()
if (sel = "") {
    Log("PASS: No text selected")
    pass++
} else {
    Log("FAIL: Text was selected: " sel)
    fail++
}

; ============================================================
; TEST 3: Space+Shift+L should select text rightward
; ============================================================
Log("")
Log("--- TEST 3: Space+Shift+L (select right) ---")

ResetText("hello world test")

Send, {Space down}
Sleep, %SimDelay%
Send, {LShift down}
Sleep, %SimDelay%
Send, {l down}
Sleep, 500
Send, {l up}
Sleep, %SimDelay%
Send, {LShift up}
Sleep, %SimDelay%
Send, {Space up}
Sleep, 500

sel := GetSelected()
if (sel != "") {
    Log("PASS: Text selected: " sel)
    pass++
} else {
    Log("FAIL: No text selected")
    fail++
}

; ============================================================
; TEST 4: Shift+Space+L (Shift first) should select
; ============================================================
Log("")
Log("--- TEST 4: Shift+Space+L (shift first, select right) ---")

ResetText("hello world test")

Send, {LShift down}
Sleep, %SimDelay%
Send, {Space down}
Sleep, %SimDelay%
Send, {l down}
Sleep, 500
Send, {l up}
Sleep, %SimDelay%
Send, {Space up}
Sleep, %SimDelay%
Send, {LShift up}
Sleep, 500

sel := GetSelected()
if (sel != "") {
    Log("PASS: Text selected: " sel)
    pass++
} else {
    Log("FAIL: No text selected")
    fail++
}

; ============================================================
; TEST 5: Space+K should move cursor up (no selection)
; ============================================================
Log("")
Log("--- TEST 5: Space+K (cursor up, no selection) ---")

ResetText("hello world test")
Send, {End}
Sleep, 100

Send, {Space down}
Sleep, %SimDelay%
Send, {k down}
Sleep, 300
Send, {k up}
Sleep, %SimDelay%
Send, {Space up}
Sleep, 500

sel := GetSelected()
if (sel = "") {
    Log("PASS: No text selected")
    pass++
} else {
    Log("FAIL: Text was selected: " sel)
    fail++
}

; ============================================================
; TEST 6: Space+J should move cursor down (no selection)
; ============================================================
Log("")
Log("--- TEST 6: Space+J (cursor down, no selection) ---")

ResetText("hello world test")

Send, {Space down}
Sleep, %SimDelay%
Send, {j down}
Sleep, 300
Send, {j up}
Sleep, %SimDelay%
Send, {Space up}
Sleep, 500

sel := GetSelected()
if (sel = "") {
    Log("PASS: No text selected")
    pass++
} else {
    Log("FAIL: Text was selected: " sel)
    fail++
}

; ============================================================
; TEST 7: Space+Shift+H should select text leftward
; ============================================================
Log("")
Log("--- TEST 7: Space+Shift+H (select left) ---")

ResetText("hello world test")
Send, {Right 5}
Sleep, 200

Send, {Space down}
Sleep, %SimDelay%
Send, {LShift down}
Sleep, %SimDelay%
Send, {h down}
Sleep, 500
Send, {h up}
Sleep, %SimDelay%
Send, {LShift up}
Sleep, %SimDelay%
Send, {Space up}
Sleep, 500

sel := GetSelected()
if (sel != "") {
    Log("PASS: Text selected leftward: " sel)
    pass++
} else {
    Log("FAIL: No text selected")
    fail++
}

; ============================================================
; TEST 8: After all tests, Shift should NOT be stuck
; ============================================================
Log("")
Log("--- TEST 8: Shift not stuck after all tests ---")
Sleep, 500

GuiControl, TestWin:, TestEdit,
Sleep, 100
GuiControl, TestWin:Focus, TestEdit
Sleep, 100
Send, testchar
Sleep, 300
Send, ^a
Sleep, 100
sel := GetSelected()
if (sel = "testchar") {
    Log("PASS: Shift is not stuck (typed lowercase)")
    pass++
} else {
    Log("FAIL: Shift may be stuck, got: " sel)
    fail++
}

; ============================================================
; Summary
; ============================================================
Log("")
total := pass + fail
Log("=== RESULTS: " pass "/" total " passed, " fail " failed ===")

Gui, TestWin:Destroy
ExitApp
