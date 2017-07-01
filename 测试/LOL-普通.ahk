Process, Priority,,high         ;脚本高优先级
SendMode Input
CoordMode Mouse, Relative
;#NoTrayIcon                        ;隐藏托盘图标
;#NoEnv                              ;不检查空变量是否为环境变量
;#Persistent                     ;让脚本持久运行(关闭或ExitApp)
#SingleInstance Force               ;跳过对话框并自动替换旧实例
;#MaxHotkeysPerInterval 300      ;时间内按热键最大次数
;#InstallMouseHook

!F12:: ExitApp
;#IfWinActive League of Legends (TM) Client ahk_class RiotWindowClass ahk_exe League of Legends.exe

#IfWinActive ahk_class RiotWindowClass

`::
    WinGetPos, X, Y, Width, Height, A
    MouseGetPos, x, y
    MouseClick, R, % Width - x, Height - y, 1, 0
    MouseClick, R, x, y, 1, 0
    Return
; ~a::
;     MouseClick, R
; ;     Return
; LButton::RButton
; RButton::LButton