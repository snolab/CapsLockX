; ========== CapsLockX ==========
; Note：Save as UTF-8 with BOM please
; 名称：VirtualDesktopManage
; 作者：snomiao (snomiao@gmail.com)
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.07.04
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
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

if !CapsLockX
    ExitApp

Return

; Define hotkeys

#if CapsLockXMode && ExtraVirtualDesktopManageFunction

; Switch desktop left and right
[:: SendInput ^#{Left}
]:: SendInput ^#{Right}

; Move the current window to the left or right desktop
+[:: MoveActiveWindowWithAction("^#{Left}")
+]:: MoveActiveWindowWithAction("^#{Right}")

#if CapsLockXMode

; Add or delete desktop
!Backspace:: Send ^#{F4}
!+Backspace:: Send ^#d

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

; Move the current window to another desktop
MoveActiveWindowWithAction(action){
    activeWin := WinActive("A")
    WinHide ahk_id %activeWin%
    SendInput %action%
    WinShow ahk_id %activeWin%
    WinActivate ahk_id %activeWin%
}
MoveActiveWindowToNewDesktop(){
    activeWin := WinActive("A")
    WinHide ahk_id %activeWin%
    SendInput ^#d
    WinShow ahk_id %activeWin%
    WinActivate ahk_id %activeWin%
}
MoveActiveWindowToDesktop(idx){
    activeWin := WinActive("A")
    WinHide ahk_id %activeWin%
    SwitchToDesktop(idx)
    WinShow ahk_id %activeWin%
    WinActivate ahk_id %activeWin%
}
MoveAllVisibleWindowToDesktop(idx){
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
    Sleep 128
    loop Parse, listOfWindow, `n
    {
        hWnd := RegExReplace(A_LoopField, "^.*?ahk_id (\S+?)$", "$1")
        if(!hWnd)
            continue

        DllCall("ShowWindowAsync", UInt, hWnd, UInt, (SW_SHOWNOACTIVATE := 0x4) )
    }
}
SwitchToDesktop(idx){
    if (!SwitchToDesktopByInternalAPI(idx)){
        TrayTip , WARN, SwitchToDesktopByHotkey
        SwitchToDesktopByHotkey(idx)
    }
}
SwitchToDesktopByHotkey(idx){
    SendInput ^#{Left 10}
    idx -= 1
    Loop %idx% {
        SendInput ^#{Right}
    }
}

IsWindowOnCurrentVirtualDesktop(hWnd){
    IVirtualDesktopManager := ComObjCreate("{AA509086-5CA9-4C25-8F95-589D3C07B48A}", "{A5CD92FF-29BE-454C-8D04-D82879FB3F1B}")
    ; 如果这个对象不存在那就没有虚拟桌面的说法了，那就默认返回true好了
    if (!IVirtualDesktopManager)
        Return 1
    IsWindowOnCurrentVirtualDesktop := vtable(IVirtualDesktopManager, 3)
    bool := 0
    DllCall(IsWindowOnCurrentVirtualDesktop, "UPtr", IVirtualDesktopManager, "UInt", hWnd, "UIntP", bool)
    ObjRelease(IVirtualDesktopManager)
    Return %bool%
}
SwitchToDesktopByInternalAPI(idx){
    succ := 0
    IServiceProvider := ComObjCreate("{C2F03A33-21F5-47FA-B4BB-156362A2F239}", "{6D5140C1-7436-11CE-8034-00AA006009FA}")
    IVirtualDesktopManagerInternal := ComObjQuery(IServiceProvider, "{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}", "{F31574D6-B682-4CDC-BD56-1827860ABEC6}")
    ObjRelease(IServiceProvider)
    if (IVirtualDesktopManagerInternal){
        GetCount := vtable(IVirtualDesktopManagerInternal, 3)
        GetDesktops := vtable(IVirtualDesktopManagerInternal, 7)
        SwitchDesktop := vtable(IVirtualDesktopManagerInternal, 9)
        ; TrayTip, , % IVirtualDesktopManagerInternal
        pDesktopIObjectArray := 0
        DllCall(GetDesktops, "Ptr", IVirtualDesktopManagerInternal, "Ptr*", pDesktopIObjectArray)
        if (pDesktopIObjectArray){
            GetDesktopCount := vtable(pDesktopIObjectArray, 3)
            GetDesktopAt := vtable(pDesktopIObjectArray, 4)
            DllCall(GetDesktopCount, "Ptr", IVirtualDesktopManagerInternal, "UInt*", DesktopCount)
            ; if idx-th desktop doesn't exists then create a new desktop
            if (idx > DesktopCount){
                diff := idx - DesktopCount
                loop %diff% {
                    Send ^#d
                }
                succ := 1
            }
            GetGUIDFromString(IID_IVirtualDesktop, "{FF72FFDD-BE7E-43FC-9C03-AD81681E88E4}")
            DllCall(GetDesktopAt, "Ptr", pDesktopIObjectArray, "UInt", idx - 1, "Ptr", &IID_IVirtualDesktop, "Ptr*", VirtualDesktop)
            ObjRelease(pDesktopIObjectArray)
            if (VirtualDesktop){
                DllCall(SwitchDesktop, "Ptr", IVirtualDesktopManagerInternal, "Ptr", VirtualDesktop)
                ObjRelease(VirtualDesktop)
                succ := 1
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

vtable(ptr, n){
    ; NumGet(ptr+0) Returns the address of the object's virtual function
    ; table (vtable for short). The remainder of the expression retrieves
    ; the address of the nth function's address from the vtable.
    Return NumGet(NumGet(ptr+0), n*A_PtrSize)
}
