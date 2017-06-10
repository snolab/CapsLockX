^!F12:: ExitApp

; #UseHook On

; ahk_class MultitaskingViewFrame ahk_exe explorer.exe
#PgUp:: Send {CtrlDown}#{Left}{CtrlUp}
#PgDn:: Send {CtrlDown}#{Right}{CtrlUp}


#Delete:: Send {CtrlDown}#{F4}{CtrlUp}
#Insert:: Send {CtrlDown}#d{CtrlUp}

; 当前窗口置顶
#0:: WinSet, AlwaysOnTop, Toggle, A

; 切换桌面
#[:: Send {CtrlDown}#{Left}{CtrlUp}
#]:: Send {CtrlDown}#{Right}{CtrlUp}

;#IfWinActive 任务切换 ahk_class MultitaskingViewFrame ahk_exe explorer.exe
#IfWinActive 任务切换 ahk_class MultitaskingViewFrame
    ; 模拟方向键
    !a:: Left
    !d:: Right
    !w:: Up
    !s:: Down

#IfWinActive 任务视图 ahk_class MultitaskingViewFrame ahk_exe explorer.exe
    ^F12:: ExitApp

    ; 模拟方向键
    w::Send {Up}
    a::Send {Left}
    s::Send {Down}
    d::Send {Right}
	
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



#IfWinActive 夜神模拟器 ahk_class Qt5QWindowIcon ahk_exe Nox.exe
	q::
	t::
		WinGetPos, X, Y, Width, Height, A
		If Y >= 0
			WinMove, A, , X, Y - Height / 2
		Else
			WinMove, A, , X, Y + Height / 2
		Return
	w:: Send {Up}
	a:: Send {Left}
	s:: Send {Down}
	d:: Send {Right}
	r::
		WinGetPos, X, Y, Width, Height, A
		MouseMove, 0, -120/1871*Height, 0, R
		Return
	f:: 
		WinGetPos, X, Y, Width, Height, A
		MouseMove, 0, 120/1871*Height, 0, R
		Return
	e:: 
		Click
		Return