SetTitleMatchMode RegEx
SetWorkingDir, %APPDATA%\调戏小冰\
FileEncoding, UTF-8

^!F12:: ExitApp
`:: Click
; 用法
; 开始：
; @qq小冰 成语接龙
; 然后 Alt 1 接上
; 如果小冰的回复里没辰[]
; 就手动复制小冰回的成语的最后一字
; 然后 Alt 2
;

#IfWinActive ahk_class TXGuiFoundation ahk_exe TIM.exe
    ^F12:: ExitApp

    conChar(char)
    {
        zuihouyigezi := char
        FileRead, chengyubiao, 1800让人接不下的成语表.txt
        regex := "mO)^(" . zuihouyigezi . ".*)"
        FoundPos2 := RegExMatch(chengyubiao, regex, Match2)
        If FoundPos2
        {
            SendInput, % Match2[0] . "，哈哈接不下去了吧"
            Return
        }

        FileRead, chengyubiao, 13000成语表.txt
        regex := "mO)^(" . zuihouyigezi . ".*)"
        FoundPos2 := RegExMatch(chengyubiao, regex, Match2)
        If FoundPos2
        {
            SendInput, % Match2[0]
            Return
        }
        
        SendInput, 提示
        Return
    }

    !2::
        Send, @{Sleep, 32}2854196{Sleep, 32}{Enter}
        Sleep, 64
        ; 获取剪贴板内容
        ClipWait, 0, text

        if ErrorLevel
        {
            MsgBox, 剪贴板里没有内容
            Return
        }
        
        zuihouyigezi := Clipboard
        conChar(zuihouyigezi)
        Return

    !1::
        Clipboard := ""
        Send, @{Sleep, 32}2{Sleep, 32}8{Sleep, 32}5{Sleep, 32}4{Sleep, 32}1{Sleep, 32}9{Sleep, 32}6{Sleep, 32}{Enter}
        Sleep, 64
        Click 0, -200, Rel
        Send, ^a^c
        Click 0, 200, Rel
        ; 获取剪贴板内容
        ClipWait, 0, text

        if ErrorLevel
        {
            MsgBox, 剪贴板里没有内容
            Return
        }
        str := Clipboard
        regex := "O)[\s\S]*「...(.)」"
        FoundPos := RegExMatch(str, regex, Match)
        If FoundPos
        {
            ;MsgBox, % "FoundPos: " FoundPos "`n" "Match: " Match[1]
            zuihouyigezi := Match[1]
            conChar(zuihouyigezi)
        }
        Return