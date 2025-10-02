; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：VirtualDesktopManage
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.07.04
; 版权：Copyright © 2017-2024 Snowstar Laboratory. All Rights Reserved.
; LICENCE: GNU GPLv3
; ========== CapsLockX ==========
;
; ref list:
;
; ahk forum:
; [[Windows 10] Switch to different virtual desktop on Win+{1, 9} - AutoHotkey Community]( https://www.autohotkey.com/boards/viewtopic.php?t=14881 )
; [How to call a Win32 API with UUID [IVirtualDesktopManager] - AutoHotkey Community]( https://www.autohotkey.com/boards/viewtopic.php?t=54202 )

; m$:
; [IVirtualDesktopManager (shobjidl_core.h) - Win32 apps | Microsoft Docs]( https://docs.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ivirtualdesktopmanager )
; [IVirtualDesktopManager::MoveWindowToDesktop (shobjidl_core.h) - Win32 apps | Microsoft Docs]( https://docs.microsoft.com/zh-cn/windows/win32/api/shobjidl_core/nf-shobjidl_core-ivirtualdesktopmanager-movewindowtodesktop?redirectedfrom=MSDN )

; API definitions
; [PowerShell Gallery | VirtualDesktop.ps1 1.1.0]( https://www.powershellgallery.com/packages/VirtualDesktop/1.1.0/Content/VirtualDesktop.ps1 )
; [Windows 10 の仮想デスクトップを制御しようとして失敗した話 | grabacr.nét]( http://grabacr.net/archives/5601 )
;
; ref project when updated to win11:
; [MScholtes/VirtualDesktop: C# command line tool to manage virtual desktops in Windows 10]( https://github.com/MScholtes/VirtualDesktop )
;
; 2024-05-19 - [VirtualDesktop/VirtualDesktop11.cs at master · MScholtes/VirtualDesktop]( https://github.com/MScholtes/VirtualDesktop/blob/master/VirtualDesktop11.cs )
; 

CLSID_ImmersiveShell := "{C2F03A33-21F5-47FA-B4BB-156362A2F239}"
CLSID_IServiceProvider10 := "{6D5140C1-7436-11CE-8034-00AA006009FA}"
CLSID_IVirtualDesktop_Win10 := "{FF72FFDD-BE7E-43FC-9C03-AD81681E88E4}"
CLSID_IVirtualDesktop_Win11 :="{536D3495-B208-4CC9-AE26-DE8111275BF8}"
CLSID_IVirtualDesktop_Win12 :="{3F07F4BE-B107-441A-AF0F-39D82529072C}"
CLSID_IVirtualDesktopManager := "{A5CD92FF-29BE-454C-8D04-D82879FB3F1B}"
CLSID_IVirtualDesktopManagerInternal_Win10 := "{F31574D6-B682-4CDC-BD56-1827860ABEC6}"
CLSID_IVirtualDesktopManagerInternal_Win11 := "{B2F925B9-5A0F-4D2E-9F4D-2B1507593C10}"
CLSID_IVirtualDesktopManagerInternal_Win12 := "{53F5CA0B-158F-4124-900C-057158060B27}"
CLSID_VirtualDesktopManager := "{AA509086-5CA9-4C25-8F95-589D3C07B48A}"
CLSID_VirtualDesktopManagerInternal := "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}"
CLSID_VirtualDesktopPinnedApps := "{B5A399E7-1C87-46B8-88E9-FC5747B171BD}"

if (!CapsLockX) {
    ExitApp
}

; @deprecated start, will remove in future version
global VirtualDesktopPinPattern1 := CLX_Config("VirtualDesktopPinPattern", "p1", "#Desktop1", "Pin matched window to desktop 1")
global VirtualDesktopPinPattern2 := CLX_Config("VirtualDesktopPinPattern", "p2", "#Desktop2", "Pin matched window to desktop 2")
global VirtualDesktopPinPattern3 := CLX_Config("VirtualDesktopPinPattern", "p3", "#Desktop3", "Pin matched window to desktop 3")
global VirtualDesktopPinPattern4 := CLX_Config("VirtualDesktopPinPattern", "p4", "#Desktop4", "Pin matched window to desktop 4")
global VirtualDesktopPinPattern5 := CLX_Config("VirtualDesktopPinPattern", "p5", "#Desktop5", "Pin matched window to desktop 5")
global VirtualDesktopPinPattern6 := CLX_Config("VirtualDesktopPinPattern", "p6", "#Desktop6", "Pin matched window to desktop 6")
global VirtualDesktopPinPattern7 := CLX_Config("VirtualDesktopPinPattern", "p7", "#Desktop7", "Pin matched window to desktop 7")
global VirtualDesktopPinPattern8 := CLX_Config("VirtualDesktopPinPattern", "p8", "#Desktop8", "Pin matched window to desktop 8")
global VirtualDesktopPinPattern9 := CLX_Config("VirtualDesktopPinPattern", "p9", "#Desktop9", "Pin matched window to desktop 9")
global VirtualDesktopPinPattern0 := CLX_Config("VirtualDesktopPinPattern", "p0", "#Desktop0", "Pin matched window to desktop 0")
; @deprecated end

; Global variable to track current desktop index (fallback when API fails)
global CurrentVirtualDesktopIdx := 1

Return

; Define hotkeys

#if CapsLockXMode && ExtraVirtualDesktopManageFunction

; Switch desktop left and right
[:: Func("SwitchToPrevDesktop").Call()
]:: Func("SwitchToNextDesktop").Call()

; Move the current window to the left or right desktop
+[:: MoveActiveWindowWithAction("^#{Left}")
+]:: MoveActiveWindowWithAction("^#{Right}")

#if CapsLockXMode

; Add or delete desktop
!Backspace:: SendEvent ^#{F4}
!+Backspace:: SendEvent ^#d

; Switch to desktop
1:: SwitchToDesktop(1)
2:: SwitchToDesktop(2)
3:: SwitchToDesktop(3)
4:: SwitchToDesktop(4)
5:: SwitchToDesktop(5)
6:: SwitchToDesktop(6)
7:: SwitchToDesktop(7)
8:: SwitchToDesktop(8)
9:: SwitchToDesktop(9)
0:: SwitchToDesktop(10)
; -:: SwitchToDesktop(11)
; =:: SwitchToDesktop(12)

; Move the current window to the X-th desktop
+1:: MoveActiveWindowToDesktop(1)
+2:: MoveActiveWindowToDesktop(2)
+3:: MoveActiveWindowToDesktop(3)
+4:: MoveActiveWindowToDesktop(4)
+5:: MoveActiveWindowToDesktop(5)
+6:: MoveActiveWindowToDesktop(6)
+7:: MoveActiveWindowToDesktop(7)
+8:: MoveActiveWindowToDesktop(8)
+9:: MoveActiveWindowToDesktop(9)
+0:: MoveActiveWindowToDesktop(10)
; +-:: MoveActiveWindowToDesktop(11)
; +=:: MoveActiveWindowToDesktop(12)
; Move the ALL visible window to the X-th desktop
!+1:: MoveAllVisibleWindowToDesktop(1)
!+2:: MoveAllVisibleWindowToDesktop(2)
!+3:: MoveAllVisibleWindowToDesktop(3)
!+4:: MoveAllVisibleWindowToDesktop(4)
!+5:: MoveAllVisibleWindowToDesktop(5)
!+6:: MoveAllVisibleWindowToDesktop(6)
!+7:: MoveAllVisibleWindowToDesktop(7)
!+8:: MoveAllVisibleWindowToDesktop(8)
!+9:: MoveAllVisibleWindowToDesktop(9)
!+0:: MoveAllVisibleWindowToDesktop(10)
; !+-:: MoveAllVisibleWindowToDesktop(11)
; !+=:: MoveAllVisibleWindowToDesktop(12)

; API definitions
/*
[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("FF72FFDD-BE7E-43FC-9C03-AD81681E88E4")]
internal interface IVirtualDesktop
{
bool IsViewVisible(IApplicationView view);
Guid GetId();
}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("F31574D6-B682-4CDC-BD56-1827860ABEC6")]
internal interface IVirtualDesktopManagerInternal
{
int GetCount();
void MoveViewToDesktop(IApplicationView view, IVirtualDesktop desktop);
bool CanViewMoveDesktops(IApplicationView view);
IVirtualDesktop GetCurrentDesktop();
void GetDesktops(out IObjectArray desktops);
[PreserveSig]
int GetAdjacentDesktop(IVirtualDesktop from, int direction, out IVirtualDesktop desktop);
void SwitchDesktop(IVirtualDesktop desktop);
IVirtualDesktop CreateDesktop();
void RemoveDesktop(IVirtualDesktop desktop, IVirtualDesktop fallback);
IVirtualDesktop FindDesktop(ref Guid desktopid);
}
1
[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("A5CD92FF-29BE-454C-8D04-D82879FB3F1B")]
internal interface IVirtualDesktopManager
{
bool IsWindowOnCurrentVirtualDesktop(IntPtr topLevelWindow);
Guid GetWindowDesktopId(IntPtr topLevelWindow);
void MoveWindowToDesktop(IntPtr topLevelWindow, ref Guid desktopId);
}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("4CE81583-1E4C-4632-A621-07A53543148F")]
internal interface IVirtualDesktopPinnedApps
{
bool IsAppIdPinned(string appId);
void PinAppID(string appId);
void UnpinAppID(string appId);
bool IsViewPinned(IApplicationView applicationView);
void PinView(IApplicationView applicationView);
void UnpinView(IApplicationView applicationView);
}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("92CA9DCD-5622-4BBA-A805-5E9F541BD8C9")]
internal interface IObjectArray
{
void GetCount(out int count);
void GetAt(int index, ref Guid iid, [MarshalAs(UnmanagedType.Interface)]out object obj);
}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("6D5140C1-7436-11CE-8034-00AA006009FA")]
internal interface IServiceProvider10
{
[Return: MarshalAs(UnmanagedType.IUnknown)]
object QueryService(ref Guid service, ref Guid riid);
}

*/

; api define
; IServiceProvider                := ComObjCreate("{C2F03A33-21F5-47FA-B4BB-156362A2F239}", "{6D5140C1-7436-11CE-8034-00AA006009FA}")
; IVirtualDesktopManagerInternal  := ComObjQuery(IServiceProvider, "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}", "{F31574D6-B682-4CDC-BD56-1827860ABEC6}")
; GetCount                        := vtable(IVirtualDesktopManagerInternal, 3)
; MoveViewDesktop                 := vtable(IVirtualDesktopManagerInternal, 4)
; CanViewMoveDesktops             := vtable(IVirtualDesktopManagerInternal, 5)
; GetCurrentDesktop               := vtable(IVirtualDesktopManagerInternal, 6)
; GetDesktops                     := vtable(IVirtualDesktopManagerInternal, 7)
; GetAdjacentDesktop              := vtable(IVirtualDesktopManagerInternal, 8)
; SwitchDesktop                   := vtable(IVirtualDesktopManagerInternal, 9)
; CreateDesktopW                  := vtable(IVirtualDesktopManagerInternal, 10)
; RemoveDesktop                   := vtable(IVirtualDesktopManagerInternal, 11)
; FindDesktop                     := vtable(IVirtualDesktopManagerInternal, 12)

; IVirtualDesktopManager          := ComObjCreate("{AA509086-5CA9-4C25-8F95-589D3C07B48A}", "{A5CD92FF-29BE-454C-8D04-D82879FB3F1B}")
; IsWindowOnCurrentVirtualDesktop := vtable(IVirtualDesktopManager, 3)
; GetWindowDesktopId              := vtable(IVirtualDesktopManager, 4)
; MoveWindowToDesktop             := vtable(IVirtualDesktopManager, 5)

#if ; Define Functions

; Get current virtual desktop index
; First tries to get from API, falls back to global variable if API fails
GetCurrentVirtualDesktopIdx()
{
    global CurrentVirtualDesktopIdx

    ; Try to get current desktop from API
    idx := GetCurrentVirtualDesktopIdxFromAPI()
    if (idx) {
        ; Update our cached value
        CurrentVirtualDesktopIdx := idx
        return idx
    }

    ; API failed, return cached value
    return CurrentVirtualDesktopIdx
}

; Helper function to get current desktop index from Windows API
GetCurrentVirtualDesktopIdxFromAPI()
{
    idx := 0

    CLSID_ImmersiveShell := "{C2F03A33-21F5-47FA-B4BB-156362A2F239}"
    CLSID_IServiceProvider10 := "{6D5140C1-7436-11CE-8034-00AA006009FA}"
    CLSID_IVirtualDesktop_Win12 := "{3F07F4BE-B107-441A-AF0F-39D82529072C}"
    CLSID_IVirtualDesktop_Win11 := "{536D3495-B208-4CC9-AE26-DE8111275BF8}"
    CLSID_IVirtualDesktop_Win10 := "{FF72FFDD-BE7E-43FC-9C03-AD81681E88E4}"
    CLSID_VirtualDesktopManagerInternal := "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}"
    CLSID_IVirtualDesktopManagerInternal_Win12 := "{53F5CA0B-158F-4124-900C-057158060B27}"
    CLSID_IVirtualDesktopManagerInternal_Win11 := "{B2F925B9-5A0F-4D2E-9F4D-2B1507593C10}"
    CLSID_IVirtualDesktopManagerInternal_Win10 := "{F31574D6-B682-4CDC-BD56-1827860ABEC6}"

    try {
        IServiceProvider := ComObjCreate(CLSID_ImmersiveShell, CLSID_IServiceProvider10)
        IVirtualDesktopManagerInternal_Win12 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win12)
        IVirtualDesktopManagerInternal_Win11 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win11)
        IVirtualDesktopManagerInternal_Win10 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win10)

        win12 := !!IVirtualDesktopManagerInternal_Win12
        win11 := !!IVirtualDesktopManagerInternal_Win11
        win10 := !!IVirtualDesktopManagerInternal_Win10

        _ := win12 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win12)
        _ := win11 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win11)
        _ := win10 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win10)

        ObjRelease(IServiceProvider)

        if (IVirtualDesktopManagerInternal) {
            GetCurrentDesktop := vtable(IVirtualDesktopManagerInternal, 6)
            GetDesktops := vtable(IVirtualDesktopManagerInternal, 7)

            ; Get current desktop
            pCurrentDesktop := 0
            _ := win12 && DllCall(GetCurrentDesktop, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pCurrentDesktop)
            _ := win11 && DllCall(GetCurrentDesktop, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "Ptr*", pCurrentDesktop)
            _ := win10 && DllCall(GetCurrentDesktop, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pCurrentDesktop)

            if (pCurrentDesktop) {
                ; Get all desktops
                pDesktopIObjectArray := 0
                _ := win12 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pDesktopIObjectArray)
                _ := win11 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "Ptr*", pDesktopIObjectArray)
                _ := win10 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pDesktopIObjectArray)

                if (pDesktopIObjectArray) {
                    GetDesktopCount := vtable(pDesktopIObjectArray, 3)
                    GetDesktopAt := vtable(pDesktopIObjectArray, 4)

                    _ := win12 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "UInt*", DesktopCount)
                    _ := win11 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "UInt*", DesktopCount)
                    _ := win10 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "UInt*", DesktopCount)

                    ; Find which desktop index matches current desktop
                    _ := win12 && GetGUIDFromString(IID_IVirtualDesktop, CLSID_IVirtualDesktop_Win12)
                    _ := win11 && GetGUIDFromString(IID_IVirtualDesktop, CLSID_IVirtualDesktop_Win11)
                    _ := win10 && GetGUIDFromString(IID_IVirtualDesktop, CLSID_IVirtualDesktop_Win10)

                    Loop %DesktopCount% {
                        pDesktop := 0
                        DllCall(GetDesktopAt, "Ptr", pDesktopIObjectArray, "UInt", A_Index - 1, "Ptr", &IID_IVirtualDesktop, "Ptr*", pDesktop)
                        if (pDesktop == pCurrentDesktop) {
                            idx := A_Index
                            ObjRelease(pDesktop)
                            break
                        }
                        if (pDesktop) {
                            ObjRelease(pDesktop)
                        }
                    }
                    ObjRelease(pDesktopIObjectArray)
                }
                ObjRelease(pCurrentDesktop)
            }
            ObjRelease(IVirtualDesktopManagerInternal)
        }
    } catch {
        ; API call failed, return 0
        idx := 0
    }

    return idx
}

; Move the current window to another desktop
MoveActiveWindowWithAction(action)
{
    global CurrentVirtualDesktopIdx

    activeWin := WinActive("A")
    WinHide ahk_id %activeWin%
    SendInput %action%

    ; Update index based on action
    if (action == "^#{Left}") {
        CurrentVirtualDesktopIdx := max(1, CurrentVirtualDesktopIdx - 1)
    } else if (action == "^#{Right}") {
        CurrentVirtualDesktopIdx := CurrentVirtualDesktopIdx + 1
    }

    WinShow ahk_id %activeWin%
    WinActivate ahk_id %activeWin%
}
MoveActiveWindowToNewDesktop()
{
    global CurrentVirtualDesktopIdx

    activeWin := WinActive("A")
    WinHide ahk_id %activeWin%
    SendInput ^#d

    ; When creating a new desktop, increment the index
    CurrentVirtualDesktopIdx := CurrentVirtualDesktopIdx + 1

    WinShow ahk_id %activeWin%
    WinActivate ahk_id %activeWin%
}
MoveActiveWindowToDesktop(idx)
{
    global CurrentVirtualDesktopIdx

    activeWin := WinActive("A")
    WinHide ahk_id %activeWin%
    SwitchToDesktop(idx)
    CurrentVirtualDesktopIdx := idx
    WinShow ahk_id %activeWin%
    WinActivate ahk_id %activeWin%
}
MoveAllVisibleWindowToDesktop(idx)
{
    global CurrentVirtualDesktopIdx

    listOfWindow := WindowsListOfMonitorFast(arrangeFlags | ARRANGE_MAXWINDOW | ARRANGE_MINWINDOW)

    loop Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        if(!hWnd)
            continue
        ; WinHide ahk_id %hWnd%
        DllCall("ShowWindowAsync", UInt, hWnd, UInt, (SW_HIDE := 0x0) )
    }
    SwitchToDesktop(idx)
    CurrentVirtualDesktopIdx := idx
    Sleep 128
    loop Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        if(!hWnd)
            continue

        DllCall("ShowWindowAsync", UInt, hWnd, UInt, (SW_SHOWNOACTIVATE := 0x4) )
    }
}
SwitchToDesktop(idx)
{
    global CurrentVirtualDesktopIdx

    if (SwitchToDesktopByInternalAPI(idx)) {
        ; ok
        CurrentVirtualDesktopIdx := idx
    } else if (SwitchToDesktopByHotkey(idx)) {
        ; Tooltip, WARN SwitchToDesktopByHotkey %idx%
        CurrentVirtualDesktopIdx := idx
    } else {
        Tooltip, WARN SwitchToDesktop FAILED
    }
    EnsureCurrentEnviromentRule(idx)
    return idx
}

SwitchToNextDesktop()
{
    currentIdx := GetCurrentVirtualDesktopIdx()

    ; Get total desktop count
    desktopCount := GetVirtualDesktopCount()
    if (!desktopCount) {
        ; If we can't get count from API, assume max 10 desktops
        desktopCount := 10
    }

    ; Calculate next index with wrap around
    nextIdx := currentIdx + 1
    if (nextIdx > desktopCount) {
        ; Wrap around to first desktop
        nextIdx := 1
    }

    return SwitchToDesktop(nextIdx)
}

SwitchToPrevDesktop()
{
    currentIdx := GetCurrentVirtualDesktopIdx()

    ; Get total desktop count for wrap around
    desktopCount := GetVirtualDesktopCount()
    if (!desktopCount) {
        ; If we can't get count from API, assume max 10 desktops
        desktopCount := 10
    }

    ; Calculate previous index with wrap around
    prevIdx := currentIdx - 1
    if (prevIdx < 1) {
        ; Wrap around to last desktop
        prevIdx := desktopCount
    }

    return SwitchToDesktop(prevIdx)
}

; Helper function to get total desktop count
GetVirtualDesktopCount()
{
    count := 0

    CLSID_ImmersiveShell := "{C2F03A33-21F5-47FA-B4BB-156362A2F239}"
    CLSID_IServiceProvider10 := "{6D5140C1-7436-11CE-8034-00AA006009FA}"
    CLSID_VirtualDesktopManagerInternal := "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}"
    CLSID_IVirtualDesktopManagerInternal_Win12 := "{53F5CA0B-158F-4124-900C-057158060B27}"
    CLSID_IVirtualDesktopManagerInternal_Win11 := "{B2F925B9-5A0F-4D2E-9F4D-2B1507593C10}"
    CLSID_IVirtualDesktopManagerInternal_Win10 := "{F31574D6-B682-4CDC-BD56-1827860ABEC6}"

    try {
        IServiceProvider := ComObjCreate(CLSID_ImmersiveShell, CLSID_IServiceProvider10)
        IVirtualDesktopManagerInternal_Win12 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win12)
        IVirtualDesktopManagerInternal_Win11 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win11)
        IVirtualDesktopManagerInternal_Win10 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win10)

        win12 := !!IVirtualDesktopManagerInternal_Win12
        win11 := !!IVirtualDesktopManagerInternal_Win11
        win10 := !!IVirtualDesktopManagerInternal_Win10

        _ := win12 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win12)
        _ := win11 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win11)
        _ := win10 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win10)

        ObjRelease(IServiceProvider)

        if (IVirtualDesktopManagerInternal) {
            GetCount := vtable(IVirtualDesktopManagerInternal, 3)
            GetDesktops := vtable(IVirtualDesktopManagerInternal, 7)

            pDesktopIObjectArray := 0
            _ := win12 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pDesktopIObjectArray)
            _ := win11 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "Ptr*", pDesktopIObjectArray)
            _ := win10 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pDesktopIObjectArray)

            if (pDesktopIObjectArray) {
                GetDesktopCount := vtable(pDesktopIObjectArray, 3)
                _ := win12 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "UInt*", count)
                _ := win11 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "UInt*", count)
                _ := win10 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "UInt*", count)

                ObjRelease(pDesktopIObjectArray)
            }
            ObjRelease(IVirtualDesktopManagerInternal)
        }
    } catch {
        ; API call failed
        count := 0
    }

    return count
}
EnsureCurrentEnviromentRule(idx)
{
    listOfWindow := WindowsListInAllVirtualDesktop()
    out:=""
    pinPatterns := VirtualDesktopPinPattern1 "`n" VirtualDesktopPinPattern2 "`n" VirtualDesktopPinPattern3 "`n" VirtualDesktopPinPattern4 "`n" VirtualDesktopPinPattern5 "`n" VirtualDesktopPinPattern6 "`n" VirtualDesktopPinPattern7 "`n" VirtualDesktopPinPattern8 "`n" VirtualDesktopPinPattern9 "`n" VirtualDesktopPinPattern0
    loop, Parse, listOfWindow, `n
    {
        win := A_LoopField
        hWnd := RegExReplace(win, "^.*?ahk_id (\S+).*?$", "$1")
        if (IsWindowOnCurrentVirtualDesktop(hWnd)) {
            continue
        }
        k:= 0
        loop, Parse, pinPatterns, `n
        {
            k := k + 1
            pattern := A_LoopField
            if (idx == k && RegExMatch(win, "i)" pattern)) {
                WinHide ahk_id %hWnd%
                WinShow ahk_id %hWnd%
            }
        }
    }
    tooltip %out%
}
SwitchToDesktopByHotkey(idx)
{
    static lastIdx := ""
    if (!lastIdx || idx == 1) {
        Loop 10 {
            SendEvent ^#{Left}
        }
        lastIdx := 1
    }
    offset := idx - lastIdx
    offsetRight := max(offset, 0)
    offsetLeft := max(-offset, 0)
    Loop %offsetRight% {
        SendEvent ^#{Right}
    }
    Loop %offsetLeft% {
        SendEvent ^#{Left}
    }
    lastIdx := idx

    return idx
}

IsWindowOnCurrentVirtualDesktop(hWnd)
{
    IVirtualDesktopManager := ComObjCreate("{AA509086-5CA9-4C25-8F95-589D3C07B48A}", "{A5CD92FF-29BE-454C-8D04-D82879FB3F1B}")
    ; 如果这个对象不存在那就没有虚拟桌面的说法了，那它一定在当前桌面下就默认返回true好了
    if (!IVirtualDesktopManager)
        Return true
    IsWindowOnCurrentVirtualDesktop := vtable(IVirtualDesktopManager, 3)
    bool := 0
    DllCall(IsWindowOnCurrentVirtualDesktop, "UPtr", IVirtualDesktopManager, "UInt", hWnd, "UIntP", bool)
    ObjRelease(IVirtualDesktopManager)
    Return %bool%
}

SwitchToDesktopByInternalAPI(idx)
{
    succ := 0
    CLSID_ImmersiveShell := "{C2F03A33-21F5-47FA-B4BB-156362A2F239}"
    CLSID_IServiceProvider10 := "{6D5140C1-7436-11CE-8034-00AA006009FA}"
    CLSID_IVirtualDesktop_Win12 :="{3F07F4BE-B107-441A-AF0F-39D82529072C}"
    CLSID_IVirtualDesktop_Win11 :="{536D3495-B208-4CC9-AE26-DE8111275BF8}"
    CLSID_IVirtualDesktop_Win10 := "{FF72FFDD-BE7E-43FC-9C03-AD81681E88E4}"
    CLSID_IVirtualDesktopManager := "{A5CD92FF-29BE-454C-8D04-D82879FB3F1B}"
    CLSID_IVirtualDesktopManagerInternal_Win12 := "{53F5CA0B-158F-4124-900C-057158060B27}"
    CLSID_IVirtualDesktopManagerInternal_Win11 := "{B2F925B9-5A0F-4D2E-9F4D-2B1507593C10}"
    CLSID_IVirtualDesktopManagerInternal_Win10 := "{F31574D6-B682-4CDC-BD56-1827860ABEC6}"
    CLSID_VirtualDesktopManager := "{AA509086-5CA9-4C25-8F95-589D3C07B48A}"
    CLSID_VirtualDesktopManagerInternal := "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}"
    CLSID_VirtualDesktopPinnedApps := "{B5A399E7-1C87-46B8-88E9-FC5747B171BD}"

    IServiceProvider := ComObjCreate(CLSID_ImmersiveShell, CLSID_IServiceProvider10)
    IVirtualDesktopManagerInternal_Win12 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win12)
    IVirtualDesktopManagerInternal_Win11 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win11)
    IVirtualDesktopManagerInternal_Win10 := ComObjQuery(IServiceProvider, CLSID_VirtualDesktopManagerInternal, CLSID_IVirtualDesktopManagerInternal_Win10)
    win12 := !!IVirtualDesktopManagerInternal_Win12
    win11 := !!IVirtualDesktopManagerInternal_Win11
    win10 := !!IVirtualDesktopManagerInternal_Win10
    _:= win12 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win12)
    _:= win11 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win11)
    _:= win10 && (IVirtualDesktopManagerInternal := IVirtualDesktopManagerInternal_Win10)
    ; ToolTip, % "win12-" . win12 . "win11-" . win11 . "win10-" . win10 . ""
    ; ToolTip win %win12% %win11% %win10%

    ObjRelease(IServiceProvider)
    if (IVirtualDesktopManagerInternal) {
        ; tooltip %idx%
        GetCount := vtable(IVirtualDesktopManagerInternal, 3)
        GetDesktops := vtable(IVirtualDesktopManagerInternal, 7)
        SwitchDesktop := vtable(IVirtualDesktopManagerInternal, 9)
        
        ; TrayTip, , % IVirtualDesktopManagerInternal
        pDesktopIObjectArray := 0
        _ := win12 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pDesktopIObjectArray)
        _ := win11 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "Ptr*", pDesktopIObjectArray)
        _ := win10 && DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pDesktopIObjectArray)
        
        ; Tooltip % pDesktopIObjectArray
        if (pDesktopIObjectArray) {
            GetDesktopCount := vtable(pDesktopIObjectArray, 3)
            GetDesktopAt := vtable(pDesktopIObjectArray, 4)
            _ := win12 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "UInt*", DesktopCount)
            _ := win11 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "UInt*", DesktopCount)
            _ := win10 && DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "UInt*", DesktopCount)

            ; TrayTip, CapsLockX, % t("切换到桌面：") . idx . "/" . DesktopCount
            ; if idx-th desktop doesn't exists then create a new desktop
            if (idx > DesktopCount) {
                diff := idx - DesktopCount
                loop %diff% {
                    SendEvent ^#d
                }
            }
            ; if desktop count is more than 10 then delete them
            if (DesktopCount > 10) {
                delCount := DesktopCount - 10 + 1
                SendEvent ^#d
                loop %delCount% {
                    SendEvent ^#{F4}
                }
            }
            _ := win12 && GetGUIDFromString(IID_IVirtualDesktop, CLSID_IVirtualDesktop_Win12)
            _ := win11 && GetGUIDFromString(IID_IVirtualDesktop, CLSID_IVirtualDesktop_Win11)
            _ := win10 && GetGUIDFromString(IID_IVirtualDesktop, CLSID_IVirtualDesktop_Win10)
            DllCall(GetDesktopAt, "Ptr", pDesktopIObjectArray, "UInt", idx - 1, "Ptr", &IID_IVirtualDesktop, "Ptr*", VirtualDesktop)
            ObjRelease(pDesktopIObjectArray)
            ; ToolTip, % "clx" . VirtualDesktop
            if (VirtualDesktop) {
                _ := win12 && DllCall(SwitchDesktop, "Ptr", IVirtualDesktopManagerInternal, "Ptr", VirtualDesktop)
                _ := win11 && DllCall(SwitchDesktop, "Ptr", IVirtualDesktopManagerInternal, "Ptr", 0, "Ptr", VirtualDesktop)
                _ := win10 && DllCall(SwitchDesktop, "Ptr", IVirtualDesktopManagerInternal, "Ptr", VirtualDesktop)
                ObjRelease(VirtualDesktop)
                succ := idx
            }
        }
        ObjRelease(IVirtualDesktopManagerInternal)
    }
    Return succ
}

GetGUIDFromString(ByRef GUID, sGUID) ; Converts a string to a binary GUID
{
    VarSetCapacity(GUID, 16, 0)
    DllCall("ole32\CLSIDFromString", "Str", sGUID, "Ptr", &GUID)
}

vtable(ptr, n)
{
    ; NumGet(ptr+0) Returns the address of the object's virtual function
    ; table (vtable for short). The remainder of the expression retrieves
    ; the address of the nth function's address from the vtable.
    Return NumGet(NumGet(ptr+0), n*A_PtrSize)
}
WindowsListInAllVirtualDesktop()
{
    windowsMatches := ""
    ; 常量定义
    WS_EX_TOOLWINDOW := 0x00000080
    WS_EX_APPWINDOW := 0x00040000
    WS_CAPTION := 0x00C00000
    WS_EX_NOANIMATION := 0x04000000
    WS_EX_NOACTIVATE := 0x08000000
    WS_POPUP := 0x80000000
    DetectHiddenWindows, Off
    WinGet, id, List, , ,
    loop %id% {
        hWnd := id%A_Index%
        filter := !WindowsListOfMonitorInAllVirtualDesktopFilter(hWnd)
        if (filter) {
            continue
        }
        WinGet, this_exe, ProcessName, ahk_id %hWnd%
        WinGetTitle, this_title, ahk_id %hWnd%
        windowsMatches .= "ahk_exe " this_exe " ahk_id " hWnd " " . this_title . "`n"
        ; windowsMatches .= "ahk_pid " this_pid " ahk_id " hWnd "`n" ; . "`t" . this_title . "`n"
    }
    Sort windowsMatches, R
    return windowsMatches
}
WindowsListOfMonitorInAllVirtualDesktopFilter(hWnd)
{
    ; 常量定义
    WS_EX_TOOLWINDOW := 0x00000080
    WS_EX_APPWINDOW := 0x00040000
    WS_CAPTION := 0x00C00000
    WS_EX_NOANIMATION := 0x04000000
    WS_EX_NOACTIVATE := 0x08000000
    WS_POPUP := 0x80000000
    WinGet, style, style, ahk_id %hWnd%
    ; ; 跳过无标题窗口
    ; if !(style & WS_CAPTION)
    ;     Continue
    ; ; 跳过工具窗口
    ; if (style & WS_EX_TOOLWINDOW)
    ;     Continue
    ; if (style & WS_POPUP)
    ;     Continue
    ; 只显示Alt+TAB里有的窗口
    if (!(style & WS_EX_APPWINDOW)) {
        return False ; ; 跳 过弹出窗口
    }
    ; ToolTip, %hWnd% mi %MonitorIndex%
    ; 尝试跳过隐藏窗口
    GWL_STYLE := -16
    GWL_EXSTYLE := -20
    ; WS_STYLE := DllCall("GetWindowLong" (A_PtrSize=8 ? "Ptr" : ""), "Ptr", hWnd, "Int", GWL_STYLE, "PTR")
    WS_VISIBLE := 0x10000000
    if (!(style & WS_VISIBLE)) {
        return False
    }
    ; 跳过不在当前虚拟桌面的窗口
    ; if (!IsWindowOnCurrentVirtualDesktop(hWnd)) {
    ;     return False
    ; }
    ; 排除不归属于当前参数显示器的窗口
    ; if (!!MonitorIndex) {
    ;     this_monitor := GetMonitorIndexFromWindow(hWnd)
    ;     if (MonitorIndex != this_monitor) {
    ;         return False
    ;     }
    ; }
    ; 尝试跳过隐藏窗口
    if ( !DllCall("IsWindowVisible", "Ptr", hWnd, "PTR") ) {
        return False
    }
    ; ; 跳过最大化窗口
    ; WinGet, minmax, minmax, ahk_id %hWnd%
    ; if (minmax == 1 && !(arrangeFlags & ARRANGE_MAXWINDOW)) {
    ;     return False
    ; }
    ; ; 跳过最小化的窗口
    ; if (minmax == -1 && !(arrangeFlags & ARRANGE_MINWINDOW)) {
    ;     return False
    ; }
    WinGetTitle, this_title, ahk_id %hWnd%
    ; 排除空标题窗口
    if (!RegExMatch(this_title, ".+")) {
        return False ; If (this_class == "Progman") ; return False ; 排除 Win10 的常驻窗口管理器
    }
    ; 跳过不可见的 UWP 窗口
    WinGetClass, this_class, ahk_id %hWnd%
    if ( this_class == "ApplicationFrameWindow") {
        return False
    }
    ; true
    return True
}