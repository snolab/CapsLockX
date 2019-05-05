; 正确运行方式……
; "C:\Program Files\AutoHotkey\AutoHotkeyU32.exe"  "C:\Users\snomiao\OneDrive\GitHub\CapslockX\工具\插件-自动滚动.ahk"

global autoScrollY
autoScrollY := 0

APISendInput(dx, dy, dwFlags, mouseData){
    p := 0
    DWORD := 4
    LONG := A_PtrSize
    ULONG_PTR := A_PtrSize

    cbSize := DWORD * 4 + LONG * 2 + ULONG_PTR * 1
    
    VarSetCapacity(sendData, cbSize, 0)
    NumPut(0, sendData, p, "Int")
    p += DWORD ; type
    NumPut(dx, sendData, p, "Ptr")
    p += LONG ; dx
    NumPut(dy, sendData, p, "Ptr")
    p += LONG ; dy
    NumPut(mouseData, sendData, p, "Int")
    p += DWORD ; mouseData
    NumPut(dwFlags, sendData, p, "Int")
    p += DWORD ; dwFlags
    NumPut(0, sendData, p, "Int")
    p += DWORD ; time
    NumPut(0, sendData, p, "Ptr")
    p += ULONG_PTR ; extra..
    DllCall("SendInput", "Int", 1, "Str", sendData, "Int", cbSize)
}
APISendInput_ScrollY(dy){
    APISendInput(0,0,0x0800, -dy)
}
APISendInput_ScrollX(dx){
    APISendInput(0,0,0x1000, dx)
}

Return

AutoScroll:
    APISendInput_ScrollY(autoScrollY)
    Return

; capsx和fn模式都能触发
#If CapsXMode == CM_CAPSX || CapsXMode == CM_FN
    $^Down::
        autoScrollY := autoScrollY ? autoScrollY : 0
        autoScrollY += 1
        ToolTip, s %autoScrollY%
        SetTimer, AutoScroll, 20
        Return

    $^Up::
        autoScrollY := autoScrollY ? autoScrollY : 0
        autoScrollY -= 1
        ToolTip, s %autoScrollY%
        SetTimer, AutoScroll, 20
        Return

    $Esc::
        If(autoScrollY){
            autoScrollY := 0
            ToolTip,
        }Else{
            autoScrollY := 0
            Send {Esc}
        }
        Return

    $End::
        ExitApp