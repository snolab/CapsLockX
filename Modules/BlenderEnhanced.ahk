; ========== CapsLockX ==========
; 名称：Blender Enhanced
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========


; CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom("Modules/01.1-插件-鼠标模拟.md" ))
; global debug_fps := new FPS_Debugger()
global X平移 := new AccModel2D(Func("X平移"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y平移 := new AccModel2D(Func("Y平移"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z平移 := new AccModel2D(Func("Z平移"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X缩放 := new AccModel2D(Func("X缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y缩放 := new AccModel2D(Func("Y缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z缩放 := new AccModel2D(Func("Z缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X旋转 := new AccModel2D(Func("X旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y旋转 := new AccModel2D(Func("Y旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z旋转 := new AccModel2D(Func("Z旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global S缩放 := new AccModel2D(Func("S缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global S旋转 := new AccModel2D(Func("S旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)

BlenderEnhanced_SendInput_MouseMoveR32(x, y)
{
    VarSetCapacity(sendData, 28, 0)
    NumPut(0, sendData, 0, "UInt")
    NumPut(x, sendData, 4, "Int")
    NumPut(y, sendData, 8, "Int")
    NumPut(0, sendData, 12, "UInt")
    NumPut(1, sendData, 16, "UInt")
    DllCall("SendInput", "UInt", 1, "Str", sendData, "UInt", 28)
}


return

; 鼠标模拟
Blender运动(启动键, dx, dy, 状态)
{
    if (状态 == "启动" ){
        ; X平移.冲突止动()
        ; Y平移.冲突止动()
        ; Z平移.冲突止动()
        ; X缩放.冲突止动()
        ; Y缩放.冲突止动()
        ; Z缩放.冲突止动()
        ; X旋转.冲突止动()
        ; Y旋转.冲突止动()
        ; Z旋转.冲突止动()
        ; MouseMove, , Y [\, Speed, R]
        WinGetPos, X, Y, W, H, A
        X+=W/3*2
        Y+=H/4
        MouseMove, %X%, %Y%, 0
        SendEvent {Enter}
        SendEvent %启动键%
        return
    }
    if (状态 =="止动" ){
        SendEvent {Enter}``s
        return
    }
    if  (状态!="移动"){
        return
    }
    ; tooltip %dx%
    BlenderEnhanced_SendInput_MouseMoveR32(dx, dy)
}
X平移( dx, _, 状态){
    ; Blender运动("``tgx", dx, -dx, 状态)
    Blender运动("gx", dx, -dx, 状态)
}
Y平移( dx, _, 状态){
    ; Blender运动("``tgy", dx, -dx, 状态)
    Blender运动("gy", dx, -dx, 状态)
}
Z平移( dx, _, 状态){
    ; Blender运动("``fgz", dx, -dx, 状态)
    Blender运动("gz", dx, -dx, 状态)
}
X缩放( dx, _, 状态){
    ; Blender运动("``tsx", dx, -dx, 状态)
    Blender运动("sx", dx, -dx, 状态)
}
Y缩放( dx, _, 状态){
    ; Blender运动("``tsy", dx, -dx, 状态)
    Blender运动("sy", dx, -dx, 状态)
}
Z缩放( dx, _, 状态){
    ; Blender运动("``fsz", dx, -dx, 状态)
    Blender运动("sz", dx, -dx, 状态)
}
X旋转( dx, _, 状态){
    ; Blender运动("``rrx", dx, dx, 状态)
    Blender运动("rx", dx, dx, 状态)
}
Y旋转( dx, _, 状态){
    ; Blender运动("``fry", dx, dx, 状态)
    Blender运动("ry", dx, dx, 状态)
}
Z旋转( dx, _, 状态){
    ; Blender运动("``trz", dx, dx, 状态)
    Blender运动("rz", dx, dx, 状态)
}
S缩放( dx, _, 状态){
    ; Blender运动("``ss", dx, -dx, 状态)
    Blender运动("s", dx, -dx, 状态)
}
S旋转( dx, _, 状态){
    ; Blender运动("``sr", dx, dx, 状态)
    Blender运动("r", dx, dx, 状态)
}

Blender视图复位(){
    SendEvent ``v``s
}

Blender窗口内(){
    return WinActive("Blender ahk_class GHOST_WindowClass ahk_exe blender.exe")
}

#if Blender窗口内()

\ & z:: SendEvent ``v``f
\ & x:: SendEvent ``v``r
\ & c:: SendEvent ``v``t
\ & v:: Blender视图复位()

\ & a::     X平移.左按() ; X平移
\ & d::     X平移.右按() ; X平移
\ & a Up::  X平移.左放()
\ & d Up::  X平移.右放()
\ & s::     Y平移.左按() ; Y平移
\ & w::     Y平移.右按() ; Y平移
\ & s Up::  Y平移.左放()
\ & w Up::  Y平移.右放()
\ & q::     Z平移.左按() ; Z平移
\ & e::     Z平移.右按() ; Z平移
\ & q Up::  Z平移.左放()
\ & e Up::  Z平移.右放()

\ & f::     X缩放.左按() ; X缩放
\ & h::     X缩放.右按() ; X缩放
\ & f Up::  X缩放.左放()
\ & h Up::  X缩放.右放()
\ & g::     Y缩放.左按() ; Y缩放
\ & t::     Y缩放.右按() ; Y缩放
\ & g Up::  Y缩放.左放()
\ & t Up::  Y缩放.右放()
\ & r::     Z缩放.左按() ; Z缩放
\ & y::     Z缩放.右按() ; Z缩放
\ & r Up::  Z缩放.左放()
\ & y Up::  Z缩放.右放()

\ & j::     X旋转.左按() ; X旋转
\ & l::     X旋转.右按() ; X旋转
\ & j Up::  X旋转.左放()
\ & l Up::  X旋转.右放()
\ & k::     Y旋转.左按() ; Y旋转
\ & i::     Y旋转.右按() ; Y旋转
\ & k Up::  Y旋转.左放()
\ & i Up::  Y旋转.右放()
\ & u::     Z旋转.左按() ; Z旋转
\ & o::     Z旋转.右按() ; Z旋转
\ & u Up::  Z旋转.左放()
\ & o Up::  Z旋转.右放()

\ & [::     S缩放.左按() ; S缩放
\ & ]::     S缩放.右按() ; S缩放
\ & [ Up::  S缩放.左放()
\ & ] Up::  S缩放.右放()

\ & <::     S旋转.左按() ; S旋转
\ & >::     S旋转.右按() ; S旋转
\ & < Up::  S旋转.左放()
\ & > Up::  S旋转.右放()

