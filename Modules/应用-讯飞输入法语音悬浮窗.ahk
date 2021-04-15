; @CapsLockX    v1
; @name         Win + H 快速启动讯飞语音悬浮窗
; @description  如题
; @author       snomiao@gmail.com
; @version      2.1.1(20200606)
; 
; 2021-04-15 更新 @telppa：[修改了一下语音识别模块的代码。・Issue #14・snolab/CapsLockX]( https://github.com/snolab/CapsLockX/issues/14 )
; 
CapsLockX_AppendHelp("
(
讯飞语音悬浮窗
| Win + H | 启动 / 切换讯飞语音输入 |
)")

Return

#if !CapsLockXMode

#h:: 讯飞语音输入法切换()
讯飞语音输入法切换(){
    ; 注册表里取路径
    RegRead, iFlyPath, HKLM\SOFTWARE\iFly Info Tek\iFlyIME, Install_Dir_Ver2 
    ; 非空检查，确保 iFlyPath 有值
    iFlyPath := (iFlyPath ? iFlyPath : "C:\Program Files (x86)\iFly Info Tek\iFlyIME\2.1.1708")
    iFlyWnd := WinExist("ahk_class UIIFlyVoiceFrame ahk_exe iFlyVoice.exe" )
    If (iFlyWnd){
        ControlClick, x20 y20, ahk_id %iFlyWnd%
    }Else{
        If (FileExist(iFlyPath "\iFlyVoice.exe")){
            Run "%iFlyPath%\iFlyVoice.exe"
        }else{
            MsgBox, 4, , 你似乎还没有安装讯飞语音输入法，是否现在下载安装包并【手动安装】到默认目录？ 
            IfMsgBox, NO, Return
            UrlDownloadToFile https://download.voicecloud.cn/200ime/iFlyIME_Setup_2.1.1708.exe, %TEMP%/iFlyIME_Setup_2.1.1708.exe
            Run %TEMP%/iFlyIME_Setup_2.1.1708.exe
        }
    }
}

; 加 Alt 访问原热键
#!h:: Send #h