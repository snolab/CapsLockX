If(CapslockX)
    Return

; 显示式
#h::
    Process, Exist, iFlyVoice.exe
    If (ErrorLevel) {
        ; 原方案使用热键触发
        ; Send ^+h
        ; 新方案直接发送模拟消息
        ControlClick, x0 y0, ahk_class UIIFlyVoiceFrame ahk_exe iFlyVoice.exe
    }Else{
        Run "C:\Program Files (x86)\iFly Info Tek\iFlyIME\2.1.1708\iFlyVoice.exe"
    }
Return

