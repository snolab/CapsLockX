CheckStrong(password){
	Return ( RegExMatch(password, "[0-9]") && RegExMatch(password, "[a-z]") && RegExMatch(password, "[A-Z]") )
}
;
GenPassword(Length := 16){
	; 此处 `` 为转义
    Chars := "!""#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_``abcdefghijklmnopqrstuvwxyz{|}~"
    Min := 1
    Max := StrLen(chars)

	pw := ""
    Loop %Length% {
	    Random StartPos, Min, Max
	    pw := pw SubStr(Chars, StartPos, 1)
    }
    Clipboard := Chars
    if (CheckStrong(pw)){
    	Return pw
    }else{
    	Return GenPassword(Length)
    }
}

Length := 16
Clipboard := GenPassword(Length)

TrayTip,, 长度%Length%的密码己复制
