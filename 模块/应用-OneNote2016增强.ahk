SetTitleMatchMode RegEx
;SetKeyDelay, 0, 0
;debug
;^F12:: ExitApp

;#UseHook On
; .* - OneNote
; ahk_class Framework::CFrame
; ahk_exe ONENOTE.EXE

altSend(altKeys){
	Send {AltDown}%altKeys%{AltUp}
}

altSendEx(altKeys, suffix){
	Send {AltDown}%altKeys%{AltUp}%suffix%
}

Return

#IfWinActive .*- OneNote ahk_class Framework\:\:CFrame
	^!F12:: ExitApp ; 退出脚本
	
	; ; 选择页面
	; ^PgUp:: Send ^{PgUp}^+a
	; ^PgDn:: Send ^{PgDn}^+a

	; 交换新建笔记热键
	$^n:: Send ^!n
	$^!n:: Send ^n

	; 移动页面
	; $!+Up:: Send ^+a!+{Up}
	; $!+Down:: Send ^+a!+{Down}
	; $!+Left:: Send ^+a!+{Left}
	; $!+Right:: Send ^+a!+{Right}

	; 选择页面
	$^+PgUp:: Send ^+g{Up}{Enter}
	$^+PgDn:: Send ^+g{Down}{Enter}

	; 同步此笔记本
	$^s:: Send +{F9}
	
	; 切换为无色背景
	$!n:: altSend("wpcn")

	; 快速删除本页页面
	;$!Delete:: altSend("pd")
	$!Delete:: Send ^+a{Delete}
	
	; 快速关闭窗口
	$^w:: altSend("{F4}")

	; 输入、套锁、橡皮
	$!q:: altSend("dl")
	$!w:: altSend("dn")
	$!e:: altSend("dek")
	
	; 输入、套锁、橡皮
	$!s:: altSend("dt")
	$!d:: altSend("dh")

	; 视图 - 缩放到页面宽度
	$!r:: altSend("wi")
	$!+r:: altSend("w1")
	
	; 笔收藏夹第一排
	$!1:: altSendEx("dp", "{Home}{Right 0}{Enter}")
	$!2:: altSendEx("dp", "{Home}{Right 1}{Enter}")
	$!3:: altSendEx("dp", "{Home}{Right 2}{Enter}")
	$!4:: altSendEx("dp", "{Home}{Right 3}{Enter}")
	$!5:: altSendEx("dp", "{Home}{Right 4}{Enter}")
	$!6:: altSendEx("dp", "{Home}{Right 5}{Enter}")
	$!7:: altSendEx("dp", "{Home}{Right 6}{Enter}")

	; 收藏夹第二排
	$!+1:: altSendEx("dp", "{Home}{Down 1}{Right 0}{Enter}")
	$!+2:: altSendEx("dp", "{Home}{Down 1}{Right 1}{Enter}")
	$!+3:: altSendEx("dp", "{Home}{Down 1}{Right 2}{Enter}")
	$!+4:: altSendEx("dp", "{Home}{Down 1}{Right 3}{Enter}")
	$!+5:: altSendEx("dp", "{Home}{Down 1}{Right 4}{Enter}")
	$!+6:: altSendEx("dp", "{Home}{Down 1}{Right 5}{Enter}")
	$!+7:: altSendEx("dp", "{Home}{Down 1}{Right 6}{Enter}")

	; 自定义颜色
	$!`:: altSendEx("dp", "{Down 2}{Left}")
	$!+`:: altSend("dc")

	; 画笔粗细
	$!t:: altSendEx("d", "{Down}{Tab 13}{Enter}")
	$!g:: altSendEx("d", "{Down}{Tab 11}{Enter}")
	
	; 调整缩放
	$![:: altSendEx("w", "{Down}{Tab 3}{Enter}")
	$!]:: altSendEx("w", "{Down}{Tab 4}{Enter}")
	$!\:: altSendEx("w", "{Down}{Tab 5}{Enter}")
	
	; 调整字体
	$^[:: altSendEx("h", "{Down}{Tab 1}{Up   2}{Enter}")
	$^]:: altSendEx("h", "{Down}{Tab 1}{Down 2}{Enter}")
	$^\:: altSendEx("h", "{Down}+{Tab 1}{Enter}")