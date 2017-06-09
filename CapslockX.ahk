CoordMode, Mouse, Screen
;#Warn All, Off
^!F12:: ExitApp


CapsLock::
	; 这里改注册表是为了禁用 Win + L，不过只有用管理员运行才管用。。。
	If(GetKeyState("ScrollLock", "T"))
		RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 0
	Else
		RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 1
	Send {ScrollLock}
	Return

!CapsLock:: CapsLock



#If GetKeyState("ScrollLock", "T")
	
	Pause::
		RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 1
		DllCall("LockWorkStation")
		Sleep, 1000
		RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 0
		Return
	~#Tab:: Send {ScrollLock}
	
	`:: Enter

	h:: Left
	l:: Right
	j:: Down
	k:: Up
	b:: Send {Delete}

	mva()
	{
		Global mvx, mvy, max, may
		max = 0
		may = 0
		If GetKeyState("a", "P")
			max -= 1
		If GetKeyState("d", "P")
			max += 1
		If GetKeyState("w", "P")
			may -= 1
		If GetKeyState("s", "P")
			may += 1

		max := max*4, may := may*4
		SetTimer, mm, 1
	}

	MoCaLi(v) ; 摩擦力
	{
		v *= 0.8
		If(v > 0)
			v -= 1
		If(v < 0)
			v += 1
		v //= 1
		Return v
	}

	mm:
		Global mvx, mvy, max, may
		mvx += max
		mvy += may

		; 摩擦力
		If((max >= 0 And mvx < 0) Or (max <= 0 And mvx > 0))
			mvx := MoCaLi(mvx)
		If((may >= 0 And mvy < 0) Or (may <= 0 And mvy > 0))
			mvy := MoCaLi(mvy)
		MouseMove, mvx, mvy , 0, R

		If(0 == mvx And 0 == mvy)
			SetTimer, mm, Off
		Return

	w:: mva()
	w Up:: mva()
	a:: mva()
	a Up:: mva()
	s:: mva()
	s Up:: mva()
	d:: mva()
	d Up:: mva()
	e:: LButton
	q:: RButton


	mvScroll()
	{
		Global svx, svy, sax, say
		say = 0
		sax = 0
		If GetKeyState("r", "P")
			say -= 1
		If GetKeyState("f", "P")
			say += 1
		If GetKeyState("z", "P")
			sax -= 1
		If GetKeyState("c", "P")
			sax += 1
		sax *= 5, say *= 5
		SetTimer, mvs, 0
	}

	mouseWheel_lParam(x, y)
	{
	  return x | (y << 16)
	}
	mvs:
		Global svx, svy, sax, say

		svx += sax
		svy += say

		; 摩擦力
		If((sax >= 0 And svx < 0) Or (sax <= 0 And svx > 0))
			svx := MoCaLi(svx)
		If((say >= 0 And svy < 0) Or (say <= 0 And svy > 0))
			svy := MoCaLi(svy)

		;MouseMove, svx, svy, 0, R
		
		MouseGetPos, mouseX, mouseY, id, fcontrol
		wParam := -svy << 16 ;zDelta
		lParam := mouseWheel_lParam(mouseX, mouseY)
		PostMessage, 0x20A, %wParam%, %lParam%, %fcontrol%, ahk_id %id%
		
		wParam := svx << 16 ;zDelta
		lParam := mouseWheel_lParam(mouseX, mouseY)
		PostMessage, 0x20E, %wParam%, %lParam%, %fcontrol%, ahk_id %id%


		If(0 == svx And 0 == svy)
			SetTimer, mvs, Off
		Return

	search(q)
	{
		Run, https://www.google.com/search?q=%q%
	}
	copySelected()
	{
		Send ^c
		ClipWait
		Return Clipboard
	}
	g:: search(copySelected())
	r:: mvScroll()
	^r:: mvScroll()
	r Up:: mvScroll()
	^r Up:: mvScroll()
	f:: mvScroll()
	^f:: mvScroll()
	f Up:: mvScroll()
	^f Up:: mvScroll()
	z:: mvScroll()
	z Up:: mvScroll()
	c:: mvScroll()
	c Up:: mvScroll()

	; 窗口
	x:: Send ^w
	^x:: Send !{F4}
	; 撤销
	u:: Send ^z
	; 重做
	+u:: Send ^z

	1:: Send #1
	2:: Send #2
	3:: Send #3
	4:: Send #4
	5:: Send #5
	6:: Send #6
	7:: Send #7
	8:: Send #8
	9:: Send #9

	F5:: Send {Media_Play_Pause}
	F6:: Send {Media_Prev}
	F7:: Send {Media_Next}
	F8:: Send {Media_Stop}

	
	F10:: Send {Volume_Mute}
	F11:: Send {Volume_Down}
	F12:: Send {Volume_Up}
