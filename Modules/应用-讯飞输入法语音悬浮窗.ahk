; @CapsLockX    v1
; @name         Win + H 快速启动讯飞语音悬浮窗
; @description  如题
; @author       snomiao@gmail.com
; @version      2.1.1(20200606)

AppendHelp("
(
讯飞语音悬浮窗
| Win + H | 启动 / 切换讯飞语音输入 |
)")

Return

#IF (!(CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN))

#h::
    If (WinExist("ahk_class UIIFlyVoiceFrame ahk_exe iFlyVoice.exe")) {
        ; 原方案使用热键触发
        ; Send ^+h
        ; 新方案直接发送模拟点击消息
        ControlClick, x0 y0, ahk_class UIIFlyVoiceFrame ahk_exe iFlyVoice.exe
    }Else{
        If (FileExist("C:\Program Files (x86)\iFly Info Tek\iFlyIME\2.1.1708\iFlyVoice.exe")){
            Run "C:\Program Files (x86)\iFly Info Tek\iFlyIME\2.1.1708\iFlyVoice.exe"
        }else{
            MsgBox, 4, , 你似乎还没有安装讯飞语音输入法，是否现在下载安装包并【手动安装】到默认目录？
            IfMsgBox, NO, Return
            UrlDownloadToFile https://download.voicecloud.cn/200ime/iFlyIME_Setup_2.1.1708.exe, %TEMP%/iFlyIME_Setup_2.1.1708.exe
            Run %TEMP%/iFlyIME_Setup_2.1.1708.exe
        }
    }
Return

#+h:: Send #h