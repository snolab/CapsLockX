CoordMode, Mouse, Screen     

; 解决多屏 DPI 问题
DllCall("Shcore.dll\SetProcessDpiAwareness", "UInt", 2)

Return

; 高性能计时
QPF(){
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
    Return QuadPart
}
QPC(){
    DllCall("QueryPerformanceCounter", "Int64*", Counter)
    Return Counter
}

; 时间计算
dt(t, tNow){
    Return t ? (tNow - t) / QPF() : 0
} ; 返回单位: 秒


; ref: https://msdn.microsoft.com/en-us/library/windows/desktop/ms646273(v=vs.85).aspx
SendInput_MouseMsg(dwFlag, mouseData = 0){
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData,  0, "UInt")
    NumPut(0, sendData,  4, "Int") 
    NumPut(0, sendData,  8, "Int") 
    NumPut(mouseData, sendData, 12, "UInt")
    NumPut(dwFlag, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
}

;
global lastTick
~WheelUp::
	tick := QPC()
	seconds := dt(lastTick, tick)
	lastTick := tick
	if (seconds > 0){
		speed := 1 / seconds
		if (speed > 8){
			svy := 120 * (speed / 8)
			ToolTip % svy+120
			SendInput_MouseMsg(0x0800, svy) ; 0x0800/*MOUSEEVENTF_WHEEL*/
			Return
		}
	}
	Return

~WheelDown::
	tick := QPC()
	seconds := dt(lastTick, tick)
	lastTick := tick
	if (seconds > 0){
		speed := 1 / seconds
		if (speed > 8){
			svy := -120 * (speed / 8)
			ToolTip % svy-120
			SendInput_MouseMsg(0x0800, svy) ; 0x0800/*MOUSEEVENTF_WHEEL*/
			Return
		}
	}
	Return