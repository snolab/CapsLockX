; ========== CapsLockX ==========
; 名称：Blender Enhanced
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========
global D旋转 = [0,0,0]
global S缩放 := new AccModel2D(Func("S缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global S旋转 := new AccModel2D(Func("S旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global S视图 := new AccModel2D(Func("S视图"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X平移 := new AccModel2D(Func("X平移"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y平移 := new AccModel2D(Func("Y平移"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z平移 := new AccModel2D(Func("Z平移"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X缩放 := new AccModel2D(Func("X缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y缩放 := new AccModel2D(Func("Y缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z缩放 := new AccModel2D(Func("Z缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X旋转 := new AccModel2D(Func("X旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y旋转 := new AccModel2D(Func("Y旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z旋转 := new AccModel2D(Func("Z旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X平移值 := new AccModel2D(Func("X平移值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y平移值 := new AccModel2D(Func("Y平移值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z平移值 := new AccModel2D(Func("Z平移值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X缩放值 := new AccModel2D(Func("X缩放值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y缩放值 := new AccModel2D(Func("Y缩放值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z缩放值 := new AccModel2D(Func("Z缩放值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global X旋转值 := new AccModel2D(Func("X旋转值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Y旋转值 := new AccModel2D(Func("Y旋转值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Z旋转值 := new AccModel2D(Func("Z旋转值"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global Blender增强模式 := false

return

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

Blender数值输入(启动键, 数值, 状态, 止动键:="{Enter}``s")
{
    static 上次输入 := ""
    if (状态 == "启动" ){
        ; MouseMove, , Y [\, Speed, R]
        WinGetPos, X, Y, W, H, A
        X += W / 3 * 2
        Y += H / 4
        MouseMove, %X%, %Y%, 0
        SendEvent {Enter} ; 重置状态
        ; SendEvent %启动键%
        上次输入 := 0
        return
    }
    if (状态 == "止动" ){
        ; SendEvent %止动键%
        return
    }
    if (状态 != "移动"){
        return
    }
    输入数值 := 数值
    ; 回删次数 := StrLen(上次输入) - StrLen(输入数值)
    ; SendEvent {BackSpace %回删次数%}%输入数值%
    ; tooltip {BackSpace %回删次数%}%输入数值% %数值%

    SendEvent %启动键%%输入数值%{Enter}
    tooltip %启动键%%输入数值%{Enter}
    Sleep 10

    上次输入 := 输入数值
}
; 鼠标模拟
Blender视图运动(启动键, dx, dy, 状态, 止动键:="{Enter}``s")
{
    if (状态 == "启动" ){
        ; MouseMove, , Y [\, Speed, R]
        WinGetPos, X, Y, W, H, A
        X+=W/3*2
        Y+=H/4
        MouseMove, %X%, %Y%, 0
        SendEvent {Enter} ; 重置状态
        SendEvent %启动键%
        return
    }
    if (状态 =="止动" ){
        SendEvent %止动键%
        return
    }
    if  (状态!="移动"){
        return
    }
    ; tooltip %dx%
    BlenderEnhanced_SendInput_MouseMoveR32(dx, dy)
}
X平移( dx, _, 状态){
    Blender数值输入("gx", dx * 0.001, 状态)
}
Y平移( dx, _, 状态){
    Blender数值输入("gy", dx * 0.001, 状态)
}
Z平移( dx, _, 状态){
    Blender数值输入("gz", dx * 0.001, 状态)
}
X缩放( dx, _, 状态){
    Blender数值输入("sx", Exp(dx*0.001), 状态)
}
Y缩放( dx, _, 状态){
    Blender数值输入("sy", Exp(dx*0.001), 状态)
}
Z缩放( dx, _, 状态){
    Blender数值输入("sz", Exp(dx*0.001), 状态)
}
X旋转( dx, _, 状态){
    Blender数值输入("rx", dx * 0.360, 状态)
}
Y旋转( dx, _, 状态){
    Blender数值输入("ry", dx * 0.360, 状态)
}
Z旋转( dx, _, 状态){
    Blender数值输入("rz", dx * 0.360, 状态)
}
S缩放( dx, _, 状态){
    Blender数值输入("s", Exp(dx*0.001), 状态)
}
S旋转( dx, _, 状态){
    Blender数值输入("r", dx * 0.360, 状态)
}
S视图( dx, dy, 状态){
    Blender视图运动("{MButton Down}", dx, dy, 状态, "{MButton Up}``s")
    ; Blender数值输入("r", dx, dy, 状态)
}

Blender视图复位(){
    SendEvent ``v``s
}

Blender窗口内(){
    return WinActive("Blender ahk_class GHOST_WindowClass ahk_exe blender.exe")
}


#if Blender窗口内()

\::
    Blender增强模式 := !Blender增强模式
    tooltip %Blender增强模式%
    return
Blender增强模式(){
    return Blender窗口内() && Blender增强模式
}

#if Blender增强模式()

z:: SendEvent ``v``f
x:: SendEvent ``v``r
c:: SendEvent ``v``t
v:: Blender视图复位()

a::     X平移.左按() ; X平移
d::     X平移.右按() ; X平移
a Up::  X平移.左放()
d Up::  X平移.右放()
s::     Y平移.左按() ; Y平移
w::     Y平移.右按() ; Y平移
s Up::  Y平移.左放()
w Up::  Y平移.右放()
q::     Z平移.左按() ; Z平移
e::     Z平移.右按() ; Z平移
q Up::  Z平移.左放()
e Up::  Z平移.右放()

f::     X缩放.左按() ; X缩放
h::     X缩放.右按() ; X缩放
f Up::  X缩放.左放()
h Up::  X缩放.右放()
g::     Y缩放.左按() ; Y缩放
t::     Y缩放.右按() ; Y缩放
g Up::  Y缩放.左放()
t Up::  Y缩放.右放()
r::     Z缩放.左按() ; Z缩放
y::     Z缩放.右按() ; Z缩放
r Up::  Z缩放.左放()
y Up::  Z缩放.右放()

j::     X旋转.左按() ; X旋转
l::     X旋转.右按() ; X旋转
j Up::  X旋转.左放()
l Up::  X旋转.右放()
k::     Y旋转.左按() ; Y旋转
i::     Y旋转.右按() ; Y旋转
k Up::  Y旋转.左放()
i Up::  Y旋转.右放()
u::     Z旋转.左按() ; Z旋转
o::     Z旋转.右按() ; Z旋转
u Up::  Z旋转.左放()
o Up::  Z旋转.右放()

[::     S缩放.左按() ; S缩放
]::     S缩放.右按() ; S缩放
[ Up::  S缩放.左放()
] Up::  S缩放.右放()

<::     S旋转.左按() ; S旋转
>::     S旋转.右按() ; S旋转
< Up::  S旋转.左放()
> Up::  S旋转.右放()

