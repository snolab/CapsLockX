; 关于普通正则表达式的例子, 请参阅 正则表达式快速参考.
SetTitleMatchMode RegEx
SetKeyDelay, 0, 0
;#UseHook On


Return

#IfWinActive .*- OneNote ahk_class ApplicationFrameWindow ahk_exe ApplicationFrameHost.exe
	^!F12:: ExitApp ; 退出脚本
	
	; 待定未定
	$!`::SendEvent !r

	; 套锁、空间、橡皮
	$!q::SendEvent !d{Down}{Tab 1}{Enter}
	$!w::SendEvent !d{Down}{Tab 2}{Enter}
	$!e::SendEvent !d{Down}{Tab 3}{Enter}
	
	; 换笔
	$!a::SendEvent !d{Down}{Tab 4}+{Left}{Enter}
	$!s::SendEvent !d{Down}{Tab 4}{Enter}
	$!d::SendEvent !d{Down}{Tab 4}+{Right}{Enter}
	
	; 颜色
	$!1::SendEvent !d{Down}{Right 4}{Enter}
	$!2::SendEvent !d{Down}{Right 5}{Enter}
	$!3::SendEvent !d{Down}{Right 6}{Enter}
	$!4::SendEvent !d{Down}{Right 7}{Enter}
	$!5::SendEvent !d{Down}{Right 8}{Enter}
	$!6::SendEvent !d{Down}{Right 9}{Enter}

	; 画笔粗细
	$!t::SendEvent !d{Down}{Tab 13}{Enter}
	$!g::SendEvent !d{Down}{Tab 11}{Enter}
	
	; 调整缩放
	$![::SendEvent !w{Down}{Tab 3}{Enter}
	$!]::SendEvent !w{Down}{Tab 4}{Enter}
	$!\::SendEvent !w{Down}{Tab 5}{Enter}
	
	; 调整字体
	$^[::SendEvent !h{Down}{Tab 1}{Up   2}{Enter}
	$^]::SendEvent !h{Down}{Tab 1}{Down 2}{Enter}
	$^\::SendEvent !h{Down}+{Tab 1}{Enter}
	
	$!Delete:: SendEvent ^+a{Delete}