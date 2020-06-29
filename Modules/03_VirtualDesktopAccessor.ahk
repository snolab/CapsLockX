; [Ciantic/VirtualDesktopAccessor: DLL for accessing Windows 10 Virtual Desktop features from e.g. AutoHotkey]( https://github.com/Ciantic/VirtualDesktopAccessor )

; Modules

global hVirtualDesktopAccessor := DllCall("LoadLibrary", Str, "Modules/VirtualDesktopAccessor.dll", "Ptr") 
; global hVirtualDesktopAccessor := DllCall("LoadLibrary", Str, "D:/Users/snomi/CapslockX/Modules/VirtualDesktopAccessor.dll", "Ptr")
global GoToDesktopNumberProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "GoToDesktopNumber", "Ptr")
global GetCurrentDesktopNumberProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "GetCurrentDesktopNumber", "Ptr")
global IsWindowOnCurrentVirtualDesktopProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "IsWindowOnCurrentVirtualDesktop", "Ptr")
global MoveWindowToDesktopNumberProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "MoveWindowToDesktopNumber", "Ptr")
global RegisterPostMessageHookProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "RegisterPostMessageHook", "Ptr")
global UnregisterPostMessageHookProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "UnregisterPostMessageHook", "Ptr")
global IsPinnedWindowProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "IsPinnedWindow", "Ptr")
global RestartVirtualDesktopAccessorProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "RestartVirtualDesktopAccessor", "Ptr")
; GetWindowDesktopNumberProc := DllCall("GetProcAddress", Ptr, hVirtualDesktopAccessor, AStr, "GetWindowDesktopNumber", "Ptr")
activeWindowByDesktop := {}

; ToolTip, % hVirtualDesktopAccessor

; Restart the virtual desktop accessor when Explorer.exe crashes, or restarts (e.g. when coming from fullscreen game)
explorerRestartMsg := DllCall("user32\RegisterWindowMessage", "Str", "TaskbarCreated")
OnMessage(explorerRestartMsg, "OnExplorerRestart")
OnExplorerRestart(wParam, lParam, msg, hwnd) {
    global RestartVirtualDesktopAccessorProc
    DllCall(RestartVirtualDesktopAccessorProc, UInt, result)
}
; GoToDesktopNumber(2)
MoveCurrentWindowToDesktop(number) {
	global MoveWindowToDesktopNumberProc, GoToDesktopNumberProc, activeWindowByDesktop
	WinGet, activeHwnd, ID, A
	activeWindowByDesktop[number] := 0 ; Do not activate
	DllCall(MoveWindowToDesktopNumberProc, UInt, activeHwnd, UInt, number)
	DllCall(GoToDesktopNumberProc, UInt, number)
}

GoToPrevDesktop() {
	global GetCurrentDesktopNumberProc, GoToDesktopNumberProc
	current := DllCall(GetCurrentDesktopNumberProc, UInt)
	if (current = 0) {
		GoToDesktopNumber(7)
	} else {
		GoToDesktopNumber(current - 1)      
	}
	return
}

GoToNextDesktop() {
	global GetCurrentDesktopNumberProc, GoToDesktopNumberProc
	current := DllCall(GetCurrentDesktopNumberProc, UInt)
	if (current = 7) {
		GoToDesktopNumber(0)
	} else {
		GoToDesktopNumber(current + 1)    
	}
	return
}

GoToDesktopNumber(num) {
	global GetCurrentDesktopNumberProc, GoToDesktopNumberProc, IsPinnedWindowProc, activeWindowByDesktop

	; Store the active window of old desktop, if it is not pinned
	WinGet, activeHwnd, ID, A
	current := DllCall(GetCurrentDesktopNumberProc, UInt) 
	isPinned := DllCall(IsPinnedWindowProc, UInt, activeHwnd)
	if (isPinned == 0) {
		activeWindowByDesktop[current] := activeHwnd
	}

	; Try to avoid flashing task bar buttons, deactivate the current window if it is not pinned
	if (isPinned != 1) {
		WinActivate, ahk_class Shell_TrayWnd
	}

	; Change desktop
	DllCall(GoToDesktopNumberProc, Int, num)
	return
}

; Windows 10 desktop changes listener

; DetectHiddenWindows, On
; hwnd:=WinExist("ahk_pid " . DllCall("GetCurrentProcessId","Uint"))
; hwnd+=0x1000<<32
; DllCall(RegisterPostMessageHookProc, Int, hwnd, Int, 0x1400 + 30)
; OnMessage(0x1400 + 30, "VWMess")
; VWMess(wParam, lParam, msg, hwnd) {
; 	; global IsWindowOnCurrentVirtualDesktopProc, IsPinnedWindowProc, activeWindowByDesktop

; 	; desktopNumber := lParam + 1
	
; 	; ; Try to restore active window from memory (if it's still on the desktop and is not pinned)
; 	; WinGet, activeHwnd, ID, A 
; 	; isPinned := DllCall(IsPinnedWindowProc, UInt, activeHwnd)
; 	; oldHwnd := activeWindowByDesktop[lParam]
; 	; isOnDesktop := DllCall(IsWindowOnCurrentVirtualDesktopProc, UInt, oldHwnd, Int)
; 	; if (isOnDesktop == 1 && isPinned != 1) {
; 	; 	WinActivate, ahk_id %oldHwnd%
; 	; }

; 	; Menu, Tray, Icon, Icons/icon%desktopNumber%.ico
	
; 	; When switching to desktop 1, set background pluto.jpg
; 	; if (lParam == 0) {
; 		; DllCall("SystemParametersInfo", UInt, 0x14, UInt, 0, Str, "C:\Users\Jarppa\Pictures\Backgrounds\saturn.jpg", UInt, 1)
; 	; When switching to desktop 2, set background DeskGmail.png
; 	; } else if (lParam == 1) {
; 		; DllCall("SystemParametersInfo", UInt, 0x14, UInt, 0, Str, "C:\Users\Jarppa\Pictures\Backgrounds\DeskGmail.png", UInt, 1)
; 	; When switching to desktop 7 or 8, set background DeskMisc.png
; 	; } else if (lParam == 2 || lParam == 3) {
; 		; DllCall("SystemParametersInfo", UInt, 0x14, UInt, 0, Str, "C:\Users\Jarppa\Pictures\Backgrounds\DeskMisc.png", UInt, 1)
; 	; Other desktops, set background to DeskWork.png
; 	; } else {
; 		; DllCall("SystemParametersInfo", UInt, 0x14, UInt, 0, Str, "C:\Users\Jarppa\Pictures\Backgrounds\DeskWork.png", UInt, 1)
; 	; }
; }

Return