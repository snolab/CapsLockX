
Global clipLen := 0
FileCreateDir %TEMP%\ClipX
Return
ClipXPush(){
	++clipLen
	FileAppend %ClipboardAll%, %TEMP%\ClipX\%clipLen%.clip ; will overwritten
	tips("你有 " clipLen "块备用剪贴板")
}

ClipXPop(){
	If(clipLen){
		FileRead Clipboard, *c %TEMP%\ClipX\%clipLen%.clip
		FileDelete %TEMP%\ClipX\%clipLen%.clip
		clipLen--
	}
	tips("你有 " clipLen "块备用剪贴板")
}


#If CapsXMode == CM_CAPSX || CapsXMode == CM_FN
	$^z::
		Send ^z
	^z Up:: Return

	$^x::
		ClipXPush()
		Send ^x
	^x Up:: Return

	$^c::
		ClipXPush()
		Send ^c
	^c Up:: Return

	$^v::
		Send ^v
		Sleep 16
		ClipXPop()
	$^v up:: Return

	; $z:: Send {Enter}
	; $z Up:: Return
	; $x:: Send ^w
	; $x Up:: Return
	; $c:: Send ^w
	; $c Up:: Return
	; $v:: Return
	; $v Up:: Return