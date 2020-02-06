If(!CapslockX)
    ExitApp
Return

; 把当前窗口移到其它桌面
MoveActiveWindow(action){
	activeWin := WinActive("A")
	WinHide ahk_id %activeWin%
	Send %action%
	WinShow ahk_id %activeWin%
	WinActivate ahk_id %activeWin%
}
; x = 0 时新键桌面
MoveActiveWindowTo(x){
	; MoveActiveWindow("^#{Left 10}^#{Right "(0 == x ? "^#d" : x - 1) "}")
	MoveActiveWindow( 0 == x ? "^#d" : "^#{Left 10}^#{Right " (x - 1) "}")
}
SwitchToDesktop(x){
	Send % "^#{Left 10}^#{Right "(0 == x ? "^#d" : x - 1) "}"
}


#Delete:: Send {CtrlDown}#{F4}{CtrlUp}
#Insert:: Send {CtrlDown}#d{CtrlUp}

; 当前窗口置顶
#0:: WinSet, AlwaysOnTop, Toggle, A

; 切换桌面
#[:: Send {CtrlDown}#{Left}{CtrlUp}
#]:: Send {CtrlDown}#{Right}{CtrlUp}

^PrintScreen:: AppsKey
#If !!(CapslockXMode & CM_FN)
	o::
		; cmd := "taskkill /f /im ApplicationFrameHost.exe"
		; cmd := "taskkill /f /im ShellExperienceHost.exe"
		; Run, %cmd%
		WinActivate ahk_class Shell_TrayWnd ahk_exe explorer.exe
		; side by side 排列
		Send {AppsKey}i
		; stacked
		; Send {AppsKey}e
		; cascade
		; Send {AppsKey}d
		
		; restart explorer if needed
		; if(A_PriorHotkey == A_ThisHotkey && A_TimeSincePriorHotkey < 1000){
		; 	cmd := "sihost"
		; 	Run, %cmd%
		; }
		Return
	; Win+Tab
	\:: Send #{Tab} 

	; 切换桌面
	[:: Send ^#{Left}
	]:: Send ^#{Right}

	; 移动当前窗口到其它桌面
	+[:: MoveActiveWindow("^#{Left}")
	+]:: MoveActiveWindow("^#{Right}")

	; 切换当前窗口置顶并透明
	'::
		; WinGet, Var, Transparent, 150, A
		WinSet, Transparent, 200, A
		Winset, Alwaysontop, , A
		Return
	+'::
		; WinGet, Var, Transparent, 150, A
		WinSet, Transparent, 255, A
		Winset, Alwaysontop, , A
		Return

	; 所有窗口透明
	`;::
		; WinGet, Var, Transparent, 150, A
		WinSet, Transparent, 100, A
		Return
	
	`; Up::
		; WinGet, Var, Transparent, 150, A
		WinSet, Transparent, 255, A
		Return

	; /::
	; 	wid := WinActive("A")
	; 	WinGetText, title, ahk_id %wid%
	; 	ToolTip 已对 %title% (%wid%) 开启窗口关闭通知
	; 	Sleep, 1000
	; 	ToolTip
	; 	WinWaitClose ahk_id %wid%
	; 	MsgBox, 窗口已关闭：%title%
	; 	Return

	; 增删桌面
	-:: Send ^#{F4}
	; =:: Send ^#d
	=:: MoveActiveWindowTo(0)

	; 把当前窗口移到新建桌面
	0:: MoveActiveWindowTo(0)

	; 把当前窗口移到第x个桌面
	1:: MoveActiveWindowTo(1)
	2:: MoveActiveWindowTo(2)
	3:: MoveActiveWindowTo(3)
	4:: MoveActiveWindowTo(4)
	5:: MoveActiveWindowTo(5)
	6:: MoveActiveWindowTo(6)
	7:: MoveActiveWindowTo(7)
	8:: MoveActiveWindowTo(8)
	9:: MoveActiveWindowTo(9)

	; ; 把当前窗口移到第x个桌面
	; !1:: MoveActiveWindowTo(1)
	; !2:: MoveActiveWindowTo(2)
	; !3:: MoveActiveWindowTo(3)
	; !4:: MoveActiveWindowTo(4)
	; !5:: MoveActiveWindowTo(5)
	; !6:: MoveActiveWindowTo(6)
	; !7:: MoveActiveWindowTo(7)
	; !8:: MoveActiveWindowTo(8)
	; !9:: MoveActiveWindowTo(9)

	; 关闭窗口
	$Esc:: Send !{F4}
	$x:: Send ^w
    $!x:: Send !{F4}
    $^!x:: WinKill A

; 确保WinTab模块优先级比Mouse高，否则此处 wasd 无效
#If CapslockXMode == CM_CAPSX || CapslockXMode == CM_FN
	; 打开系统设定
    p::
		; ToolTip s %CapslockXMode%p
		Send #{Pause}
		Return

    `:: Send ^!q

	; 把当前窗口移到第x个桌面
	1:: SwitchToDesktop(1)
	2:: SwitchToDesktop(2)
	3:: SwitchToDesktop(3)
	4:: SwitchToDesktop(4)
	5:: SwitchToDesktop(5)
	6:: SwitchToDesktop(6)
	7:: SwitchToDesktop(7)
	8:: SwitchToDesktop(8)
	9:: SwitchToDesktop(9)

    ; 关闭窗口
    x:: Send ^w
    !x:: Send !{F4}

    ; Up:: Send #{Up}        ; 把热键让给转屏模块
    ; Down:: Send #{Down}    ; 把热键让给转屏模块
    ; Left:: Send #{Left}    ; 把热键让给转屏模块
    ; Right:: Send #{Right}  ; 把热键让给转屏模块
; 帮助：
; 条件：WinActive ahk_class MultitaskingViewFrame
;

; ~#Tab Up::
; 	CapsXTurnOff()
; 	Return


;
;
;
#If WinActive("ahk_class MultitaskingViewFrame")
    ; 在 Alt+Tab 下, WASD 模拟方向键 , 1803之后还可以用
    !a:: Left
    !d:: Right
    !w:: Up
    !s:: Down
    ; qe 切换桌面
	!q::
		SendEvent {Blind}{Enter}
		Sleep 200
		SendEvent ^#{Left}
		Return
	!e::
		SendEvent {Blind}{Enter}
		Sleep 200
		SendEvent ^#{Right}
		Return
	!+q::
		SendEvent {Blind}{Enter}
		Sleep 200
		MoveActiveWindow("^#{Left}")
		Return
	!+e::
		SendEvent {Blind}{Enter}
		Sleep 200
		MoveActiveWindow("^#{Right}")
		Return
	; cx 关闭应用
	!c:: SendEvent {Blind}{Delete}{Right}
	!x:: SendEvent {Blind}{Delete}{Right}
	
	; 新建桌面
	!z::
		SendEvent {Blind}{Esc}
		Sleep 200
		Send ^#d
		Return
	; 新建桌面并移动窗口
	!v::
		SendEvent {Blind}{Esc}
		Sleep 200
		MoveActiveWindowTo(0)
		Return
	
    ; 模拟 Tab 键切换焦点
	\:: Send {Tab}
    ; 在 Win10 下的 Win+Tab 界面，WASD 切换窗口焦点
	; 以及在窗口贴边后，WASD 切换窗口焦点

    ; 模拟方向键
    w:: Send {Up}
    a:: Send {Left}
    s:: Send {Down}
    d:: Send {Right}

	; 切换桌面概览
	q:: Send ^#{Left}
	e:: Send ^#{Right}
	[:: Send ^#{Left}
	]:: Send ^#{Right}

	; 增删桌面
	=:: Send ^#d
	-:: Send ^#{F4}
	z:: Send ^#{F4}

	; 关掉窗口
	x:: Send ^w{Right}
	`;:: Send ^w{Right}

	; 切换到第x个桌面
	1::Send {AppsKey}m{Down 0}{Enter}
	2::Send {AppsKey}m{Down 1}{Enter}
	3::Send {AppsKey}m{Down 2}{Enter}
	4::Send {AppsKey}m{Down 3}{Enter}
	5::Send {AppsKey}m{Down 4}{Enter}
	6::Send {AppsKey}m{Down 5}{Enter}
	7::Send {AppsKey}m{Down 6}{Enter}
	8::Send {AppsKey}m{Down 7}{Enter}
	9::Send {AppsKey}m{Down 8}{Enter}

	; ; 移到除了自己的最后一个桌面（或新建桌面）
	; 0::Send {AppsKey}m{Up 2}{Enter}

	; 移到新建桌面
	v:: Send {AppsKey}mn{Sleep 16}+{Tab}
	':: Send {AppsKey}mn{Sleep 16}+{Tab}

	; 移到新建桌面，并激活窗口
	c:: Send {AppsKey}mn{Enter}

; 新版

; ahk_class Windows.UI.Core.CoreWindowi
; ahk_exe explorer.exe

; #IfWinActive (?:Task View)|任务视图 ahk_class Windows.UI.Core.CoreWindow ; ahk_exe explorer.exe
#IfWinActive ahk_class Windows.UI.Core.CoreWindow ahk_exe explorer.exe
    ; 在 Alt+Tab 下, WASD 模拟方向键
    !a:: Left
    !d:: Right
    !w:: Up
    !s:: Down
 ;    ; qe 切换桌面
	; !q:: Send ^#{Left}
	; !e:: Send ^#{Right}
	; ; qe 切换桌面
	; !c:: Delete

    ; 模拟 Tab 键切换焦点
	\:: Send {Tab}
    ; 在 Win10 下的 Win+Tab 界面，WASD 切换窗口焦点
	; 以及在窗口贴边后，WASD 切换窗口焦点

    ; 模拟方向键
    w:: Send {Up}
    a:: Send {Left}
    s:: Send {Down}
    d:: Send {Right}

	; 切换桌面概览
	q:: Send {Enter}; ^#{Left}
	e:: Send {Enter}; ^#{Right}
	[:: Send ^#{Left}
	]:: Send ^#{Right}

	; 增删桌面
	=:: Send ^#d
	-:: Send ^#{F4}
	z:: Send ^#{F4}

	; 关掉窗口
	x:: Send ^w
	`;:: Send ^w

	; 切换到第x个桌面
	1::Send {AppsKey}m{Down 0}{Enter}
	2::Send {AppsKey}m{Down 1}{Enter}
	3::Send {AppsKey}m{Down 2}{Enter}
	4::Send {AppsKey}m{Down 3}{Enter}
	5::Send {AppsKey}m{Down 4}{Enter}
	6::Send {AppsKey}m{Down 5}{Enter}
	7::Send {AppsKey}m{Down 6}{Enter}
	8::Send {AppsKey}m{Down 7}{Enter}
	9::Send {AppsKey}m{Down 8}{Enter}

	; ; 移到除了自己的最后一个桌面（或新建桌面）
	; 0::Send {AppsKey}m{Up 2}{Enter}

	; 移到新建桌面
	v:: Send {AppsKey}mn{Sleep 16}+{Tab}
	':: Send {AppsKey}mn{Sleep 16}+{Tab}

	; 移到新建桌面，并激活窗口
	c:: Send {AppsKey}mn{Enter}
