; @CapsLockX    v1
; @name         Win + H 快速启动讯飞语音悬浮窗
; @description  如题
; @author       snomiao@gmail.com
; @version      2.1.1(20200606)
;
; 2021-04-15 更新 @telppa：[修改了一下语音识别模块的代码。・Issue #14・snolab/CapsLockX]( https://github.com/snolab/CapsLockX/issues/14 )
;
CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))

Return

#if !CapsLockXMode

#!h:: 讯飞语音输入法切换()

讯飞语音输入法切换(){
    ; 注册表里取路径
    RegRead, iFlyPath, HKLM\SOFTWARE\iFly Info Tek\iFlyIME, Install_Dir_Ver2
    ; 非空检查，确保 iFlyPath 有值
    iFlyPath := (iFlyPath ? iFlyPath : "C:\Program Files (x86)\iFly Info Tek\iFlyIME\2.1.1708")
    iFlyWnd := WinExist("ahk_class UIIFlyVoiceFrame ahk_exe iFlyVoice.exe" )
    if (iFlyWnd) {
        ; WinGet, Transparent, Transparent
        ; WinSet, TransColor, Off, ahk_id %iFlyWnd%
        ; WinSet, TransColor, 0xffffff 150, ahk_id %iFlyWnd%
        ; WinSet, Transparent, 200, ahk_id %iFlyWnd%
        ControlClick, x20 y20, ahk_id %iFlyWnd%
    } else {
        if (FileExist(iFlyPath "\iFlyVoice.exe")) {
            Run "%iFlyPath%\iFlyVoice.exe"
        } else {
            MsgBox, 4, , 你似乎还没有安装讯飞语音输入法，是否现在下载安装包并【手动安装】到默认目录？
            IfMsgBox, NO, Return
            UrlDownloadToFile https://download.voicecloud.cn/200ime/iFlyIME_Setup_2.1.1708.exe, %TEMP%/iFlyIME_Setup_2.1.1708.exe
            Run %TEMP%/iFlyIME_Setup_2.1.1708.exe
        }
    }
}

; 加 Alt 访问原热键
; #h:: Send #h
; From Acc.ahk by Sean, jethrow, malcev, FeiYue
GetCaret()
{
    static oleacc
    CoordMode, Caret, Screen
    CaretX:=A_CaretX, CaretY:=A_CaretY
    if (!CaretX && !CaretY) {
        try {
            if (!oleacc)
                oleacc:=DllCall("LoadLibrary", "Str", "oleacc", "Ptr")
            VarSetCapacity(IID, 16)
            idObject:=OBJID_CARET:=0xFFFFFFF8
            NumPut(idObject==0xFFFFFFF0?0x0000000000020400:0x11CF3C3D618736E0, IID, "Int64")
            NumPut(idObject==0xFFFFFFF0?0x46000000000000C0:0x719B3800AA000C81, IID, 8, "Int64")
            aofw := DllCall("oleacc\AccessibleObjectFromWindow", "Ptr", WinExist("A"), "UInt", idObject, "Ptr", &IID, "Ptr*", pacc)
            if (aofw==0) {
                Acc := ComObject(9, pacc, 1), ObjAddRef(pacc)
                , Acc.accLocation(ComObj(0x4003, &x:=0), ComObj(0x4003, &y:=0)
                , ComObj(0x4003, &w:=0), ComObj(0x4003, &h:=0), ChildId:=0)
                , CaretX:=NumGet(x, 0, "int"), CaretY:=NumGet(y, 0, "int")
            }
        }
    }
    return {x: CaretX, y: CaretY}
}