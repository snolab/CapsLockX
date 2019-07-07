; 正确运行方式……
; "C:\Program Files\AutoHotkey\AutoHotkeyU32.exe"  "C:\Users\snomiao\OneDrive\GitHub\CapslockX\工具\插件-自动滚动.ahk"

global autoScrollY, autoScrollStep
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

autoScrollY_SetTimer(){
    If(autoScrollY){
        autoScrollStepAbs := Max(1, -12 + 4 * Abs(autoScrollY))
        autoScrollInterval := Max(1, 96 - 32 * Abs(autoScrollY))
        
        autoScrollStep := autoScrollY > 0 ? autoScrollStepAbs : -autoScrollStepAbs

        ToolTip, s %autoScrollY% %autoScrollInterval% %autoScrollStep%
        SetTimer, autoScroll, %autoScrollInterval%
    }else{
        ToolTip,
        SetTimer, autoScroll, Off
    }
}
Return

autoScroll:
    If(!GetKeyState("Ctrl", "P")){
        APISendInput_ScrollY(autoScrollStep)
    }
    Return

; autoPage:
;     If(!GetKeyState("Ctrl", "P")){
;         ToolTip, s %autoPageY%
;         If(autoPageY > 0){
;             Loop, %autoPageY% {
;                 Send {PgDn}
;             }
;         }
;         If(autoPageY < 0){
;             r := -autoPageY
;             Loop, %r% {
;                 Send {PgUp}
;             }
;         }
;     }
;     Return

; capsx和fn模式都能触发
#If CapsXMode == CM_CAPSX || CapsXMode == CM_FN
    $PgUp::
        autoScrollY := autoScrollY ? autoScrollY : 0
        autoScrollY -= 1
        autoScrollY_SetTimer()

        Return
    $PgDn::
        autoScrollY := autoScrollY ? autoScrollY : 0
        autoScrollY += 1
        autoScrollY_SetTimer()
        Return

    ; $PgUp::
    ;     autoPageY := autoPageY ? autoPageY : 0
    ;     autoPageY += 1
    ;     ToolTip, s %autoPageY%
    ;     SetTimer, autoPage, 1000
    ;     Return

    ; $PgDn::
    ;     autoPageY := autoPageY ? autoPageY : 0
    ;     autoPageY -= 1
    ;     ToolTip, s %autoPageY%
    ;     SetTimer, autoPage, 1000
    ;     Return

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