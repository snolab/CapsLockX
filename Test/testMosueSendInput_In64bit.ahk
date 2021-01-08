; 不知道这个API为什么在64位的AHK下无效
; 还请高人出手

; INPUTDATA = class {
;     INT type; 
;     union input = {
;         struct  mi = MOUSEINPUT(); 
;     };
; } 
; INPUTDATA.MOUSEINPUT = class {
;     INT dx;
;     INT dy;
;     INT mouseData;
;     INT dwFlags;
;     INT time;//if this parameter is 0, the system will provide its own time stamp
;     INT dwExtraInfo;
; } 

cbSize := 28
x := 200
y := 100

VarSetCapacity(sendData, cbSize, 0) ;为鼠标信息 结构 设置出20字节空间
NumPut(0, sendData,  0, "UInt") ; DWORD INPUT.type
NumPut(x, sendData,  4, "Int") ; INPUT.mouse_event.dx
NumPut(y, sendData,  8, "Int") ; INPUT.mouse_event.dy
NumPut(1, sendData, 16, "UInt") ; INPUT.mouse_event.dwFlags = 1/*_MOUSEEVENTF_MOVE*/
DllCall("User32.dll\SendInput", "UInt", 1, "Ptr", &sendData, "UInt", cbSize)

;MsgBox, % sendData


;    Return, 0


;MsgBox, % re
;MsgBox, % sendData
;DllCall("MessageBox","Uint",0,"Str","This Message is poped through DLLcall","Str","I typed that title","Uint","0x00000036L")