; #Requires AutoHotkey v1.0
; doesnt require capslockx
; run this by manually

return

^!v:: ClipboardStackPaste()

ClipboardStackPaste() {
    RegExMatch(clipboard, "O)([\s\S]*?)\s*\r?\n========+\r?\n\s*([\s\S]*)", ClipboardStackMatch)
    if (ClipboardStackMatch) {
        ; MsgBox % ClipboardStackMatch[1]
        clipboard := ""
        clipboard := ClipboardStackMatch[1]
        Send ^v
        Sleep 200
        clipboard := ClipboardStackMatch[2]
    } else {
        Send ^v
    }
    return
}
