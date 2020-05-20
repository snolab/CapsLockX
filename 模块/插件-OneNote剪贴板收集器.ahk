;
; Save as UTF8 With BOM
;
Return
#IfWinExist CollectClipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE
    ~^c::
        Clipboard := ""
        ClipWait, 2, 1
        If ErrorLevel
        {
            MsgBox, The attempt 2 copy text onto the clipboard failed.
            Return
        }
        
        winid := WinExist("CollectClipboard ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")
        If(winid){
            WinGet, current, ID, A
            WinActivate, ahk_id %winid%
            WinWaitActive, ahk_id %winid%
            SendEvent, ^{End}!{Enter 2}

            FormatTime, TimeString, , [yyyyMMdd.HHmmss]
            SendEvent, {Text}%TimeString%
            SendEvent, ^v
            
            WinActivate, ahk_id %current%
        }
        return