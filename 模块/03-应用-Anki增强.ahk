;SetTitleMatchMode RegEx
; FileEncoding, UTF-8
; save with utf8 with bom

;^!F12:: ExitApp

Global Anki增强_Lock := 0
Return
     
AnkiEnlock(key, to){
    If Anki增强_Lock{
        Send {%key% up}
        Return
    }
    Anki增强_Lock := 1
    Send %to%
}
AnkiUnlock(x){
    Anki增强_Lock := 0
    Send %x%
}

;#UseHook On
    

; ANKI 2.0 and 2.1
#If WinActive("Anki -.* ahk_class QWidget ahk_exe anki.exe") or WinActive("Anki - .*|.* - Anki ahk_class Qt5QWindowIcon ahk_exe anki.exe")
    $x:: SendEvent s ; study
    $q:: SendEvent d ; quit
    $c:: SendEvent a ; create
 
    ; 撤销
    $5:: SendEvent ^z
    $Numpad5:: SendEvent ^z
    
    ; 暂停卡片
    $`:: SendEvent {Space}
    $6:: SendEvent @
    $Numpad6:: SendEvent @

    ; 方向键控制
    $w::                 AnkiEnlock("w"                ,"^z")
    $a::                 AnkiEnlock("a"                ,"432")
    $s::                 AnkiEnlock("s"                ,"2")
    $d::                 AnkiEnlock("d"                ,"1")
    $w up::              AnkiUnlock("{space}")
    $a up::              AnkiUnlock("{space}")
    $s up::              AnkiUnlock("{space}")
    $d up::              AnkiUnlock("{space}")
    
    ; 方向键控制
    $k::                 AnkiEnlock("k"                ,"^z")
    $h::                 AnkiEnlock("h"                ,"432")
    $j::                 AnkiEnlock("j"                ,"2")
    $l::                 AnkiEnlock("l"                ,"1")
    $k up::              AnkiUnlock("{space}")
    $h up::              AnkiUnlock("{space}")
    $j up::              AnkiUnlock("{space}")
    $l up::              AnkiUnlock("{space}")

    ; 方向键控制
    $Up::                AnkiEnlock("Up"               ,"^z")
    $Left::              AnkiEnlock("Left"             ,"432")
    $Down::              AnkiEnlock("Down"             ,"2")
    $Right::             AnkiEnlock("Right"            ,"1")
    $Up up::             AnkiUnlock("{space}")
    $Left up::           AnkiUnlock("{space}")
    $Down up::           AnkiUnlock("{space}")
    $Right up::          AnkiUnlock("{space}")
    
    ; 快速从剪贴板导入卡片列表
    $!i::
        ; 获取剪贴板内容
        ClipWait, 0, text
        If ErrorLevel {
            MsgBox, 剪贴板里没有内容
            Return
        }

        ToolTip, %text%

        ; 让 Anki 打开导入框
        Send ^+i
        
        ; 获取到文本后保存到临时文件……
        FileName = %APPDATA%\Anki2\剪贴板导入.txt
        file := FileOpen(FileName, "w", "UTF-8")
        If !IsObject(file) {
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

#If WinActive("添加|Add ahk_exe anki.exe ahk_class QWidget") or WinActive("添加|Add ahk_exe anki.exe ahk_class Qt5QWindowIcon")
    ; 快速添加内容
    $!c::
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
            ToolTip, 没有获取到剪贴板的内容, 2
            Return
        }
        SendEvent ^v
        Return

    $!s:: SendEvent ^{Enter}
    $!x:: SendEvent ^+x

#If WinExist("添加|Add ahk_class QWidget ahk_exe anki.exe") or WinActive("添加|Add ahk_class Qt5QWindowIcon ahk_exe anki.exe")
    $!1::
        ToolTip 123
        ahkid := LastFoundExist
        WinHide ahk_id %ahkid%
        Clipboard := ""
        Send #+s
        WinWait Screen snipping ahk_class Windows.UI.Core.CoreWindow ahk_exe ShellExperienceHost.exe
        WinShow ahk_id %ahkid%
        WinActivate, ahk_id %ahkid%
        ClipWait, 10, 1
        if ErrorLevel   
        {
            ToolTip, 没有获取到剪贴板的内容, 2
            Return
        }
        SendEvent ^v{Tab}
    $!h::
        ToolTip "; 快速添加内容 $!c::"
        Return