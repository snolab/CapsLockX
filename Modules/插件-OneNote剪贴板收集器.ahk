;
; Save as UTF8 With BOM
;
SetTitleMatchMode, RegEx

Return
; #IfWinExist CollectClipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE
;     ~^c::
;         Clipboard := ""
;         ClipWait, 2, 1
;         If ErrorLevel
;         {
;             MsgBox, The attempt 2 copy text onto the clipboard failed.
;             Return
;         }

;         winid := WinExist("CollectClipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")
;         If(winid){
;             WinGet, current, ID, A
;             WinActivate, ahk_id %winid%
;             WinWaitActive, ahk_id %winid%
;             SendEvent, ^{End}!{Enter 2}

;             FormatTime, TimeString, , [yyyyMMdd.HHmmss]
;             SendEvent, {Text}%TimeString%
;             SendEvent, ^v

;             WinActivate, ahk_id %current%
;         }
;         Return

#IfWinExist 剪贴板.*|Clipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE

~^c::
    hwndOneNote := WinExist("剪贴板.*|Clipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")
    if(!hwndOneNote)
        Return
    ; 通常在弹起时触发
    Clipboard := ""
    ClipWait, 2, 1 ; 2 secons
    if ErrorLevel {
        ; MsgBox, The attempt 2 copy text onto the clipboard failed.
        Return
    }
    WinGet, current, ID, A
    WinActivate, ahk_id %hwndOneNote%
    FormatTime, timeString, , (yyyyMMdd.HHmmss)
    SendEvent, ^{End}{Enter}
    SendEvent, {text}%timeString%
    SendEvent, ^v
    Sleep 16
    WinActivate, ahk_id %current%
Return
