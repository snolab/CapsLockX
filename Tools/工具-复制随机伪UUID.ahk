
GenRandomHex(Length){
	; 此处 `` 为转义
    Chars := "0123456789abcdef"
    Min := 1
    Max := StrLen(chars)

	randHex := ""
    Loop %Length% {
	    Random StartPos, Min, Max
	    randHex := randHex SubStr(Chars, StartPos, 1)
    }
    Return randHex
}
GenRandomUUID(){
    Return GenRandomHex(8) "-" GenRandomHex(4) "-" GenRandomHex(4) "-" GenRandomHex(4) "-" GenRandomHex(12)
}
Clipboard := GenRandomUUID()
Length := StrLen(Clipboard)

TrayTip,, 长度%Length%的UUID己复制
