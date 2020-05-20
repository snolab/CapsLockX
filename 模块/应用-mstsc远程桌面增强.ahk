
If(!CapslockX)
    ExitApp
global last_mstsc := 0

; ; 如果当前操作的远程桌面窗口是全屏窗口，则自动置底，这样可以跟当前电脑桌面上的窗口共同操作
; SetTimer, toggleBottomOrTop, 1
Return
; toggleBottomOrTop:
;     ; 不用担心 SetTimer 消耗 CPU 性能，因为它会在这一步阻塞
;     WinWaitActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
;     WinGet last_mstsc
;     WinGet mm, MinMax, ahk_id %last_mstsc%
;     WinGetPos, X, Y, Width, Height, A
;     SysGet, VirtualWidth, 78
;     SysGet, VirtualHeight, 79
;     ; Tooltip %X% %Y% %VirtualWidth% %VirtualHeight% %Width% %Height% %A_ScreenWidth% %A_ScreenHeight%
;     ; 如果当前操作的远程桌面窗口是全屏窗口，就把它置底
;     if(VirtualWidth == Width && VirtualHeight == Height){
;         WinSet Bottom, , ahk_id %last_mstsc%
;     }
;     WinWaitNotActive ahk_id %last_mstsc%
;     Return

; 左右Alt一起按 显示当前mstsc窗口
<!RAlt::
>!LAlt::
    ; WinGet, last_mstsc
    WinActivate ahk_id %last_mstsc%
    ; WinSet, TopMost, , ahk_id %last_mstsc%
    ; WinMinimizeAllUndo
    Return


; ahk_class TscShellContainerClass ahk_exe mstsc.exe
#IfWinActive ahk_class TscShellContainerClass ahk_exe mstsc.exe
    ; 左右Alt一起按 前置当前mstsc窗口
    <!RAlt::
    >!LAlt::
        WinGet, last_mstsc
        WinRestore, ahk_id %last_mstsc%
        WinSet, Top, , ahk_id %last_mstsc%
        ; WinShow,  ahk_id %last_mstsc%
        Return
    
    ; 左Ctrl + 右Alt 使远程窗口最小化并失去焦点，显示其它窗口
    <^RAlt::
    >!LCtrl::
        WinGet, last_mstsc
        ; WinMinimizeAllUndo
        WinMinimize ahk_id %last_mstsc%
        ; WinRestore, ahk_id %last_mstsc%
        ; 失去焦点
        WinHide,  ahk_id %last_mstsc%
        WinShow,  ahk_id %last_mstsc%
        Return
