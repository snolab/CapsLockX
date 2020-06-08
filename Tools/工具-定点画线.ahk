
ptcount := 0
; 添加点
`::
	ptCount := ptCount + 1
	MouseGetPos X%ptCount%, Y%ptCount%
	Return

; 执行
!`::
	MouseMove X1, Y1, 0
	Send {LButton Down}
	Loop %ptCount%{
		MouseMove X%A_Index%, Y%A_Index%, 0
		;MouseMove X%A_Index%, Y%A_Index%, 7
	}
	Send {LButton Up}
	ptCount := 0
	Return
;  ceshi
^`::
	MouseMove X1, Y1, 0
	Send {LButton Down}
	Loop %ptCount%{
		Sleep, 16
		MouseMove X%A_Index%, Y%A_Index%, 0
	}
	Send {LButton Up}
	Return

