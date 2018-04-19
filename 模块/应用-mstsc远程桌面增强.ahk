
If(!CapsX)
    ExitApp


; SetTimer, asdfwer, 0
; asdfwer:
;     SetTimer, asdfwer, Off
;     If ( last_mstsc ) {
;         WinSet, Bottom, , ahk_id %last_mstsc%
;     }
;     WinShow, ahk_class Shell_TrayWnd ahk_exe explorer.exe
;     WinSet, AlwaysOnTop, On, ahk_class Shell_TrayWnd ahk_exe explorer.exe
;     SetTimer, asdfwer, O
;     Return


SetTimer, tobottom, 1
tobottom:
    WinWaitActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
    WinGet last_mstsc
    WinSet Bottom, , ahk_id %last_mstsc%
    WinWaitNotActive ahk_id %last_mstsc%
    Return
; SetTimer, rdplayer, 1000
; rdplayer:
;     WinWaitActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
;     ;WinMinimizeAll
;     WinGet, last_mstsc
;     WinSet, Bottom, , ahk_id %last_mstsc%
;     ;WinActivate, ahk_id %last_mstsc%
;     WinWaitNotActive, ahk_id %last_mstsc%
;     Return

global last_mstsc := 0

>^RAlt::
>!RCtrl::
    global last_mstsc
    If ( last_mstsc ) {
        WinActivate ahk_id %last_mstsc%
        ;WinActivate, ahk_id %last_mstsc%
    }Else{
        WinActivate ahk_class TscShellContainerClass ahk_exe mstsc.exe
    }
    Return


; SNO20503 - 远程桌面连接 ahk_class TscShellContainerClass ahk_exe mstsc.exe
#IfWinActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
    >^RAlt::
    >!RCtrl::
        global last_mstsc
        WinGet, last_mstsc
        WinSet, Bottom, , ahk_id %last_mstsc%
        WinMinimizeAllUndo
        Return
    
    <!RAlt::
    >!LAlt::
        global last_mstsc
        WinGet, last_mstsc
        WinMinimize ahk_id %last_mstsc%
        Return