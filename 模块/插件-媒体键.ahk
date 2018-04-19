#MaxHotkeysPerInterval 100
Return

#If !!(CapsXMode & CM_FN) || !!(CapsXMode & CM_CAPSX)

    ; $WheelUp:: 
    ;     Send {Volume_Up}
    ;     CapsX_FnActed := 1
    ;     Return
    ; $WheelDown::
    ;     Send {Volume_Down}
    ;     CapsX_FnActed := 1
    ;     Return
    ; $!WheelUp:: 
    ;     Send {Media_Prev}
    ;     CapsX_FnActed := 1
    ;     Return
    ; $!WheelDown::
    ;     Send {Media_Next}
    ;     CapsX_FnActed := 1
    ;     Return

    F1:: Launch_App1 ; 打开我的电脑
    F2:: Launch_App2 ; 计算器
    F3:: Browser_Home
    F4:: Launch_Media

    F5:: Send {Media_Play_Pause}
    F6:: Send {Media_Prev}
    F7:: Send {Media_Next}
    F8:: Send {Media_Stop}
    
    F9:: Send {Volume_Up}
    F10:: Send {Volume_Down}
    F11:: Send {Volume_Mute}
    F12:: Send {Launch_App2}
    
    Pause:: SendMessage,0x112,0xF170,2,,Program Manager