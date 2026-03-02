; ========== CapsLockX ==========
; 名称：编辑增强
; 描述：YUIO HJKL 操作光标 GT 回车删除
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2024 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
; 光标加速度微分对称模型（不要在意这中二的名字hhhh

; will include once
#Include, Modules/AccModel/AccModel.ahk

if (!CapsLockX) {
    MsgBox, % "本模块只为 CapsLockX 工作"
    ExitApp
}
; In --no-core mode Rust owns HJKL/YUIO cursor; skip this module entirely.
if (CLX_NoCore)
    return

global CLX_HJKL_Scroll := CLX_Config("TMouse", "CLX_HJKL_Scroll", 0, t("使用IJKL滚轮移动滚轮，比RF多一个横轴。"))
global 编辑增强_SpeedRatioX := CLX_Config("EditEnhance", "SpeedRatioX", 1, t("HJKL光标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类"))
global 编辑增强_SpeedRatioY := CLX_Config("EditEnhance", "SpeedRatioY", 1, t("HJKL光标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类"))
global 编辑增强_PageSpeed := CLX_Config("EditEnhance", "PageSpeed", 1, t("U:PageDown I:PageUP 翻页速率"))
global 编辑增强_TabSpeed := CLX_Config("EditEnhance", "TabSpeed", 1, t("Tab速率"))
global 方向键模拟 := new AccModel2D(Func("方向键模拟"), 0.1, 编辑增强_SpeedRatioX * 15, 编辑增强_SpeedRatioY * 15)
global 翻页键模拟 := new AccModel2D(Func("翻页键模拟"), 0.1, 20 * 编辑增强_PageSpeed)
global Tab键模拟 := new AccModel2D(Func("Tab键模拟"), 0.1, 15 * 编辑增强_TabSpeed)
方向键模拟.最大速度 := 250
翻页键模拟.最大速度 := 250
Tab键模拟.最大速度 := 250

CLX_AppendHelp( CLX_LoadHelpFrom(CLX_THIS_MODULE_HELP_FILE_PATH))
; DisableLockWorkstation()
Return

DisableLockWorkstation()
{
    ; 这里改注册表是为了禁用 Win + L 锁定机器，让 Win+hjkl 可以挪窗口位置，不过只有用管理员运行才管用。（好像也不管用
    value := !!(ModuleState & MF_EditX) ? 0 : 1
    RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, %value%
}

; 这里用 SendEvent 防止把 hl 按出来
左方向键发送(n:=1){
    loop %n%{
        if (A_Index > 128) {
            return
        }
        SendEvent {Blind}{Left}
    }
}
右方向键发送(n:=1){
    loop %n%{
        if (A_Index > 128) {
            return
        }
        SendEvent {Blind}{Right}
    }
}
上翻页键发送(n:=1){
    loop %n%{
        if (A_Index > 128) {
            return
        }
        SendEvent {Blind}{PgUp}
    }
}
下翻页键发送(n:=1){
    loop %n%{
        if (A_Index > 128) {
            return
        }
        SendEvent {Blind}{PgDn}
    }
}
左翻页键发送(n:=1){
    loop %n%{
        if (A_Index > 128) {
            return
        }
        SendEvent {Blind}{Home}
    }
}
右翻页键发送(n:=1){
    loop %n%{
        if (A_Index > 128) {
            return
        }
        SendEvent {Blind}{End}
    }
}
正Tab键发送(n:=1){
    loop %n%{
        if (A_Index > 32) {
            return
        }
        SendEvent {Blind}{Tab}
    }
}
反Tab键发送(n:=1){
    loop %n%{
        if (A_Index > 32) {
            return
        }
        SendEvent {Blind}+{Tab}
    }
}
上方向键发送(n:=1)
{
    ; 在 OneNote 笔记内部直接 Send 上下方向键无反应， 故特殊处理使用 ControlSend 。
    if (hWnd := WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")) {
        ControlGetFocus, focusedClassNN, ahk_id %hWnd%
        if (focusedClassNN == "OneNote`:`:DocumentCanvas1") {
            loop %n%{
                if (A_Index > 128) {
                    return
                }
                ControlSend, OneNote::DocumentCanvas1, {Blind}{Up}
            }
        }
    }
    loop %n%{
        if (A_Index > 128) {
            return
        }
        ; 这里如果使用 SendInput 则会出现打出kj的情况。
        SendEvent {Blind}{up}
    }
}
下方向键发送(n:=1)
{
    ; 在 OneNote 笔记内部直接 Send 上下方向键无反应， 故特殊处理使用 ControlSend 。
    if (hWnd := WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")) {
        ControlGetFocus, focusedClassNN, ahk_id %hWnd%
        if (focusedClassNN == "OneNote`:`:DocumentCanvas1") {
            loop %n%{
                if (A_Index > 128) {
                    return
                }
                ControlSend, OneNote::DocumentCanvas1, {Blind}{Down}
            }
            return
        }
    }
    loop %n%{
        if (A_Index > 128) {
            return
        }
        ; 这里如果使用 SendInput 则会出现打出kj的情况。
        SendEvent {Blind}{Down}
    }
}

翻页键模拟(dx, dy, 状态){
    if (!CapsLockXMode) {
        return 翻页键模拟.止动()
    }
    if (状态 != "移动") {
        return
    }
    if (状态 == "纵中键") {
        return 翻页键模拟.止动()
    }
    if (状态 == "横中键") {
        if (dx > 0) {
            Send {End}+{Home}
        } else {
            Send {Home}+{End}
        }
        return 翻页键模拟.止动()
    }
    _ := dy < 0 && 上翻页键发送(-dy)
    _ := dy > 0 && 下翻页键发送(dy )
    _ := dx < 0 && 左翻页键发送(-dx)
    _ := dx > 0 && 右翻页键发送(dx )
}

Tab键模拟(dx, dy, 状态){
    if (!CapsLockXMode) {
        return 翻页键模拟.止动()
    }
    if (状态 != "移动") {
        return
    }
    ; reverse by shift key pressed
    sdy := GetKeyState("Shift", "P") ? -dy : dy
    ;
    _ := sdy < 0 && 反Tab键发送(-sdy)
    _ := sdy > 0 && 正Tab键发送(sdy )
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
            Send ^{Right}^+{Left}
        } else {
            Send ^{Left}^+{Right}
        }
        return 方向键模拟.止动()
    }
    if (状态 == "纵中键") {
        ; jk 选句
        ; 先按下再按上
        if (dy > 0) {
            上方向键发送(1)
            Send {Home}+{End}
            ; Send {End}+{Home}
        } else {
            下方向键发送(1)
            Send {Home}+{End}
        }
        return 方向键模拟.止动()
    }
    if (状态 != "移动") {
        return
    }
    ; tooltip % dy " " dx
    _ := dy < 0 && 上方向键发送(-dy)
    _ := dy > 0 && 下方向键发送(dy )
    _ := dx < 0 && 左方向键发送(-dx)
    _ := dx > 0 && 右方向键发送(dx )
}

#if CapsLockXMode

*t:: Delete
*g:: Enter
; *[:: Tab键模拟.上按("[")
; *]:: Tab键模拟.下按("]")
*p:: Tab键模拟.上按("p")
*n:: Tab键模拟.下按("n")

#if CapsLockXMode && !CLX_HJKL_Scroll

*i:: 翻页键模拟.上按("i")
*u:: 翻页键模拟.下按("u")
*y:: 翻页键模拟.左按("y")
*o:: 翻页键模拟.右按("o")
*h:: 方向键模拟.左按("h")
*l:: 方向键模拟.右按("l")
*k:: 方向键模拟.上按("k")
*j:: 方向键模拟.下按("j")
