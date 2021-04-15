#SingleInstance Force
SetBatchLines, -1
CoordMode, ToolTip, Client

Text=
(
当坐标模式为 窗口 或 客户区 时，默认使用活动窗口作为目标，同时也能自行指定。
更多可设置的参数，自行查看 btt() 函数中的说明。

When the CoordMode is window or client.
By default, the active window is used as target.
But you can also specify your own target.
For more parameters that can be set, see the description in btt() function.
)

Gui, +Hwndtarget		; get Hwnd
Gui, Font, s60
Gui, Add, Text, x0 y0 w800 h350 Center, 试试移动窗口。`n`nTry Move this window.
Gui, Font, s20
Gui, Add, Text, x0 y350 w800 h300, 此示例仅提供思路，你需要自行完善。`n`nThis example only provides ideas, you need to improve it by yourself.
Gui, Show, w800 h600 x0 y0 NA

SetTimer, Show, 10
Sleep, 10000
ExitApp

Show:
	btt(Text,800-1,600-1,,"Style3",{TargetHWND:target})
return

GuiClose:
	ExitApp
return