; 帝国僧侣海

SetMouseDelay 2
SetKeyDelay 2, 2
SetDefaultMouseSpeed 2
SetControlDelay 2


Pos2Long(x, y){
    Return x | (y << 16)
}

#IfWinActive Age of Empires II: HD Edition
	; ahk_class Age of Empires II: HD Edition
	; ahk_exe AoK HD.exe

	; 僧侣海方案1 招降
	; `::
	; 	MouseGetPos mx, my
	; 	sw := A_ScreenWidth, sh := A_ScreenHeight
	; 	MouseMove % 700, sh - 250
	; 	Send ^{Click}^3
	; 	MouseMove % mx, my
	; 	Return
	
	; 僧侣海方案2 招降
	; `::
	; 	MouseGetPos mx, my
	; 	sw := A_ScreenWidth, sh := A_ScreenHeight
	; 	Send +2^2
	; 	Send 3
	; 	MouseMove % 700, sh - 250
	; 	Send {Click}^0
	; 	MouseMove % mx, my
	; 	Return

	; ; 僧侣海方案2 选出伤兵
	; `::
	; 	; MouseGetPos mx, my
	; 	sw := A_ScreenWidth, sh := A_ScreenHeight
	; 	;lParam := Pos2Long(700, sh - 250)
	; 	hx := 716, hy := sh - 223
	; 	cx := hx, cy := hy
	; 	clickList := []

	; 	start:
	; 	PixelGetColor hcolor, hx, hy, RGB
	; 	; tooltip %clickList%
	; 	if (0x00FF00 == hcolor){
	; 		clickList.Push([cx, cy])
	; 		hx += 82
	; 		Goto, start
	; 	}else if (0xFF0000 == hcolor){
	; 		; clickList.Push([cx, cy])
	; 		cx += 82
	; 		hx += 82
	; 		Goto, start
	; 	}
	; 	MouseGetPos, mx, my, win, fControl

	; 	wParam := 0x0008 ; MK_CONTROL
	; 	For Key, Value in clickList{
	; 		lParam := Pos2Long(Value[1], Value[2])
	; 		SendMessage 0x0201, %wParam%, %lParam% ;, %fControl%, ahk_id %win%
	; 		Sleep 64
	; 		SendMessage 0x0202, %wParam%, %lParam% ;, %fControl%, ahk_id %win%
	; 		Sleep 64
	; 	}
	; 	Send ^0
	; 	Return

	; !`::
	; 	Send ^0
	; 	; MouseGetPos mx, my
	; 	sw := A_ScreenWidth, sh := A_ScreenHeight
	; 	;lParam := Pos2Long(700, sh - 250)
	; 	hx := 716, hy := sh - 223
	; 	cx := hx, cy := hy
	; 	clickList := []

	; 	start2:
	; 	PixelGetColor hcolor, hx, hy, RGB
	; 	; tooltip %clickList%
	; 	if (0xFF0000 == hcolor){
	; 		clickList.Push([cx, cy])
	; 		hx += 82
	; 		Goto, start2
	; 	}else if (0x00FF00 == hcolor){
	; 		; clickList.Push([cx, cy])
	; 		cx += 82
	; 		hx += 82
	; 		Goto, start2
	; 	}
	; 	MouseGetPos, mx, my, win, fControl

	; 	wParam := 0x0008 ; MK_CONTROL
	; 	For Key, Value in clickList{
	; 		lParam := Pos2Long(Value[1], Value[2])
	; 		SendMessage 0x0201, %wParam%, %lParam% ;, %fControl%, ahk_id %win%
	; 		Sleep 64
	; 		SendMessage 0x0202, %wParam%, %lParam% ;, %fControl%, ahk_id %win%
	; 		Sleep 64
	; 	}
	; 	Return

	`::
		;Send ^b+s+s+s
		Send ^a+f+f+f
		Return

	; ; ; EVO方案3 杀死伤兵
	; `::
	; 	; MouseGetPos mx, my
	; 	sw := A_ScreenWidth, sh := A_ScreenHeight
	; 	;lParam := Pos2Long(700, sh - 250)
	; 	hx := 716, hy := sh - 223
	; 	cx := hx, cy := hy
	; 	clickList := []

	; 	start:
	; 	PixelGetColor hcolor, hx, hy, RGB
	; 	; tooltip %clickList%
	; 	if (0x00FF00 == hcolor){
	; 		; clickList.Push([cx, cy])
	; 		wParam := 0
	; 		lParam := Pos2Long(Value[1], Value[2])
	; 		SendMessage 0x0201, %wParam%, %lParam% ;, %fControl%, ahk_id %win%
	; 		;;Sleep 64
	; 		SendMessage 0x0202, %wParam%, %lParam% ;, %fControl%, ahk_id %win%

	; 		hx += 82
	; 		Goto, start
	; 	}else if (0xFF0000 == hcolor){
	; 		; clickList.Push([cx, cy])
	; 		cx += 82
	; 		hx += 82
	; 		Goto, start
	; 	}
	; 	MouseGetPos, mx, my, win, fControl

	; 	For Key, Value in clickList{
	; 		wParam := 0x0008 ; MK_CONTROL
	; 		lParam := Pos2Long(Value[1], Value[2])
	; 		SendMessage 0x0201, %wParam%, %lParam% ;, %fControl%, ahk_id %win%
	; 		;;Sleep 64
	; 		SendMessage 0x0202, %wParam%, %lParam% ;, %fControl%, ahk_id %win%
	; 		;Sleep 64
	; 	}
	; 	; Send ^0
	; 	Return


	=::
		Send {Delete}{F3}{F3}0aa{Click}{F3}{F3}{Click}
		Return
Pause:: ExitApp
End:: ExitApp