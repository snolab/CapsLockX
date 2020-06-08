Return

tips(x, t = 1024){
	ToolTip %x%
	SetTimer, RemoveToolTip, %t%
}

RemoveToolTip:
	SetTimer, RemoveToolTip, Off
	ToolTip
	Return
