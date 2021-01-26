; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：快速切换最近的窗口
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2021.01.24
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; 许可证 LICENCE: GNU GPLv3 ( https://www.gnu.org/licenses/gpl-3.0.html )
; ========== CapsLockX ==========
;
; MAIN REFERENCE:
; [Find the blinking window on the taskbar - Ask for Help - AutoHotkey Community]( https://autohotkey.com/board/topic/54990-find-the-blinking-window-on-the-taskbar/ )

; **************************************
; *** Switch to Last Flashing Window ***
; **************************************
; set trigger for flashing window

; InitFlashingWinTrigger
global lastFlashWinIDs := []
Gui +LastFound
hWnd := WinExist() , DllCall( "RegisterShellHookWindow", UInt, hWnd )
MsgNum := DllCall( "RegisterWindowMessage", Str,"SHELLHOOK" )
OnMessage( MsgNum, "ShellMessage" )
Return 

#If CapsLockXMode

*z:: ActivateLastFlashWindow()

#if


ReverseArray(oArray)
{
	Array := Object()
	For i,v in oArray
		Array[oArray.Length()-i+1] := v
	Return Array
}

arrayDistinctKeepTheLastOne(arr) { ; Hash O(n)
    hash := {}, newArr := []
    rarr := ReverseArray(arr)
    for e, v in rarr
        if (!hash.Haskey(v))
            hash[(v)] := 1, newArr.push(v)
    return ReverseArray(newArr)
}

ShellMessage( wParam,lParam ) {
    HSHELL_FLASH := 0x8006 ;  0x8006 is 32774 as shown in Spy!
    if (wParam = HSHELL_FLASH) {
        global lastFlashWinIDs
        hWnd := lParam
        lastFlashWinIDs.Push(hWnd)
        lastFlashWinIDs := arrayDistinctKeepTheLastOne(lastFlashWinIDs)
        ; lastFlashWinIDs.__Set(hWnd)
        ; WinGetTitle, this_title, ahk_id %hWnd%
        ; TrayTip, blinking, %this_title% is blinking
    }
}
; activate

ActivateLastFlashWindow(){
    While % lastFlashWinIDs.Count(){
        hWnd := WinExist("ahk_id " lastFlashWinIDs.Pop())
        if (hWnd){
            WinActivate, ahk_id %hWnd%
            WinGetTitle, this_title, ahk_id %hWnd%
            TrayTip, switched, switched to blinking %this_title%
            Return
        }
    }
    Send {Blind}!{Esc}
}