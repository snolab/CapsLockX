; ========== CapsLockX ==========
; 名称：编辑增强
; 描述：HJKL操作光标
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; 光标加速度微分对称模型（不要在意这中二的名字hhhh

if (!CapsLockX) {
    MsgBox, % "本模块只为 CapsLockX 工作"
    ExitApp
}
global 编辑增强_SpeedRatioX := CapsLockX_Config("EditEnhance", "SpeedRatioX", 1, "光标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")
global 编辑增强_SpeedRatioY := CapsLockX_Config("EditEnhance", "SpeedRatioY", 1, "光标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")
global 编辑增强_PageSpeed := CapsLockX_Config("EditEnhance", "PageSpeed", 1, "翻页速率")

global 方向键模拟 := new AccModel2D(Func("方向键模拟"), 0.1, 编辑增强_SpeedRatioX * 40, 编辑增强_SpeedRatioY * 20)
global 翻页键模拟 := new AccModel2D(Func("翻页键模拟"), 0.1, 20 * 编辑增强_PageSpeed)
方向键模拟.最大速度 := 250
翻页键模拟.最大速度 := 250

global 编辑增强_TurboTab := CapsLockX_Config("EditEnhance", "TurboTab", 0, "Tab键加速，可能和一些游戏不兼容，默认禁用")
if (编辑增强_TurboTab) {
    global TurboTab := new AccModel2D(Func("TurboTab"), 0.1, 10)
    TurboTab.最大速度 := 500
}

; Tab加速器 := new AccModel2D(1, 0, 0.01)
; Tab加速器.实动 := Func("Tab加速器")

CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))
; DisableLockWorkstation()
Return

DisableLockWorkstation()
{
    ; 这里改注册表是为了禁用 Win + L 锁定机器，让 Win+hjkl 可以挪窗口位置，不过只有用管理员运行才管用。（好像也不管用
    value := !!(ModuleState & MF_EditX) ? 0 : 1
    RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, %value%
}

左方向键发送(n:=1){
    loop %n%{
        SendEvent {Blind}{Left}
    }
}
右方向键发送(n:=1){
    loop %n%{
        SendEvent {Blind}{Right}
    }
}
上翻页键发送(n:=1){
    loop %n%{
        SendEvent {Blind}{PgUp}
    }
}
下翻页键发送(n:=1){
    loop %n%{
        SendEvent {Blind}{PgDn}
    }
}
上方向键发送(n:=1)
{
    ; 在 OneNote 笔记内部直接 SendEvent 上下方向键无反应， 故特殊处理使用 ControlSend 。
    if (hWnd := WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")) {
        ControlGetFocus, focusedClassNN, ahk_id %hWnd%
        if (focusedClassNN == "OneNote`:`:DocumentCanvas1") {
            loop %n%{
                ControlSend, OneNote::DocumentCanvas1, {Blind}{Up}
            }
        }
    }
    loop %n%{
        SendEvent {Blind}{up}
    }
}
下方向键发送(n:=1)
{
    ; 在 OneNote 笔记内部直接 SendEvent 上下方向键无反应， 故特殊处理使用 ControlSend 。
    if (hWnd := WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")) {
        ControlGetFocus, focusedClassNN, ahk_id %hWnd%
        if (focusedClassNN == "OneNote`:`:DocumentCanvas1") {
            loop %n%{
                ControlSend, OneNote::DocumentCanvas1, {Blind}{Down}
            }
            return
        }
    }
    loop %n%{
        SendEvent {Blind}{Down}
    }
}

翻页键模拟(dx, dy, 状态){
    if (!CapsLockXMode) {
        return 翻页键模拟.止动()
    }
    if (状态 != "移动"){
        return
    }
    _ := dy < 0 && 上翻页键发送(-dy)
    _ := dy > 0 && 下翻页键发送(dy )
}

TurboTab(dx, dy, 状态)
{
    ; _ := dy < 0 && 上翻页键发送(-dy)
    if (状态 != "移动"){
        return
    }
    loop %dy%{
        SendEvent {Blind}{Tab}
    }
}

方向键模拟(dx, dy, 状态)
{
    if (!CapsLockXMode) {
        return 方向键模拟.止动()
    }
    if (状态 == "横中键") {
        ; hl 选词
        ; 先按右再按左
        if (dx > 0) {
            SendEvent ^{Right}^+{Left}
        } else {
            SendEvent ^{Left}^+{Right}
        }
        return 方向键模拟.止动()
    }
    if (状态 == "纵中键") {
        ; jk 选句
        ; 先按下再按上
        if (dy > 0) {
            上方向键发送(1)
            SendEvent  {Home}+{End}
            ; SendEvent {End}+{Home}
        } else {
            下方向键发送(1)
            SendEvent  {Home}+{End}
        }
        return 方向键模拟.止动()
    }
    if (状态 != "移动"){
        return
    }
    ; tooltip % dy " " dx
    _ := dy < 0 && 上方向键发送(-dy)
    _ := dy > 0 && 下方向键发送(dy )
    _ := dx < 0 && 左方向键发送(-dx)
    _ := dx > 0 && 右方向键发送(dx )
}

#if 编辑增强_TurboTab

*Tab:: TurboTab.下按("Tab")

#if CapsLockXMode
    
*u:: 翻页键模拟.下按("u")
*i:: 翻页键模拟.上按("i")
*h:: 方向键模拟.左按("h")
*l:: 方向键模拟.右按("l")
*k:: 方向键模拟.上按("k")
*j:: 方向键模拟.下按("j")

*y:: Home
*o:: End
; 一起按相当于选择当前行，不同的顺序影响按完之后的光标位置（在前在后）
y & o:: Send {Home}+{End}
o & y:: Send {End}+{Home}

; 删除
*t:: Send {Blind}{Delete}
; *+t:: Send {Blind}{Shift Up}{BackSpace}{Shift Down}

; 回车
*g:: Enter
