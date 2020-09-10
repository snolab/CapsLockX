
;
GenPassword(Length){
	; 此处 `` 为转义
    Chars := "0123456789"
    Min := 1
    Max := StrLen(chars)

	pw := ""
    Loop %Length% {
	    Random StartPos, Min, Max
	    pw := pw SubStr(Chars, StartPos, 1)
    }
    Clipboard := Chars
    Return pw
}

Length := 16
Clipboard := GenPassword(Length)

TrayTip,, 长度%Length%的密码己复制
