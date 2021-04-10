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
global 方动中 := 0
global 方刻左 := 0, 方刻右 := 0, 方刻上 := 0, 方刻下 := 0
global 方速横 := 0, 方速纵 := 0, 方位横 := 0, 方位纵 := 0

global 编辑增强_SpeedRatioX := CapsLockX_Config("EditEnhance", "SpeedRatioX", 1, "光标标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")
global 编辑增强_SpeedRatioY := CapsLockX_Config("EditEnhance", "SpeedRatioY", 1, "光标标加速度比率, 默认为 1, 你想慢点就改成 0.5 之类")

CapsLockX_AppendHelp("
(
编辑增强
| 全局 | CapsLockX + k j h l | 上下左右 方向键 |
| 全局 | CapsLockX + hl | hl 一起按选择当前词 |
| 全局 | CapsLockX + y o | Home End |
| 全局 | CapsLockX + yo | yo 一起按选择当前行 |
| 全局 | CapsLockX + g | 回车 |
| 全局 | CapsLockX + t | BackSpace |
| 全局 | CapsLockX + Shift + t | Delete |
)")

Return

OnSwitch(){
    ; 这里改注册表是为了禁用 Win + L 锁定机器，让 Win+hjkl 可以挪窗口位置，不过只有用管理员运行才管用。
    value := !!(ModuleState & MF_EditX) ? 0 : 1
    RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, %value%
}

SendArrowUp(){
    ; 在 OneNote 笔记内部直接 SendEvent 上下方向键无反应， 故使用 ControlSend 。
    if (hWnd := WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")) {
        ControlGetFocus, focusedClassNN, ahk_id %hWnd%
        if (focusedClassNN == "OneNote`:`:DocumentCanvas1") {
            ControlSend, OneNote::DocumentCanvas1, {Blind}{Up}
            return
        }
    }
    SendEvent {Blind}{up}
}
SendArrowDown(){
    ; 在 OneNote 笔记内部直接 SendEvent 上下方向键无反应， 故使用 ControlSend 。
    if (hWnd := WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE")) {
        ControlGetFocus, focusedClassNN, ahk_id %hWnd%
        if (focusedClassNN == "OneNote`:`:DocumentCanvas1") {
            ControlSend, OneNote::DocumentCanvas1, {Blind}{Down}
            return
        }
    }
    SendEvent {Blind}{down}
}

ArrowTicker(){
    ; 在非 CapsLockX 模式下直接停止
    if (!(CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN)){
        ArrowTickerStop()
        Return
    }
    ; else{
    tNow := TM_QPC()
    ; 计算用户操作时间
    tda := dt(方刻左, tNow), tdd := dt(方刻右, tNow)
    tdw := dt(方刻上, tNow), tds := dt(方刻下, tNow)
    ; 计算加速度
    kax := ma(tdd - tda)*编辑增强_SpeedRatioX, kay := ma(tds - tdw)*编辑增强_SpeedRatioY

    ; 摩擦力不阻碍用户意志
    方速横 := Friction(方速横 + kax, kax), 方速纵 := Friction(方速纵 + kay, kay)

    ; 稳定化
    方位横 += 方速横 / 200, 方位纵 += 方速纵 / 200
    ; ToolTip, %方刻左% _ %方刻右% _ %方刻上% _ %方刻下% `n %kax% _ %kay% _ %方速横% _ %方速纵% _ %方位横% _ %方位纵%
    ; msgbox % 方位横
    ; 完成移动时
    if ( 0 == 方速横 && 0 == 方速纵){
        ArrowTickerStop()
        Return
    }
    ; ToolTip, % 方位横 " " 方位纵
    ; TODO: 输出速度时间曲线，用于DEBUG
    if (方位横 >= 1){
        Loop %方位横% {
            SendEvent {Blind}{Right}
        }
        方位横 -= 方位横 | 0
    }
    if (方位横 <= -1){
        方位横 := -方位横
        Loop %方位横% {
            SendEvent {Blind}{Left}
        }
        方位横 := -方位横
        方位横 -= 方位横 | 0
    }
    if (方位纵 >= 1){
        Loop %方位纵% {
            SendArrowDown()
        }
        方位纵 -= 方位纵 | 0
    }
    if (方位纵 <= -1){
        方位纵 := -方位纵
        Loop %方位纵% {
            SendArrowUp()
        }
        方位纵 := -方位纵
        方位纵 -= 方位纵 | 0
    }
}

; 时间处理
ArrowTickerStart(){
    方动中 := 1
    SetTimer, ArrowTicker, 1
}
ArrowTickerStop(){
    ; 重置相关参数
    方动中 := 0, 方刻左 := 0, 方刻右 := 0, 方刻上 := 0, 方刻下 := 0, 方速横 := 0, 方速纵 := 0, 方位横 := 0, 方位纵 := 0
    SetTimer, ArrowTicker, Off
}

#if CapsLockXMode

; 上下左右
; 光标运动处理
ArrowLeftPressed(){
    if (方刻左)
        Return
    if (方刻右){
        ; 选中当前词
        SendEvent ^{Right}^+{Left}
        方刻右 := 0
        Return
    }
    ArrowTickerStart()
    方刻左 := TM_QPC()
    SendEvent {Blind}{Left}
}
ArrowRightPressed(){
    if (方刻右)
        Return
    if (方刻左){
        ; 选中当前词
        SendEvent ^{Left}^+{Right}
        方刻左 := 0
        Return
    }
    方刻右 := TM_QPC()
    SendEvent {Blind}{Right}
    ArrowTickerStart()
}
ArrowUpPressed(){
    if (方刻上) 
        Return
    if (方刻下){
        ; KJ一起按选择当前行
        SendArrowUp()
        SendEvent {Home}+{End}
        方刻下 := 0
        Return
    }
    方刻上 := TM_QPC()
    SendArrowUp()
    ArrowTickerStart()
}
ArrowDownPressed(){
    if (方刻下)
        Return
    if (方刻上){
        ; KJ一起按选择当前行
        SendArrowDown()
        SendEvent {End}+{Home}
        方刻上 := 0
        Return
    }
    方刻下 := TM_QPC()
    SendArrowDown()
    ArrowTickerStart()
}

*u:: PgDn
*i:: PgUp
;
*h:: ArrowLeftPressed()
*l:: ArrowRightPressed()
*k:: ArrowUpPressed()
*j:: ArrowDownPressed()
*h Up:: 方刻左 := 0, ArrowTickerStart()
*l Up:: 方刻右 := 0, ArrowTickerStart()
*k Up:: 方刻上 := 0, ArrowTickerStart()
*j Up:: 方刻下 := 0, ArrowTickerStart()

*y:: Home
*o:: End
; 一起按相当于选择当前行，不同的顺序影响按完之后的光标位置（在前在后）
y & o:: Send {Home}+{End}
o & y:: Send {End}+{Home}

; 前删，后删
*t:: Send {Blind}{BackSpace}
*+t:: Send {Blind}{Delete}
*^t:: Send ^{BackSpace}
*^+t:: Send ^{Delete}

; 回车
*g:: Enter
