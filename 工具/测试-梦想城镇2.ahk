; ; # 时间
; 1分钟=60秒
; 1小时=60分钟

; ; # 
; ; 1百吉饼=2小麦+1糖+3鸡蛋+30分钟
; ; 1曲奇饼干=2小麦+2鸡蛋+15分钟
; ; 1面包=2小麦+5分钟


; 养羊场
; 	6羊毛=3小时+6绵羊饲料
; 养鸡场
; 	6鸡蛋=1小时+6鸡食
; 养牛场
; 	6牛奶=20分钟+6奶牛饲料
; 饲料厂
; 	3绵羊饲料=20分钟+2玉米+2萝卜
; 	3鸡食=10分钟+2小麦+1萝卜
; 	3奶牛饲料=5分钟+2小麦+1玉米

; 全速运行的话
; 	3个场留给饲料的时间为
; 	3小时+1小时+20分钟=


; 6牛奶=20分钟+6奶牛饲料;
; 6奶牛饲料


; 3绵羊饲料=5分钟+2玉米+2萝卜;
; 3鸡食=5分钟+2小麦+1萝卜;
; 3奶牛饲料=5分钟+2小麦+1玉米;

; 羊毛
; 鸡蛋
; 牛奶


; ; #
; 1糖=1甘蔗+20分

; # 1. 先识别出所有物品，没有物品就创造物品
; # 2. 等量关系

; 搞个空窗口?

#SingleInstance, Force
; 长时间运行用
Process, Priority, , High

; 回主页：ZXC
; 收获：FG
; 卖东西：VBNm
; 播种：FQ



; fg
; step := 0

; slow:


SetKeyDelay 2048, 128

; p:: GoSub fast
;SetTimer slow, 125000


WinGet win, ID, BlueStacks App Player ahk_class BS2CHINAUI
ControlGet, hwnd, HWND, , , ahk_id %win%
hwnd := DllCall("GetParent","int",hwnd)

GoSub longfull

; SetKeyDelay 1024, 128
; ControlSend, , zx c, ahk_id %hwnd%
; Gosub, fast
Return
stop:
	SetTimer, fast, Off
	SetTimer, slow, Off
	SetTimer, fastfull, Off
	SetTimer, slowfull, Off
	SetTimer, playfull, Off
	SetTimer, longfull, Off
	ToolTip
	Return
slow:
	ToolTip, slow 正在执行
	SetKeyDelay 512, 128

	; 收获：fg
	; 播种：fq
	; 卖东西：vbnm
	; 回主页：zxc
	ControlSend, , zx cfgfqvbnm , ahk_id %hwnd%
	Gosub stop
	SetTimer slow, 117000
	Return
fast:
	ToolTip, fast 正在执行
	SetKeyDelay 512, 128

	; 收获：fg
	; 播种：fq
	; 卖东西：vbnm
	; 回主页：zxc

	ControlSend, , fgfqvbnm , ahk_id %hwnd%
	Gosub stop
	SetTimer fast, 118000
	Return
slowfull:
	ToolTip, slowfull 正在执行
	SetKeyDelay 512, 128

	; 收获：fg
	; 播种：fq
	; 卖东西：vbnm
	; 回主页：zxc
	ControlSend, , zx c fgfqvb{p Down}{Sleep 2}{p Up}nm , ahk_id %hwnd%
	Gosub stop
	SetTimer slowfull, 114000
	Return
fastfull:
	ToolTip, fastfull 正在执行
	SetKeyDelay 512, 128
	; 收获：fg
	; 播种：fqb
	; 卖东西：vbnm
	; 回主页：zxc
	ControlSend, , fgfqvb{p Down}{Sleep 3}{p Up}nm , ahk_id %hwnd%
	Gosub stop
	SetTimer fastfull, 117000
	Return
playfull:
	ToolTip, playfull 正在执行
	SetKeyDelay 500, 100
	
	; rrr{Esc} ; 重置到
	; zx c ; 
	; fgfq ; 小麦收割
	; vb{p Down}{Sleep 2}{p Up}nm 卖掉数量最多的物品

	ControlSend, , rrr{Esc}{b Down}t{b Up}zx c  fgfqvb{p Down}{Sleep 1}{p Up}nm , ahk_id %hwnd%
	Gosub stop
	SetTimer playfull, 110000 ; 120 减去上面这串花的时间
	Return

longfull:
	ToolTip, longfull 正在执行
	
	; rrr{Esc} ; 重置到
	; zx c ; 
	; fgfq ; 小麦收割
	; vb{p Down}{Sleep 2}{p Up}nm 卖掉数量最多的物品
	
	;SetKeyDelay 500, 100
	;ControlSend, , rrr{Esc}{b Down}t{b Up}zx c  fgfqvb{ = Down}{Sleep 3}{p Up}nm , ahk_id %hwnd%
	
	SetKeyDelay 500, 100
	ControlSend, , {Blind}rrr{Esc}{b Down}t{b Up}zx c  fgfqvb, ahk_id %hwnd%
	SetKeyDelay 16, 16
	ControlSend, , {Blind}{p 50}, ahk_id %hwnd%
	SetKeyDelay 500, 100
	ControlSend, , {Blind}nm , ahk_id %hwnd%
	;ControlSend, , rrr{Esc}{b Down}t{b Up}zx c  fgfqvb{p 60}nm , ahk_id %hwnd%
	Gosub stop
	SetTimer longfull, 110000 ; 120 减去上面这串花的时间
	Return
exitgame:
	ControlSend, , rrr{Esc}{Esc}y , ahk_id %hwnd%
	ExitApp
	Return
; Send ZXCV

; FQ
; step++
; SetTimer, slow, 125000

; FG

!`:: Send {p Down}{Sleep 0.2}{p Up}
#IfWinActive BlueStacks App Player ahk_class BS2CHINAUI
	`:: Send {Click}
	F1:: GoSub slow
	F2:: GoSub fast
	F3::
		GoSub stop
		ToolTip, 所有操作己暂停
		Return
	F4:: ControlSend, , zx c, ahk_id %hwnd%
	F5:: GoSub slowfull
	F6:: GoSub fastfull
	F7:: GoSub playfull
	F8:: GoSub longfull
	F12:: GoSub exitgame