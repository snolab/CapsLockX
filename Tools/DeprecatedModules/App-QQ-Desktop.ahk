; ========== CapsLockX ==========
; 名称：桌面QQ增强
; 版本：v1.0.0
; 作者：snomiao
; 联系：snomiao@gmail.com
; 更新于: (20220217)
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

Return

#if QQ资料页面中()
QQ资料页面中(){
    return WinActive(".*的资料 ahk_class TXGuiFoundation ahk_exe QQ.exe")
}

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

#if QQ多人会话窗口中()
QQ多人会话窗口中(){
    return WinActive(".*等\d+个会话 ahk_class TXGuiFoundation ahk_exe QQ.exe")
}

F2:: ; 看资料
Send +{Tab 3}{Enter}
Return

!f:: ; 点开左上角搜索
CoordMode, Mouse, Client
Click 32, 32
Return

!d:: ; 定位到功能栏
SendInput, +{Tab 4}{Enter}
Return

!b:: ; 屏蔽此人
SendInput, +{Tab 4}{Left}{Enter}
Return

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
Return

#if QQ单人会话窗口中()
QQ单人会话窗口中(){
    return WinActive("ahk_class TXGuiFoundation ahk_exe QQ.exe")
}

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
