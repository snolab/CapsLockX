; ========== CapsLockX ==========
; 名称：桌面QQ增强
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; 

If(!CapsLockX)
    ExitApp
Return

#IfWinActive .*的资料 ahk_class TXGuiFoundation ahk_exe QQ.exe
/:: CapslockXShowHelp("
(
QQ 资料卡界面
F2:: ; 改备注名
F3:: ; 改分组
F4:: ; 加备注（手机号等）
)")

F2:: ; 改备注名
; 这里的 ^+{Tab 2}{Tab 1} 是利用QQ 的 Edit 控件无法使用 Ctrl + Tab 跳转离开的特性，来重置光标到备注栏。
Send {Tab 10}^+{Tab 2}{Tab 1}{Enter}
Return

F3:: ; 改分组
Send {Tab 10}^+{Tab 2}{Tab 2}{Enter}
Return

F4:: ; 加备注（手机号等）
Send +{Tab}{Enter}
Return



#IfWinActive .*等\d+个会话 ahk_class TXGuiFoundation ahk_exe QQ.exe

/:: CapslockXShowHelp("
(
QQ 多人会话界面
F2:: ; 看资料
!f:: ; 点开左上角搜索（仅主屏有效）
!w:: ; 开出小窗口
!r:: ; 快速点击接收文件
!h:: ; 聊天记录
)")

F2:: ; 看资料
Send +{Tab 3}{Enter}
Return

!f:: ; 点开左上角搜索
CoordMode, Mouse, Client
Click 32, 32
Return

!d:: ; 定位到功能栏
SendInput, +{Tab 4}{Enter}
return

!b:: ; 屏蔽此人
SendInput, +{Tab 4}{Left}{Enter}
return

!w:: ; 开出小窗口
CoordMode, Mouse, Client
MouseClickDrag, Left, 32, 128, 256, 128, 0
WinActivate .*\d+个会话 ahk_class TXGuiFoundation ahk_exe QQ.exe
Return


!r:: ; 快速点击接收文件
Send 1!s{Tab 10}{Space}!s
Return


!n:: ; 群通知设定 (或抖窗)
SendInput, {Tab 5}{Right 6}{Enter}
return

#IfWinActive ahk_class TXGuiFoundation ahk_exe QQ.exe
/:: CapslockXShowHelp("
(
QQ 单人会话界面
!m:: ; 屏蔽鼠标指向的群
!r:: ; 快速点击接收文件
F2:: ; 查看这个人的资料
)")

!m:: ; 屏蔽鼠标指向的群
Send {RButton}{Down 2}{Right}{Up}{Enter}
MouseMove, 0, -86, 0, R
Return

!r:: ; 快速点击接收文件
Send 1!s+{Tab 8}{Space}!s
Return

F2:: ; 查看这个人的资料
Send +{Tab 2}{Enter}
Return
