SetTitleMatchMode RegEx
; SetKeyDelay, 0, 0
;debug
;^F12:: ExitApp

;#UseHook On
; .* - OneNote
; ahk_class Framework::CFrame
; ahk_exe ONENOTE.EXE

altSend(altKeys){
	SetKeyDelay, 1, 1 ; 配置纠错
	SendEvent {AltDown}%altKeys%{AltUp}
}

altSendEx(altKeys, suffix){
	SetKeyDelay, 1, 1 ; 配置纠错
	SendEvent {AltDown}%altKeys%{AltUp}%suffix%
}

StrJoin(sep, params*) {
    for index, param in params
        str .= param . sep
    return SubStr(str, 2, -StrLen(sep))
}

Return

GetFocusControlName(){
	ControlGetFocus, name, A
	return name
}

; ClassNN:	RICHEDIT60W1

#If !!(CapsXMode & CM_FN)
	h:: Run "https://support.office.com/zh-cn/article/OneNote-2013-%25E4%25B8%25AD%25E7%259A%2584%25E9%2594%25AE%25E7%259B%2598%25E5%25BF%25AB%25E6%258D%25B7%25E6%2596%25B9%25E5%25BC%258F-65dc79fa-de36-4ca0-9a6e-dfe7f3452ff8?ui=zh-CN&rs=zh-CN&ad=CN&fromAR=1"

#If !CapsX
	^+!F12:: ExitApp ; 退出脚本

; #If CapsXMode and (WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") or WinActive("ahk_class ahk_class OneNote`:`:NavigationUIPopup ahk_exe ONENOTE.EXE"))
#If (WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") or WinActive("ahk_class ahk_class OneNote`:`:NavigationUIPopup ahk_exe ONENOTE.EXE"))
	$^e::
		; 增强搜索
		Send ^e
		
		If ("RICHEDIT60W1" != GetFocusControlName()){
			Return
		}
		Clipboard =
		Send ^c
		ClipWait, 1
		
		If ErrorLevel {
		    Return
		}

		s1 := Clipboard

		a := StrSplit(s1)
		s2 := StrJoin(" ", a*)
		str := """" s1 """" " OR " """" s2 """"

		Clipboard := str
		Send ^v
		Return
	$^f::
		; 增强搜索
		Send ^f
		
		If ("RICHEDIT60W1" != GetFocusControlName()){
			Return
		}
		Clipboard =
		Send ^c
		ClipWait, 1
		
		If ErrorLevel {
		    Return
		}

		s1 := Clipboard

		a := StrSplit(s1)
		s2 := StrJoin(" ", a*)
		str := """" s1 """" " OR " """" s2 """"

		Clipboard := str
		Send ^v
		Return
#IfWinActive .*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE
	
	; ; 选择页面
	; ^PgUp:: Send ^{PgUp}^+a
	; ^PgDn:: Send ^{PgDn}^+a

	; ; 将此页面向上合并
	$!j::
		Send ^+t^a^x
		Send ^{PgUp}^+t^{End}^v
		Send ^{PgDn}^+a{Del}
		Send ^{PgUp}
		Return

	$!c::
		SetKeyDelay, 1, 1 ; 配置纠错
		ToolTip, %A_KeyDelay%
		;Send {AltDown}wi{AltSubmit}
		;Send, {Blind}{AltDown}wi{AltUp}
		; ControlSendRaw, , wi, A
		; ControlSend, , wi, A

		;SendEvent, {AltUp}!w!i
		SendEvent, {AltDown}wi{AltUp}
		Return
	$!x::
		Clipboard := ""
		Send {AppsKey}x
		Send {Tab}^c
		ClipWait 1
		Send {Esc}
		Clipboard := RegExReplace(Clipboard, "([一-龥]) ", "$1")
		Send ^!{Right}^v
		Return
	$!+x::
		Clipboard := ""
		Send {AppsKey}y
		ClipWait 1
		Clipboard := RegExReplace(Clipboard, "([一-龥]) ", "$1")
		Send ^!{Right}^v
		Return

	; 重命名笔记
	$F2:: Send ^+t
	; 重命名分区
	$+F2:: Send ^+g{AppsKey}r

	; 移动笔记
	$!m:: Send ^!m

	; 移动分区
	$!+m:: Send ^+g{AppsKey}m

	; 交换新建笔记热键
	$^n:: Send ^!n
	$^!n:: Send ^n
	
	; 移动页面
	; $!+Up:: Send ^+a!+{Up}
	; $!+Down:: Send ^+a!+{Down}
	; $!+Left:: Send ^+a!+{Left}
	; $!+Right:: Send ^+a!+{Right}

	; 搜索标记
	$!f:: altSend("hg")

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