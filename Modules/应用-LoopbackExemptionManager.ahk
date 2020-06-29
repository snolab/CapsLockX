
If(!CapsLockX)
	ExitApp
Return
	 

#IfWinActive Loopback Exemption Manager ahk_exe WindowsLoopbackManager.exe
	f1:: Send {Tab}{Space}{Down}
	f2::
		Loop 2 {
			Send {Tab}{Space}{Down}
		}
		Return
	f3:: 
		Loop 4 {
			Send {Tab}{Space}{Down}
		}
		Return
	f4:: 
		Loop 8 {
			Send {Tab}{Space}{Down}
		}
		Return
	f5:: 
		Loop 16 {
			Send {Tab}{Space}{Down}
		}
		Return
	f6:: 
		Loop 32{
			Send {Tab}{Space}{Down}
		}
		Return
	f7:: 
		Loop 64{
			Send {Tab}{Space}{Down}
		}
		Return
	f8:: 
		Loop 128{
			Send {Tab}{Space}{Down}
		}
		Return
