; ========== CapsLockX ==========
; 名称：Anki 增强
; 描述：rt
; 作者：snomiao
; 联系：snomiao@gmail.com
; 版权：Copyright 2020-2022 snomiao@gmail.com
; 版本：2.0.0
; 日期：2022-07-17
; ========== CapsLockX ==========

;SetTitleMatchMode RegEx
; FileEncoding, UTF-8
; save with utf8 with bom

;^!F12:: ExitApp

Global AnkiEnhance_Lock := 0

#WinActivateForce
Return

AnkiEnlock(key, to)
{
    if (AnkiEnhance_Lock) {
        Send {%key% up}r
        Return
    }
    AnkiEnhance_Lock := 1
    Send %to%
    KeyWait, %key%, T60 ; wait for 60 seconds
}
AnkiUnlock(x)
{
    AnkiEnhance_Lock := 0
    Send %x%
}

;#UseHook On

; ANKI 2.0 and 2.1
#if CapsLockXMode && AnkiWindowActiveQ()

AnkiWindowActiveQ()
{
    if(WinActive("Anki -.* ahk_class QWidget ahk_exe anki.exe")) {
        return 1
    }
    if( WinActive("Anki - .*|.* - Anki ahk_class Qt5QWindowIcon ahk_exe anki.exe")) {
        return 1
    }
    if( WinActive("Anki - .*|.* - Anki ahk_class Qt6.*QWindowIcon ahk_exe anki.exe")) {
        return 1
    }
    return 0
}

#if 在Anki学习界面()

在Anki学习界面(){
    return !CapsLockXMode && AnkiWindowActiveQ()
}
$x:: Send s ; study
$q:: Send d ; quit
$c:: Send a ; create

; 撤销
$5:: Send ^z
$Numpad5:: Send ^z

; 暂停卡片
$`:: Send {Space}
$6:: Send @
$Numpad6:: Send @

; 方向键控制
$w:: AnkiEnlock("w", "^z")
$a:: AnkiEnlock("a", "432")
$s:: AnkiEnlock("s", "32")
$d:: AnkiEnlock("d", "1")
$w up:: AnkiUnlock("{space}")
$a up:: AnkiUnlock("{space}")
$s up:: AnkiUnlock("{space}")
$d up:: AnkiUnlock("{space}")

; 方向键控制
$k:: AnkiEnlock("k", "^z")
$h:: AnkiEnlock("h", "432")
$j:: AnkiEnlock("j", "32")
$l:: AnkiEnlock("l", "1")
$k up:: AnkiUnlock("{space}")
$h up:: AnkiUnlock("{space}")
$j up:: AnkiUnlock("{space}")
$l up:: AnkiUnlock("{space}")

; 方向键控制
$Up:: AnkiEnlock("Up", "^z")
$Left:: AnkiEnlock("Left", "432")
$Down:: AnkiEnlock("Down", "32")
$Right:: AnkiEnlock("Right", "1")
$Up up:: AnkiUnlock("{space}")
$Left up:: AnkiUnlock("{space}")
$Down up:: AnkiUnlock("{space}")
$Right up:: AnkiUnlock("{space}")

; 快速从ClipboardImported卡片列表
$!i:: AnkiImport()
AnkiImport()
{
    ; 获取剪贴板内容
    ClipWait, 0, text
    if ErrorLevel {
        MsgBox, 剪贴板里没有内容
        Return
    }
    TrayTip, Anki导入, 获取到 %text%

    ; 让 Anki 打开导入框
    Send ^+i

    ; 获取到文本后保存到临时文件……
    FileName = %APPDATA%\Anki2\ClipboardImported.txt
    file := FileOpen(FileName, "w", "UTF-8")
    if !IsObject(file) {
        MsgBox Can't open "%FileName%" for writing.
        Return
    }
    file.Write(Clipboard)
    file.Close()

    ; 把临时文件路径粘贴到 Anki 文件框
    Clipboard = %FileName%
    WinWait, 导入 ahk_class Qt5QWindowIcon ahk_exe anki.exe, , 3
    Send ^v

    ; 打开
    Send {Enter}

    Sleep 1000
    ToolTip

    Return
}

AnkiAddWindowActiveQ()
{
    return WinActive("添加|Add ahk_exe anki.exe ahk_class QWidget") || WinActive("添加|Add ahk_exe anki.exe ahk_class Qt5QWindowIcon")
}

CaptureScreenNoteAdd()
{
    a := WinExist("添加|Add ahk_class QWidget ahk_exe anki.exe")
    b := WinExist("添加|Add ahk_class Qt5QWindowIcon ahk_exe anki.exe")
    addWindow := a ? a : b
    WinActivate ahk_id %addWindow%
    WinHide ahk_id %addWindow%
    Clipboard := ""
    Send #+s
    ClipWait, 10, 1
    WinShow ahk_id %addWindow%
    if ErrorLevel {
        TrayTip, CapsLockX, % t("没有获取到剪贴板的内容")
        Return False
    }
    while !WinActive("ahk_id" addWindow) && WinExist("ahk_id" addWindow)
    WinActivate ahk_id %addWindow%
    Return True
}
#if AnkiAddWindowActiveQ()

$!c::
    ; 快速添加内容
    WinActive("A")
    WinHide
    Clipboard := ""
    ; Send ^!a
    Send #+s
    Sleep, 128
    WinShow
    ClipWait, 10, 1
    if ErrorLevel
    {
        TrayTip, CapsLockX, % t("没有获取到剪贴板的内容")
    Return
}
Send ^v
Return

$!s:: Send ^{Enter}
$!x:: Send ^+x

#if (WinExist("添加|Add ahk_class QWidget ahk_exe anki.exe") or WinExist("添加|Add ahk_class Qt5QWindowIcon ahk_exe anki.exe")) ;

$F1::
    if (!CaptureScreenNoteAdd()) {
    Return
}
Send ^v
Sleep 200
Send {Tab}
Return
$F2::
    if (!CaptureScreenNoteAdd()) {
    Return
}
Send ^v
Sleep 200
Send ^{Enter}
$F3::
    if (!CaptureScreenNoteAdd()) {
    Return
}
Send ^+o
Return
