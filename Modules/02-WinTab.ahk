
If(!CapsX)
    ExitApp
Return

#Delete:: Send {CtrlDown}#{F4}{CtrlUp}
#Insert:: Send {CtrlDown}#d{CtrlUp}

; 当前窗口置顶
#0:: WinSet, AlwaysOnTop, Toggle, A

; 切换桌面
#[:: Send {CtrlDown}#{Left}{CtrlUp}
#]:: Send {CtrlDown}#{Right}{CtrlUp}

; 确保WinTab模块优先级比Mouse高，否则此处 wasd 无效

; 帮助：
; 条件：WinActive ahk_class MultitaskingViewFrame 
; 
#IfWinActive ahk_class MultitaskingViewFrame

	!F1:: tips("WASD 切换窗口焦点")
	F1:: tips("WASD 切换窗口焦点`nQE切换桌面")
    ; 在 Alt+Tab 下, WASD 模拟方向键
    !a:: Left
    !d:: Right
    !w:: Up
    !s:: Down

    ; 在 Win10 下的 Win+Tab 界面，WASD 切换窗口焦点
	; 以及在窗口贴边后，WASD 切换窗口焦点
	
    ; 模拟方向键
    w:: Send {Up}
    a:: Send {Left}
    s:: Send {Down}
    d:: Send {Right}
	
	; 切换桌面概览
	q::Send ^#{Left}
	e::Send ^#{Right}

	; 新建桌面
	; v::Send ^#d

	; 删除桌面
	z::Send ^#{F4}

	; 关掉窗口
	x::Send ^w{Right} 

	; 移到除了自己的第x个桌面（或新建桌面）
	1::Send {AppsKey}m{Down 0}{Enter}
	2::Send {AppsKey}m{Down 1}{Enter}
	3::Send {AppsKey}m{Down 2}{Enter}
	4::Send {AppsKey}m{Down 3}{Enter}
	5::Send {AppsKey}m{Down 4}{Enter}
	6::Send {AppsKey}m{Down 5}{Enter}
	7::Send {AppsKey}m{Down 6}{Enter}
	8::Send {AppsKey}m{Down 7}{Enter}
	9::Send {AppsKey}m{Down 8}{Enter}
	; 移到除了自己的最后一个桌面（或新建桌面）
	0::Send {AppsKey}m{Up 2}{Enter}

	; 移到新建桌面
	v::Send {AppsKey}mn{Sleep 16}+{Tab}
	
	; 移到新建桌面，并激活窗口
	c::Send {AppsKey}mn{Enter}