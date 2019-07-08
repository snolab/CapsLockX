
If(!CapslockX)
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

global mstsc_st := "bottom"
; 如果当前操作的远程桌面窗口是全屏窗口，则自动置底，这样可以跟当前电脑桌面上的窗口共同操作
SetTimer, toggleBottomOrTop, 1
toggleBottomOrTop:
    ; 不用担心 SetTimer 消耗 CPU 性能，因为它会在这一步阻塞
    WinWaitActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
    WinGet last_mstsc
    WinGet mm, MinMax, ahk_id %last_mstsc%
    WinGetPos, X, Y, Width, Height, A
    SysGet, VirtualWidth, 78
    SysGet, VirtualHeight, 79

    ; Tooltip %X% %Y% %VirtualWidth% %VirtualHeight% %Width% %Height% %A_ScreenWidth% %A_ScreenHeight%
    ; 如果当前操作的远程桌面窗口是全屏窗口，就把它置底
    
    if(VirtualWidth == Width && VirtualHeight == Height){
        WinSet Bottom, , ahk_id %last_mstsc%
    }
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

; >^RAlt::
; >!RCtrl::
;     global last_mstsc
;     If ( last_mstsc ) {
;         WinActivate ahk_id %last_mstsc%
;         ;WinActivate, ahk_id %last_mstsc%
;     }Else{
;         WinActivate ahk_class TscShellContainerClass ahk_exe mstsc.exe
;     }
;     Return


<!RAlt::
>!LAlt::
    global last_mstsc
    WinGet, last_mstsc
    WinMinimize ahk_id %last_mstsc%
    Return

; ahk_class TscShellContainerClass ahk_exe mstsc.exe
#IfWinActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
    
    >^RAlt::
    >!RCtrl::
        global last_mstsc
        WinGet, last_mstsc
        WinSet, Bottom, , ahk_id %last_mstsc%
        WinMinimizeAllUndo
        Return
    
    ; 左右Alt一起按，切换最小化远程桌面窗口
    <!RAlt::
    >!LAlt::
        global last_mstsc
        WinGet, last_mstsc
        WinMinimize ahk_id %last_mstsc%
        Return