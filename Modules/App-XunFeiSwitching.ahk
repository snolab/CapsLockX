; @CapsLockX    v1
; @name         Win + H 快速启动讯飞语音悬浮窗
; @description  如题
; @author       snomiao@gmail.com
; @version      2.1.1(20200606)
;
; 2021-04-15 更新 @telppa：[修改了一下语音识别模块的代码。・Issue #14・snolab/CapsLockX]( https://github.com/snolab/CapsLockX/issues/14 )
;

global T_EnableXunFeiSwitching := CLX_Config("App", "T_EnableXunFeiSwitching", 1, "使用 Win+H 快速启动讯飞语音悬浮窗（默认启用）")
CLX_AppendHelp( CLX_LoadHelpFrom(CLX_THIS_MODULE_HELP_FILE_PATH))

Return

#if !CapsLockXMode && T_EnableXunFeiSwitching

#h:: 讯飞语音输入法切换()

讯飞语音输入法切换(){

    ; v3
    iFlyWnd := WinExist("ahk_class BaseGui ahk_exe iFlyVoice.exe" )
    If (iFlyWnd){
        ; WinGet, Transparent, Transparent
        ; WinSet, TransColor, Off, ahk_id %iFlyWnd%
        ; WinSet, TransColor, 0xffffff 150, ahk_id %iFlyWnd%
        ; WinSet, Transparent, 200, ahk_id %iFlyWnd%
        ; w357 h177
        ControlClick, x175 y80, ahk_id %iFlyWnd%
        return
    }

    ; v2
    iFlyWnd := WinExist("ahk_class UIIFlyVoiceFrame ahk_exe iFlyVoice.exe" )
    If (iFlyWnd){
        ; WinGet, Transparent, Transparent
        ; WinSet, TransColor, Off, ahk_id %iFlyWnd%
        ; WinSet, TransColor, 0xffffff 150, ahk_id %iFlyWnd%
        ; WinSet, Transparent, 200, ahk_id %iFlyWnd%
        ControlClick, x20 y20, ahk_id %iFlyWnd%
        return
    }

    ; 注册表里取路径
    RegRead, iFlyPath, HKLM\SOFTWARE\iFly Info Tek\iFlyIME, Install_Dir_Ver2
    ; 非空检查，确保 iFlyPath 有值
    iFlyPath := (iFlyPath ? iFlyPath : "C:\Program Files (x86)\iFly Info Tek\iFlyIME\2.1.1708")
    If (FileExist(iFlyPath "\iFlyVoice.exe")){
        Run "%iFlyPath%\iFlyVoice.exe"
    }else{
        MsgBox, 4, , 你似乎还没有安装讯飞语音输入法，是否现在下载安装包并【手动安装】到默认目录？ - [讯飞输入法官网 - 更好用的手机输入法，提供专业输入法定制解决方案！]( https://srf.xunfei.cn/#/ )
        IfMsgBox, NO, Return
        ; Run https://download.voicecloud.cn/200ime/iFlyIME_Setup_2.1.1708.exe
        run https://srf.xunfei.cn/#/
        ; - [讯飞输入法官网 - 更好用的手机输入法，提供专业输入法定制解决方案！]( https://srf.xunfei.cn/#/ )
        ; run https://download.voicecloud.cn/200ime/iFlyIME_Setup_3.0.1734.exe
        ; UrlDownloadToFile https://download.voicecloud.cn/200ime/iFlyIME_Setup_2.1.1708.exe, %TEMP%/iFlyIME_Setup_2.1.1708.exe
        ; Run %TEMP%/iFlyIME_Setup_2.1.1708.exe
    }
}

; 加 Alt 访问原热键
#!h:: Send #h
