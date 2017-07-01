; 关于普通正则表达式的例子, 请参阅 正则表达式快速参考.
SetTitleMatchMode RegEx
SetKeyDelay, 0, 0
;#UseHook On

Return

#IfWinActive .*- OneNote ahk_class ApplicationFrameWindow
	^!F12:: ExitApp ; 退出脚本
	
	; 待定未定
	$!`::Send !r

	; 套锁、空间、橡皮
	$!q::Send !d{Down}{Tab 0}{Enter}
	$!w::Send !d{Down}{Tab 1}{Enter}
	$!e::Send !d{Down}{Tab 2}{Enter}
	
	; 换笔
	$!a::Send !d{Down}{Tab 3}{Enter}
	$!s::Send !d{Down}{Tab 4}{Enter}
	$!d::Send !d{Down}{Tab 5}{Enter}
	
	; 颜色
	$!1::Send !d{Down}{Tab 6}{Enter}
	$!2::Send !d{Down}{Tab 7}{Enter}
	$!3::Send !d{Down}{Tab 8}{Enter}
	$!4::Send !d{Down}{Tab 9}{Enter}
	$!5::Send !d{Down}{Tab 10}{Enter}

	; 画笔粗细
	$!t::Send !d{Down}{Tab 13}{Enter}
	$!g::Send !d{Down}{Tab 11}{Enter}
	
	; 调整缩放
	$![::Send !w{Down}{Tab 3}{Enter}
	$!]::Send !w{Down}{Tab 4}{Enter}
	$!\::Send !w{Down}{Tab 5}{Enter}
	
	; 调整字体
	$^[::Send !h{Down}{Tab 1}{Up   2}{Enter}
	$^]::Send !h{Down}{Tab 1}{Down 2}{Enter}
	$^\::Send !h{Down}+{Tab 1}{Enter}