; # 时间
; 1分钟=60秒
; 1小时=60分钟

; # 
; 1百吉饼=2小麦+1糖+3鸡蛋+30分钟
; 1曲奇饼干=2小麦+2鸡蛋+15分钟
; 1面包=2小麦+5分钟

; #
; 1糖=1甘蔗+20分

; # 1. 先识别出所有物品，没有物品就创造物品
; # 2. 等量关系

; 搞个空窗口?

#SingleInstance, Force

dm := ComObjCreate("dm.dmsoft") ;此处:=表示表达式返回值赋值给变量dm
ver := dm.ver() ;调用大漠的插件版本查询函数，只会出现两种情况返回值为空及返回值为版本号。
if (ver){ ;如果版本号存在
	;MsgBox,,,注册成功！,1
}else {
	MsgBox,,,注册失败！`n请检查ahk版本及大漠是否已注册到系统`n请参考大漠接口说明-常见问题-创建对象失败怎么办,4
}

;ControlGet, OutputVar, Cmd, [Value, Control, WinTitle, WinText, ExcludeTitle, ExcludeText

WinGet win, ID, BlueStacks App Player ahk_class BS2CHINAUI
ControlGet, hwnd, HWND, , , ahk_id %win%
hwnd := DllCall("GetParent","int",hwnd)
;ControlGet, hwnd, HWND, , , ahk_id %hwnd%
WinGetTitle, title, ahk_id %hwnd%
WinGetClass, c, ahk_id %hwnd%
;ControlGet, hwnd, HWND, , , BlueStacks App Player ahk_class BS2CHINAUI

;hwnd := dm.FindWindow("BlueStacksApp", "_ctl.Window")

;hwnd := dm.FindWindow("BlueStacksApp", "")



cx := 60
cy := 660

Pos2Long(x, y){
    Return x | (y << 16)
}

wParam := 0x0000 
lParam := Pos2Long(cx, cy)
;SendMessage 0x0201, %wParam%, %lParam%, %hwnd%, ahk_id %win%
Sleep, 50
;SendMessage 0x0202, %wParam%, %lParam%, %hwnd%, ahk_id %win%
SetKeyDelay 1024, 128
ControlSend, , fg, ahk_id %hwnd%
; SendMessage 0x0201, %wParam%, %lParam%, %hwnd%, ahk_id %win%
; SendMessage 0x0202, %wParam%, %lParam%, %hwnd%, ahk_id %win%

;Msgbox %hwnd% / %c%
;ControlClick, x20 y20, ahk_id %hwnd%

;WinHide ahk_id %hwnd%
; WinShow ahk_id %hwnd%

;ControlClick, x20 y20, ahk_id %hwnd%
;dm.BindWindow(hwnd, "normal","dx","normal",0)
;dm.BindWindow(hwnd)
;dm.moveto 20,20 ;这一行报错了，说没有这个函数。
;dm.LeftClick

; WinGet win, ID, BlueStacks App Player
; ControlGet win, List, ahk_class _ctl.Window

; MsgBox %win%
; WIN_PATTERN := "DESKTOP-TALVARO 上的 Win10 - 虚拟机连接 ahk_exe vmconnect.exe"

; WIN_PATTERN := "DESKTOP-TALVARO 上的 Win10 - 虚拟机连接 ahk_exe vmconnect.exe"
; WinGet win, ID, %WIN_PATTERN%


; WIN_PATTERN := "DESKTOP-TALVARO 上的 Win10 - 虚拟机连接 ahk_exe vmconnect.exe"
; WinGet win, ID, %WIN_PATTERN%

; ; ControlGet, OutputVar, Cmd, [Value, Control, WinTitle, WinText, ExcludeTitle, ExcludeText
; MouseGetPos, mx, my, win, fControl

; ;cx := 60
; cx := 60
; ;cy := 720
; cy := 720

; Pos2Long(x, y){
;     Return x | (y << 16)
; }

; wParam := 0x0000
; lParam := Pos2Long(cx, cy)
; SendMessage 0x0201, %wParam%, %lParam%, %fControl%, ahk_id %win%
; Sleep, 50
; SendMessage 0x0202, %wParam%, %lParam%, %fControl%, ahk_id %win%

;WinActivate, ahk_id %win%
;ControlClick, x60 y720, ahk_id %win%

;ControlClick, ahk_id %win%, Left, x60 y720
;Click 60, 720
;1:: MouseClickDrag, L, 200, 200, 300, 300, 5
F12:: ExitApp

; 取位置(){; "好友列表	64	-64"


	

; }

; 点击(位置){
; }

; 回到初始位置(){
; 	点击("好友列表")
; 	点击("家")
; }