
IUIAutomation := ComObjCreate(CLSID_CUIAutomation := "{ff48dba4-60ef-4201-aa87-54103eef594e}", IID_IUIAutomation := "{30cbe57d-d9d0-452a-ab13-7ac5ac4825ee}")
if (!IUIAutomation) {
    ; SNOCLICK 不可用
    return
}
global FLAG_SNOCLICK := false
return
; SNOCLICK()
; DllCall(NumGet(NumGet(IUIAutomation+0)+6*A_PtrSize), "ptr", IUIAutomation, "ptr", WinExist("A"), "ptr*", ElementFromHandle)   ; IUIAutomation::ElementFromHandle
; TODO SCAN SCREEN

SNOCLICK(dx, dy)
{
    ; static stepDeep := 1
    ; if (!(dx || dy)) {
    ; stepDeep := 1
    ; FLAG_SNOCLICK := !FLAG_SNOCLICK
    ; return
    ; }
    ; Screen / stepDeep ** 2
    ; stepDeep++
    /*
    确保不在 SNOCLICK 模式
    屏幕截图
    CUDA   边缘检测 可点击区域计算
    显示一个键盘热键点击界面
    */
    ; traytip SNOCLICK 雪星之触, 功能开发中，敬请期待
    ; TrayTip test, test
    ; tooltip test
}
; #if FLAG_SNOCLICK

; a:: SNOCLICK(-1, 0)
; d:: SNOCLICK( 1, 0)
; w:: SNOCLICK(0, -1)
; s:: SNOCLICK(0, 1)
; g:: SNOCLICK(0, 0)
