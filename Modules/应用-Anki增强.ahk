;SetTitleMatchMode RegEx
; FileEncoding, UTF-8
; save with utf8 with bom

;^!F12:: ExitApp

Global Anki增强_Lock := 0
#WinActivateForce
Return

AnkiEnlock(key, to){
    If (Anki增强_Lock){
        Send {%key% up}r
        Return
    }
    Anki增强_Lock := 1
    Send %to%
    KeyWait, %key%, T60 ; wait for 60 seconds
}
AnkiUnlock(x)
{
    Anki增强_Lock := 0
    Send %x%
}

;#UseHook On

; ANKI 2.0 and 2.1
#if CapsLockXMode && ( WinActive("Anki -.* ahk_class QWidget ahk_exe anki.exe") || WinActive("Anki - .*|.* - Anki ahk_class Qt5QWindowIcon ahk_exe anki.exe"))

; DEPRECATED -- USE .md please
/:: CapsLockX_ShowHelp("
(
# Anki 增强
| 模式 | Anki 增强模块 | 说明 |
| -------------------- | :-----------: | ----------------------------------------------------------- |
| 在 Anki-学习界面 | w 或 k 或 ↑ | 按下=撤销，松开显示答案 |
| 在 Anki-学习界面 | a 或 h 或 ← | 按下=顺利，松开显示答案 |
| 在 Anki-学习界面 | s 或 j 或 ↓ | 按下=困难，松开显示答案 |
| 在 Anki-学习界面 | d 或 l 或 → | 按下=生疏，松开显示答案 |
| 在 Anki-学习界面 | q | 返回上个界面 |
| 在 Anki-学习界面 | c | 添加新卡片 |
| 在 Anki-学习界面 | 1 或 NumPad1 | 困难（原键位不动） |
| 在 Anki-学习界面 | 2 或 NumPad2 | 生疏（原键位不动） |
| 在 Anki-学习界面 | 3 或 NumPad3 | 一般（原键位不动） |
| 在 Anki-学习界面 | 4 或 NumPad4 | 顺利（原键位不动） |
| 在 Anki-学习界面 | 5 或 NumPad5 | 撤销 |
| 在 Anki-学习界面 | 6 或 NumPad6 | 暂停卡片 |
| 在 Anki-学习界面 | Alt + i | 快速导入剪贴版的内容（按 Tab 分割） / 比如可以从 Excel 复制 |
| 在 Anki-添加卡片界面 | Alt + s | 按下 添加 按钮 |
)")

#If 在Anki学习界面()

在Anki学习界面(){
    return !CapsLockXMode && (WinActive("Anki -.* ahk_class QWidget ahk_exe anki.exe") or WinActive("Anki - .*|.* - Anki ahk_class Qt5QWindowIcon ahk_exe anki.exe"))
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
$s:: AnkiEnlock("s", "2")
$d:: AnkiEnlock("d", "1")
$w up:: AnkiUnlock("{space}")
$a up:: AnkiUnlock("{space}")
$s up:: AnkiUnlock("{space}")
$d up:: AnkiUnlock("{space}")

; 方向键控制
$k:: AnkiEnlock("k", "^z")
$h:: AnkiEnlock("h", "432")
$j:: AnkiEnlock("j", "2")
$l:: AnkiEnlock("l", "1")
$k up:: AnkiUnlock("{space}")
$h up:: AnkiUnlock("{space}")
$j up:: AnkiUnlock("{space}")
$l up:: AnkiUnlock("{space}")

; 方向键控制
$Up:: AnkiEnlock("Up", "^z")
$Left:: AnkiEnlock("Left", "432")
$Down:: AnkiEnlock("Down", "2")
$Right:: AnkiEnlock("Right", "1")
$Up up:: AnkiUnlock("{space}")
$Left up:: AnkiUnlock("{space}")
$Down up:: AnkiUnlock("{space}")
$Right up:: AnkiUnlock("{space}")

; 快速从剪贴板导入卡片列表
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
    FileName = %APPDATA%\Anki2\剪贴板导入.txt
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
        TrayTip, CapsLockX, 没有获取到剪贴板的内容
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
        TrayTip, CapsLockX, 没有获取到剪贴板的内容
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
