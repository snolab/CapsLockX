; placeholder
; ; 
; ; ; SnowstarCyan_Monitor_Libriay
; ; ; Updates:
; ; ; 2016y 01m 28d
; ; ; Created By Snowstar (SnowstarCyan@gmail.com)
; ; ; For rotate my screens to play computer games on bed at winter. ~(≧▽≦)~
; ; 
; ; API Functions for Reference
; ; ChangeDisplaySettingsA    := DllCall("User32.dll\ChangeDisplaySettingsA", "Ptr", &lpDevMode, "Int", dwflags)
; ; ChangeDisplaySettingsExA  := DllCall("User32.dll\ChangeDisplaySettingsExA", "Str", DeviceName, "Ptr", &lpDevMode, "Ptr", HWND, "Int", dwflags, "Ptr", lParam)
; ; EnumDisplaySettingsA      := DllCall("User32.dll\EnumDisplaySettingsA", "Str", DeviceName, "Int", iModeNum, "Ptr", &lpDevMode)
; ; EnumDisplayDevicesA       := DllCall("User32.dll\EnumDisplayDevicesA", "Str", DeviceName, "Int", iDevNum, "Ptr", &lpDevMode, "Int", dwflags)

; ; MsgBox % "EnumDisplayDevicesA" . EnumDisplayDevicesA
; ; 
; ; Monitor.RotateOneMonitor(90)
; ; len := GetDisplayDevices().length

; ; MsgBox, % "len" . len

; class DEVMODE_DISPLAY_DEVICE {
;     __New() {
;         this.dmSize := 156
;         this.dmFields := 0
;     }
;     __Set(key, value) {
;         if (key == "position_x") || (key == "position_y")
;             this.dmPosition := &value
;         else
;             this[key] := value
;     }
; }

; class DISPLAY_DEVICE {
;     __New() {
;         this.cb := 424
;     }
; }

; ; _DISPLAY_DEVICE_ATTACHED_TO_DESKTOP := 1

; Monitor := {}
; Monitor.MapByFunc := Func("MapByFunc")
; Monitor.Merge := Func("Merge")
; Monitor.GetDeviceMode := Func("GetDeviceMode")
; Monitor.GetDeviceName := Func("GetDeviceName")
; Monitor.RotateDevMode90 := Func("RotateDevMode90")
; Monitor.ApplyDevMode := Func("ApplyDevMode")
; Monitor.GetDisplayDevices := Func("GetDisplayDevices")
; Monitor.RotateOneMonitor := Func("RotateOneMonitor")
; Monitor.RotateAllMonitor := Func("RotateAllMonitor")

; Monitor.RotateOneMonitor(90)
; MsgBox, % "len: " . GetMonitorCount()

; ; len := GetDisplayDevices().length


; MapByFunc(tab, func, params*) {
;     newtab := {}
;     for k, v in tab {
;         newtab[k] := %func%(v, params*)
;     }
;     return newtab
; }

; Merge(tab1, tab2) {
;     newtab := {}
;     for k, v in tab1 {
;         if (tab1[k] && tab2[k]) {
;             newtab.push([tab1[k], tab2[k]])
;         }
;     }
;     return newtab
; }

; GetDeviceMode(deviceName) {
;     devMode := new DEVMODE_DISPLAY_DEVICE()
;     success := DllCall("User32.dll\EnumDisplaySettingsA", "Str", deviceName, "Int", -1, "Ptr", devMode)
;     if (!success) {
;         MsgBox Can't get current settings.
;         return null
;     }
;     return devMode
; }

; GetDeviceName(device) {
;     return device.DeviceName
; }

; RotateDevMode90(devMode, devModeMain) {
;     New_Orientation := Mod(devMode.displayOrientation + 1, 4)

;     ; Calculate new positions
;     New_position_x := (devModeMain ? devModeMain.pelsHeight - devMode.position_y - devMode.pelsHeight : devMode.position_y)
;     New_position_y := devMode.position_x
;     New_pelsWidth := devMode.pelsHeight
;     New_pelsHeight := devMode.pelsWidth

;     devMode.position_x := New_position_x
;     devMode.position_y := New_position_y
;     devMode.pelsWidth := New_pelsWidth
;     devMode.pelsHeight := New_pelsHeight
;     devMode.displayOrientation := New_Orientation

;     return devMode
; }

; ApplyDevMode(deviceToDevmode) {
;     deviceName := deviceToDevmode[1]
;     devMode := deviceToDevmode[2]
;     success := DllCall("User32.dll\ChangeDisplaySettingsExA", "Str", deviceName, "Ptr", devMode, "Ptr", 0, "Int", 0, "Ptr", 0)
;     return success
; }

; GetDisplayDevices() {
;     lsDisplayDevice := []
;     iDevNum := 0
;     Loop {
;         lpDisplayDevice := new DISPLAY_DEVICE()
;         if !DllCall("User32.dll\EnumDisplayDevicesA", "ptr", 0, "Int", iDevNum, "Ptr", lpDisplayDevice, "Int", 0) {
;             break
;         }
;         if (lpDisplayDevice.StateFlags & _DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) {
;             lsDisplayDevice.Push(lpDisplayDevice)
;         }
;         MsgBox % "num" . iDevNum
;         iDevNum++
;     }
;     return lsDisplayDevice
; }

; RotateOneMonitor(degree, abs := false, deviceName := "") {
;     devMode := Monitor.GetDeviceMode(deviceName)
;     if !devMode {
;         return
;     }

;     rotCount := Mod((Floor(degree / 90)), 4)
;     if abs {
;         rotCount := Mod(rotCount - devMode.displayOrientation, 4)
;     }
;     Loop %rotCount% {
;         devMode := Monitor.RotateDevMode90(devMode)
;     }

;     success := Monitor.ApplyDevMode([deviceName, devMode])
;     return success
; }

; RotateAllMonitor(degree, abs := false) {
;     lsDevice := Monitor.GetDisplayDevices()
;     lsDeviceName := Monitor.MapByFunc(lsDevice, Monitor.GetDeviceName)

;     lsDevMode := Monitor.MapByFunc(lsDeviceName, Monitor.GetDeviceMode)
;     if !lsDevMode[1] {
;         MsgBox You can't rotate the `non-screen`
;         return
;     }

;     rotCount := Mod((Floor(degree / 90)), 4)
;     if abs {
;         rotCount := Mod(rotCount - lsDevMode[1].displayOrientation, 4)
;     }
;     Loop %rotCount% {
;         lsDevMode := Monitor.MapByFunc(lsDevMode, Monitor.RotateDevMode90, lsDevMode[1])
;     }

;     lsDeviceToDevmode := Monitor.Merge(lsDeviceName, lsDevMode)
;     lsSuccess := Monitor.MapByFunc(lsDeviceToDevmode, Monitor.ApplyDevMode)
;     return lsSuccess
; }

; ; return

; ; #^p:: ScreenRotate()
; ; ScreenRotate(){

; ;     ; Unit Test
; ;     Monitor.RotateOneMonitor(90)
; ;     ; Sleep, 3000
; ;     ; Monitor.RotateOneMonitor(0, true)
; ;     ; Sleep, 3000
; ;     ; Monitor.RotateAllMonitor(-90)
; ;     ; Sleep, 3000
; ;     ; Monitor.RotateAllMonitor(180)
; ;     ; Sleep, 3000
; ;     ; Monitor.RotateAllMonitor(90, true)
; ;     ; Sleep, 3000
; ;     ; Monitor.RotateAllMonitor(-90, true)
; ;     ; Sleep, 3000
; ;     ; Monitor.RotateAllMonitor(0, true)
; ;     ; Sleep, 3000
; ; }
