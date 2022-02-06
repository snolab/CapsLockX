; ========== CapsLockX ==========
; 名称：Blender Enhanced
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v0.0.1
; ========== CapsLockX ==========

BlenderEnhancedInit()

global Blender物件调整 := Blender物件调整初始值获取()
Blender物件调整初始值获取(){
    return [[[0, 0, 0], [0, 0, 0], [0, 0, 0]], [[0, 0, 0], [0, 0, 0], [0, 0, 0]]]
}
global 平移精度 := 0.1
global 平移精度控制 := new AccModel2D(Func("平移精度控制"), 0.1, 10)
global S缩放 := new AccModel2D(Func("S缩放"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global S旋转 := new AccModel2D(Func("S旋转"), 0.1, TMouse_DPIRatio * 360 * TMouse_MouseSpeedRatio)
global 平移导航 := new AccModel2D(Func("平移导航"), 0.1, TMouse_DPIRatio * 120 * TMouse_MouseSpeedRatio)
global 旋转导航 := new AccModel2D(Func("旋转导航"), 0.1, TMouse_DPIRatio * 120 * TMouse_MouseSpeedRatio)
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
Blender视图运动(启动键, dx, dy, 状态)
{
    if (状态 == "启动" ) {
        WinGetPos, X, Y, W, H, A
        X+=W/3*2, Y+=H/4
        MouseMove, %X%, %Y%, 0
        SendEvent {Enter}%启动键%
        return
    }
    if (状态 == "止动" ) {
        SendEvent {Enter}``s
        return
    }
    if (状态 != "移动") {
        return
    }
    MouseMove, %X%, %Y%, 0
    
    BlenderEnhanced_SendInput_MouseMoveR32(dx, dy)
}

平移导航(dx, dy, 状态){
    if (状态=="启动") {
        Send {ShiftDown}{MButton Down}
        return
    }
    if (状态=="止动") {
        Send {MButton Up}{ShiftUp}
    }
    if (状态!=="移动") {
        return
    }
    BlenderEnhanced_SendInput_MouseMoveR32(-dx, -dy)
}
旋转导航(dx, dy, 状态){
    if (状态=="启动") {
        Send {MButton Down}
        return
    }
    if (状态=="止动") {
        Send {MButton Up}
    }
    if (状态!=="移动") {
        return
    }
    BlenderEnhanced_SendInput_MouseMoveR32(-dx, -dy)
}

Blender物件数值调整(lg, operation, dimention, delta){
    global Blender物件调整中
    Blender物件调整[lg][operation][dimention] += delta
    if (!Blender物件调整中) {
        Blender物件调整中 := 1
        SetTimer Blender物件调整, -1
    }
}
Blender物件调整(){
    static t0 := S缩放._QPS()
    static rdt := 0.0
    t := S缩放._QPS()
    dt := t - t0
    rdt := (rdt * 5 + dt * 1 ) /6
    t0 := t
    global msg := ""
    msg .= 平移精度 "`n"
    msg .= rdt "`t" dt "`n"
    msg .= Blender物件调整[1][1][1] "`t" Blender物件调整[1][1][2] "`t" Blender物件调整[1][1][3] "`n"
    msg .= Blender物件调整[1][2][1] "`t" Blender物件调整[1][2][2] "`t" Blender物件调整[1][2][3] "`n"
    msg .= Blender物件调整[1][3][1] "`t" Blender物件调整[1][3][2] "`t" Blender物件调整[1][3][3] "`n"
    msg .= Blender物件调整[2][1][1] "`t" Blender物件调整[2][1][2] "`t" Blender物件调整[2][1][3] "`n"
    msg .= Blender物件调整[2][2][1] "`t" Blender物件调整[2][2][2] "`t" Blender物件调整[2][2][3] "`n"
    msg .= Blender物件调整[2][3][1] "`t" Blender物件调整[2][3][2] "`t" Blender物件调整[2][3][3] "`n"
    /*
    amap=(a, f)=>a.map(f).join('\n');
    smap=(s, f)=>s.split('').map(f).join('\n');
    xyz='XYZ';
    gsr=['平移', '缩放', '旋转'];
    lg=['本地', '全局'];
    `    _ := (Blender物件调整[1][1][1]!=0) && Blender数值输入("${gsr[gi]+xyz[xi]+(li?xyz[xi]:'')}", (Blender物件调整[${li+1}][1][1]*平移精度), "移动")`
    =
    */
    _ := (Blender物件调整[1][1][1]!=0) && Blender数值输入("gxx", (Blender物件调整[1][1][1]*平移精度), "移动")
    _ := (Blender物件调整[1][1][2]!=0) && Blender数值输入("gyy", (Blender物件调整[1][1][2]*平移精度), "移动")
    _ := (Blender物件调整[1][1][3]!=0) && Blender数值输入("gzz", (Blender物件调整[1][1][3]*平移精度), "移动")
    _ := (Blender物件调整[1][2][1]!=0) && Blender数值输入("sxx", (exp(Blender物件调整[1][2][1]*0.01)), "移动")
    _ := (Blender物件调整[1][2][2]!=0) && Blender数值输入("syy", (exp(Blender物件调整[1][2][2]*0.01)), "移动")
    _ := (Blender物件调整[1][2][3]!=0) && Blender数值输入("szz", (exp(Blender物件调整[1][2][3]*0.01)), "移动")
    _ := (Blender物件调整[1][3][1]!=0) && Blender数值输入("rxx", (Blender物件调整[1][3][1]*180/180), "移动")
    _ := (Blender物件调整[1][3][2]!=0) && Blender数值输入("ryy", (Blender物件调整[1][3][2]*180/180), "移动")
    _ := (Blender物件调整[1][3][3]!=0) && Blender数值输入("rzz", (Blender物件调整[1][3][3]*180/180), "移动")
    _ := (Blender物件调整[2][1][1]!=0) && Blender数值输入("gx", (Blender物件调整[2][1][1]*平移精度), "移动")
    _ := (Blender物件调整[2][1][2]!=0) && Blender数值输入("gy", (Blender物件调整[2][1][2]*平移精度), "移动")
    _ := (Blender物件调整[2][1][3]!=0) && Blender数值输入("gz", (Blender物件调整[2][1][3]*平移精度), "移动")
    _ := (Blender物件调整[2][2][1]!=0) && Blender数值输入("sx", (exp(Blender物件调整[2][2][1]*0.01)), "移动")
    _ := (Blender物件调整[2][2][2]!=0) && Blender数值输入("sy", (exp(Blender物件调整[2][2][2]*0.01)), "移动")
    _ := (Blender物件调整[2][2][3]!=0) && Blender数值输入("sz", (exp(Blender物件调整[2][2][3]*0.01)), "移动")
    _ := (Blender物件调整[2][3][1]!=0) && Blender数值输入("rx", (Blender物件调整[2][3][1]*180/180), "移动")
    _ := (Blender物件调整[2][3][2]!=0) && Blender数值输入("ry", (Blender物件调整[2][3][2]*180/180), "移动")
    _ := (Blender物件调整[2][3][3]!=0) && Blender数值输入("rz", (Blender物件调整[2][3][3]*180/180), "移动")
    Blender物件调整 := Blender物件调整初始值获取()
    SetTimer Blender物件调整, Off
    global Blender物件调整中 := 0
    tooltip %msg%
    
    ; Sleep 16 ; a frame
}
Blender数值输入(启动键, 数值, 状态){
    if (状态 == "启动") {
        SendEvent {Enter}
        return
    }
    if (状态 == "止动") {
        SendEvent {Enter}``s
        return
    }
    if (状态 != "移动") {
        return
    }
    
    ; 数值 := Round(数值, Log(平移精度)/Log(10))
    if(数值 != 0) {
        SendEvent %启动键%%数值%{Enter}
    }
    global msg
    msgp = %启动键%%数值%{Enter}
    msg .= msgp "`n"
}

S缩放( dx, _, 状态){
    Blender视图运动("s", dx, -dx, 状态)
}
S旋转( dx, _, 状态){
    Blender视图运动("r", dx, dx, 状态)
}
Blender视图复位(){
    SendEvent ``v``s
}

#if Blender窗口内()

Blender窗口内(){
    return WinActive("Blender ahk_class GHOST_WindowClass ahk_exe blender.exe")
}
\::
    global Blender增强模式 := !Blender增强模式
    tooltip Blender增强模式 %Blender增强模式%
return
\ & a:: 平移导航.左按("a")
\ & d:: 平移导航.右按("d")
\ & w:: 平移导航.上按("w")
\ & s:: 平移导航.下按("s")
\ & q:: 旋转导航.左按("q")
\ & e:: 旋转导航.右按("e")
\ & r:: 旋转导航.上按("r")
\ & f:: 旋转导航.下按("f")

#if Blender窗口内() && Blender增强模式

j & r:: 平移精度控制.上按("r")
u & r:: 平移精度控制.上按("r")
j & f:: 平移精度控制.下按("f")
u & f:: 平移精度控制.下按("f")

平移精度控制(_, dy, 状态){
    if (状态 != "移动") {
        return
    }
    平移精度 *= 10 ** -dy
    tooltip 平移精度 %平移精度%
}

; 操作设计：
; 2x3x3
; 全局平移缩放旋转 uio
; 本地平移缩放旋转 jkl
; 3轴数值运动：adswqe

/* 注：下方代码通过注释内 js 生成
amap=(a, f)=>a.map(f).join('\n');
smap=(s, f)=>s.split('').map(f).join('\n');
xyz='XYZ';
gsr=['平移', '缩放', '旋转'];
lg=['本地', '全局'];

re = '\n'

re +=
smap("jkluio", (os, oi)=>
smap("adswqe", (ds, di)=>
`
${os} & ${ds}:: ${xyz[di/2|0]+lg[oi/3|0]+gsr[oi%3]}.${'左右'[di%2]}按("${ds}") ; ${xyz[di/2|0]+lg[oi/3|0]+gsr[oi%3]+'+-'[di%2]}
`.trim()
))+'\n'

re +=
`BlenderEnhancedInit(){\n`+
smap('lg', (ls, li)=>
amap(gsr, (os, oi)=>
smap(xyz, (ds, di)=>
`    global ${xyz[di]+lg[li]+gsr[oi]} := new AccModel2D(Func("${xyz[di]+lg[li]+gsr[oi]}"), 0.1, 50)`
)))+
'\n}\n'

re +=
smap('lg', (ls, li)=>
amap(gsr, (os, oi)=>
smap(xyz, (ds, di)=>
`${xyz[di]+lg[li]+gsr[oi]}(dx, _, 状态){
Blender物件数值调整(${li+1}, ${oi+1}, ${di+1}, dx)
}`
)))+'\n'

re
=
*/
j & a:: X本地平移.左按("a") ; X本地平移+
j & d:: X本地平移.右按("d") ; X本地平移-
j & s:: Y本地平移.左按("s") ; Y本地平移+
j & w:: Y本地平移.右按("w") ; Y本地平移-
j & q:: Z本地平移.左按("q") ; Z本地平移+
j & e:: Z本地平移.右按("e") ; Z本地平移-
k & a:: X本地缩放.左按("a") ; X本地缩放+
k & d:: X本地缩放.右按("d") ; X本地缩放-
k & s:: Y本地缩放.左按("s") ; Y本地缩放+
k & w:: Y本地缩放.右按("w") ; Y本地缩放-
k & q:: Z本地缩放.左按("q") ; Z本地缩放+
k & e:: Z本地缩放.右按("e") ; Z本地缩放-
l & a:: X本地旋转.左按("a") ; X本地旋转+
l & d:: X本地旋转.右按("d") ; X本地旋转-
l & s:: Y本地旋转.左按("s") ; Y本地旋转+
l & w:: Y本地旋转.右按("w") ; Y本地旋转-
l & q:: Z本地旋转.左按("q") ; Z本地旋转+
l & e:: Z本地旋转.右按("e") ; Z本地旋转-
u & a:: X全局平移.左按("a") ; X全局平移+
u & d:: X全局平移.右按("d") ; X全局平移-
u & s:: Y全局平移.左按("s") ; Y全局平移+
u & w:: Y全局平移.右按("w") ; Y全局平移-
u & q:: Z全局平移.左按("q") ; Z全局平移+
u & e:: Z全局平移.右按("e") ; Z全局平移-
i & a:: X全局缩放.左按("a") ; X全局缩放+
i & d:: X全局缩放.右按("d") ; X全局缩放-
i & s:: Y全局缩放.左按("s") ; Y全局缩放+
i & w:: Y全局缩放.右按("w") ; Y全局缩放-
i & q:: Z全局缩放.左按("q") ; Z全局缩放+
i & e:: Z全局缩放.右按("e") ; Z全局缩放-
o & a:: X全局旋转.左按("a") ; X全局旋转+
o & d:: X全局旋转.右按("d") ; X全局旋转-
o & s:: Y全局旋转.左按("s") ; Y全局旋转+
o & w:: Y全局旋转.右按("w") ; Y全局旋转-
o & q:: Z全局旋转.左按("q") ; Z全局旋转+
o & e:: Z全局旋转.右按("e") ; Z全局旋转-
BlenderEnhancedInit()
{
    global X本地平移 := new AccModel2D(Func("X本地平移"), 0.1, 50)
    global Y本地平移 := new AccModel2D(Func("Y本地平移"), 0.1, 50)
    global Z本地平移 := new AccModel2D(Func("Z本地平移"), 0.1, 50)
    global X本地缩放 := new AccModel2D(Func("X本地缩放"), 0.1, 50)
    global Y本地缩放 := new AccModel2D(Func("Y本地缩放"), 0.1, 50)
    global Z本地缩放 := new AccModel2D(Func("Z本地缩放"), 0.1, 50)
    global X本地旋转 := new AccModel2D(Func("X本地旋转"), 0.1, 50)
    global Y本地旋转 := new AccModel2D(Func("Y本地旋转"), 0.1, 50)
    global Z本地旋转 := new AccModel2D(Func("Z本地旋转"), 0.1, 50)
    global X全局平移 := new AccModel2D(Func("X全局平移"), 0.1, 50)
    global Y全局平移 := new AccModel2D(Func("Y全局平移"), 0.1, 50)
    global Z全局平移 := new AccModel2D(Func("Z全局平移"), 0.1, 50)
    global X全局缩放 := new AccModel2D(Func("X全局缩放"), 0.1, 50)
    global Y全局缩放 := new AccModel2D(Func("Y全局缩放"), 0.1, 50)
    global Z全局缩放 := new AccModel2D(Func("Z全局缩放"), 0.1, 50)
    global X全局旋转 := new AccModel2D(Func("X全局旋转"), 0.1, 50)
    global Y全局旋转 := new AccModel2D(Func("Y全局旋转"), 0.1, 50)
    global Z全局旋转 := new AccModel2D(Func("Z全局旋转"), 0.1, 50)
}
X本地平移(dx, _, 状态){
    Blender物件数值调整(1, 1, 1, dx)
}
Y本地平移(dx, _, 状态){
    Blender物件数值调整(1, 1, 2, dx)
}
Z本地平移(dx, _, 状态){
    Blender物件数值调整(1, 1, 3, dx)
}
X本地缩放(dx, _, 状态){
    Blender物件数值调整(1, 2, 1, dx)
}
Y本地缩放(dx, _, 状态){
    Blender物件数值调整(1, 2, 2, dx)
}
Z本地缩放(dx, _, 状态){
    Blender物件数值调整(1, 2, 3, dx)
}
X本地旋转(dx, _, 状态){
    Blender物件数值调整(1, 3, 1, dx)
}
Y本地旋转(dx, _, 状态){
    Blender物件数值调整(1, 3, 2, dx)
}
Z本地旋转(dx, _, 状态){
    Blender物件数值调整(1, 3, 3, dx)
}
X全局平移(dx, _, 状态){
    Blender物件数值调整(2, 1, 1, dx)
}
Y全局平移(dx, _, 状态){
    Blender物件数值调整(2, 1, 2, dx)
}
Z全局平移(dx, _, 状态){
    Blender物件数值调整(2, 1, 3, dx)
}
X全局缩放(dx, _, 状态){
    Blender物件数值调整(2, 2, 1, dx)
}
Y全局缩放(dx, _, 状态){
    Blender物件数值调整(2, 2, 2, dx)
}
Z全局缩放(dx, _, 状态){
    Blender物件数值调整(2, 2, 3, dx)
}
X全局旋转(dx, _, 状态){
    Blender物件数值调整(2, 3, 1, dx)
}
Y全局旋转(dx, _, 状态){
    Blender物件数值调整(2, 3, 2, dx)
}
Z全局旋转(dx, _, 状态){
    Blender物件数值调整(2, 3, 3, dx)
}
