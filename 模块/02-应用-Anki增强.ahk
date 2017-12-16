;SetTitleMatchMode RegEx
;FileEncoding, UTF-8

;^!F12:: ExitApp

Global _Lock := 0
AnkiEnlock(key, to){
    If _Lock{
        Send {%key% up}
        Return
    }
    _Lock := 1
    Send %to%
}
AnkiUnlock(x){
    _Lock := 0
    Send %x%
}

Return

;#UseHook On
#IfWinActive Anki -.* ahk_exe anki.exe ahk_class QWidget
    !^F12:: ExitApp

    $x:: Send s ;study
    $q:: Send d ;quit
    $c:: Send a ;create
     
    ; 撤销
    $5:: Send ^z
    $Numpad5:: Send ^z
    
    ; 暂停卡片
    $6:: Send @
    $Numpad6:: Send @

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
    
    ; 手柄控制
    $Browser_Refresh::      AnkiEnlock("Browser_Refresh"  ,"^z")
    $Browser_Favorites::    AnkiEnlock("Browser_Favorites","432")
    $Browser_Home::         AnkiEnlock("Browser_Home"     ,"2")
    $Browser_Stop::         AnkiEnlock("Browser_Stop"     ,"1")
    $Browser_Refresh up::   AnkiUnlock("{space}")
    $Browser_Favorites up:: AnkiUnlock("{space}")
    $Browser_Home up::      AnkiUnlock("{space}")
    $Browser_Stop up::      AnkiUnlock("{space}")
    ;$Media_Play_Pause::  AnkiEnlock("Media_Play_Pause" ,"@")
    $Browser_Back::      AnkiEnlock("Media_Play_Pause" ,"@")
    
    ; 方向键控制

    ; 方向键控制

    ;$Media_Play_Pause up::  AnkiUnlock("{space}")
    $Browser_Back up::      AnkiUnlock("{space}")

    ; 扩充功能
    ; 搜索
    $g::
        Click 2
        Send ^c
        ClipWait, 0, text
        If ErrorLevel
            Return
        Run, https://www.google.com/#q=%Clipboard%
        Return

    ; 快速从剪贴板导入
    $!i::
        ; 获取剪贴板内容
        ClipWait, 0, text
        If ErrorLevel {
            MsgBox, 剪贴板里没有内容
            Return
        }

        ToolTip, %text%

        ; 让 Anki 打开导入框
        Send ^i
        
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
        WinWait, 导入 ahk_class QWidget ahk_exe anki.exe, , 3
        Send ^v

        ; 打开
        Send {Enter}

        Sleep 1000
        ToolTip

        Return