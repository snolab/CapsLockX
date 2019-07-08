SetTitleMatchMode RegEx
			 			 
Return
#IfWinActive .*- Microsoft Edge ahk_class ApplicationFrameWindow ahk_exe ApplicationFrameHost.exe
	; 对于 UWP 应用，SendEvent 比 SendInput更好用

	; 拿笔（不稳定……）//暂不支持全屏模式下切换笔
	$!w:: SendEvent {Esc}{Esc}{F4}{Sleep 5}{F4}{Sleep 5}{Tab 3}{Space}
	$!q:: SendEvent {F6}{Sleep 10}{Left}{Enter}{F6}
	$!e:: SendEvent {F6}{Sleep 10}{Right}{Enter}{F6}
	
	; 章节跳转
	!,::
		SendEvent !t
		Sleep, 500
		SendEvent {Up}{Enter}{Esc}
		Return
	!.::
		SendEvent !t
		Sleep, 500
		SendEvent {Down}{Enter}{Esc}
		Return
	; 显示目录
	!/:: SendEvent !t
	; 切换自适应页面大小
	!;:: SendEvent ^+a{Esc}
	; 切换双页布局
	!':: SendEvent {F8}{Esc}
	