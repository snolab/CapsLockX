; ========== CapsLockX ==========
; 名称：编辑增强
; 版本：v2020.06.27
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========
;
; 光标加速度微分对称模型（不要在意这中二的名字hhhh
global arrow_tl := 0, arrow_tr := 0, arrow_tu := 0, arrow_td := 0, arrow_vx := 0, arrow_vy := 0, arrow_dx := 0, arrow_dy := 0, ArrowTickerTiming := False

AppendHelp("
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

ArrowTicker:
    ArrowTicker()
Return

OnSwitch()
{
    ; 这里改注册表是为了禁用 Win + L 锁定机器，让 Win+hjkl 可以挪窗口位置，不过只有用管理员运行才管用。
    value := !!(ModuleState & MF_EditX) ? 0 : 1
    RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, %value%
}

SendArrowUp()
{
    if WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") {
        ControlSend, OneNote::DocumentCanvas1, {Blind}{Up}
    } else {
        SendEvent {Blind}{up}
    }
}
SendArrowDown()
{
    ; sendplay {Blind}{down}
    if WinActive(".*- OneNote ahk_class Framework\:\:CFrame ahk_exe ONENOTE.EXE") {
        ControlSend, OneNote::DocumentCanvas1, {Blind}{Down}
    } else {
        SendEvent {Blind}{down}
    }
}

ArrowTicker()
{
    ; 在非 CapsLockX 模式下直接停止
    if (!(CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN)) {
        ArrowTickerStop()
        Return
    }
    ; else{
    tNow := QPC()
    ; 计算用户操作时间
    tda := dt(arrow_tl, tNow), tdd := dt(arrow_tr, tNow)
    tdw := dt(arrow_tu, tNow), tds := dt(arrow_td, tNow)
    ; 计算加速度
    kax := ma(tdd - tda), kay := ma(tds - tdw)

    ; 摩擦力不阻碍用户意志
    arrow_vx := Friction(arrow_vx + kax, kax), arrow_vy := Friction(arrow_vy + kay, kay)

    ; 稳定化
    arrow_dx += arrow_vx / 200, arrow_dy += arrow_vy / 200
    ; ToolTip, %arrow_tl% _ %arrow_tr% _ %arrow_tu% _ %arrow_td% `n %kax% _ %kay% _ %arrow_vx% _ %arrow_vy% _ %arrow_dx% _ %arrow_dy%
    ; msgbox % arrow_dx
    ; 完成移动时
    if ( 0 == arrow_vx && 0 == arrow_vy) {
        ArrowTickerStop()
        Return
    }
    ; ToolTip, % arrow_dx " " arrow_dy
    ; TODO: 输出速度时间曲线，用于DEBUG
    if (arrow_dx >= 1) {
        Loop %arrow_dx% {
            SendEvent {Blind}{Right}
        }
        arrow_dx -= arrow_dx | 0
    }
    if (arrow_dx <= -1) {
        arrow_dx := -arrow_dx
        Loop %arrow_dx% {
            SendEvent {Blind}{Left}
        }
        arrow_dx := -arrow_dx
        arrow_dx -= arrow_dx | 0
    }
    if (arrow_dy >= 1) {
        Loop %arrow_dy%
        {
            SendArrowDown()
        }
        arrow_dy -= arrow_dy | 0
    }
    if (arrow_dy <= -1) {
        arrow_dy := -arrow_dy
        Loop %arrow_dy%
        {
            SendArrowUp()
        }
        arrow_dy := -arrow_dy
        arrow_dy -= arrow_dy | 0
    }
}

; 时间处理
ArrowTickerStart()
{
    if (!ArrowTickerTiming) {
        ArrowTickerTiming := True
    }
    SetTimer, ArrowTicker, 0
}
ArrowTickerStop()
{
    ; 重置相关参数
    arrow_tl := 0, arrow_tr := 0, arrow_tu := 0, arrow_td := 0, arrow_vx := 0, arrow_vy := 0, arrow_dx := 0, arrow_dy := 0
    ; 退出定时
    ArrowTickerTiming := False
    SetTimer, ArrowTicker, Off
}

#if CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN

*u:: PgDn
*i:: PgUp
; 上下左右

; 不知为啥这个kj在OneNote里有时候会不管用, 于是就设定了特殊的编辑操作
; 见 OneNote 2016 增强

; 光标运动处理
; *h:: Left
; *j:: Down
; *k:: UP
; *l:: Right

; ; ; 光标运动处理
ArrowLeftPressed()
{
    ArrowTickerStart()
    if (arrow_tl) {
        Return
    }
    if (arrow_tr) {
        ; 选中当前词
        SendEvent ^{Right}^+{Left}
        arrow_tr := 0
        Return
    }
    arrow_tl := QPC()
    SendEvent {Blind}{Left}
}
ArrowRightPressed()
{
    ArrowTickerStart()
    if (arrow_tr) {
        Return
    }
    if (arrow_tl) {
        ; 选中当前词
        SendEvent ^{Left}^+{Right}
        arrow_tl := 0
        Return
    }
    arrow_tr := QPC()
    SendEvent {Blind}{Right}
}
ArrowUpPressed()
{
    ArrowTickerStart()
    if (arrow_tu) {
        Return
    }
    if (arrow_td) {
        ; KJ一起按
        arrow_td := 0
        Return
    }
    arrow_tu := QPC()
    SendArrowUp()
}
ArrowDownPressed()
{
    ArrowTickerStart()
    if (arrow_td) {
        Return
    }
    if (arrow_tu) {
        ; KJ一起按
        arrow_tu := 0
        Return
    }
    arrow_td := QPC()
    SendArrowDown()
}

;
*h:: ArrowLeftPressed()
*l:: ArrowRightPressed()
*k:: ArrowUpPressed()
*j:: ArrowDownPressed()
*h Up:: arrow_tl := 0, ArrowTickerStart()
*l Up:: arrow_tr := 0, ArrowTickerStart()
*k Up:: arrow_tu := 0, ArrowTickerStart()
*j Up:: arrow_td := 0, ArrowTickerStart()

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

