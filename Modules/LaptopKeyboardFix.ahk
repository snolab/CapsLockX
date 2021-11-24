; ========== CapsLockX ==========
; 描述：拯救笔记本
; 名称：提供常见缺键补全，例如缺失 Esc 键、右 Ctrl 键、左 Win 键、Pause键、PrtScn键等
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

if !CapsLockX
    ExitApp

global FLAG_SWAP_ESC_STROKE := CapsLockX_Config("CLX_LKF", "FLAG_SWAP_ESC_STROKE", 0, "交换ESC和~键，你可以按CLX+Esc来切换这个选项")

CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))

Return
; 专治 Surface 的残破键盘，合并右Ctrl与Menu键！
; 单击 Menu 键为 Menu 键功能，按住 Menu 键再按别的键则表现为 Ctrl 组合键
$*AppsKey:: Send {Blind}{RControl Down}
$*AppsKey Up::
    if ("AppsKey" == A_PriorKey){
        Send {Blind}{RControl Up}{AppsKey}
    } else {
        Send {Blind}{RControl Up}
    }
Return
~*RControl Up::
    if ("RControl" == A_PriorKey){
        Send {AppsKey}
    }
Return

; Win+Alt+P 打开系统设定 (模拟 pause 键)
$#!p::
    Send #{Pause}
Return

; 对于没有 Win 键的环境，用 Ctrl + ESC 一起按来模拟 Win 键
; ] & [:: LWin【
; *] Up:: Send {Blind}]
; *!\:: Send {Blind}{Tab}

; 对于没有Esc或没有 Stroke 键的键
#if CapsLockXMode

`::   FLAG_SWAP_ESC_STROKE := CapsLockX_ConfigSet("CLX_LKF", "FLAG_SWAP_ESC_STROKE", !FLAG_SWAP_ESC_STROKE, "交换ESC和~键，你可以按CLX+Esc来切换这个选项")
Esc:: FLAG_SWAP_ESC_STROKE := CapsLockX_ConfigSet("CLX_LKF", "FLAG_SWAP_ESC_STROKE", !FLAG_SWAP_ESC_STROKE, "交换ESC和~键，你可以按CLX+Esc来切换这个选项")

#if FLAG_SWAP_ESC_STROKE

*`:: Esc
<^`:: LWin
*Esc:: `

#if !FLAG_SWAP_ESC_STROKE

<^Esc:: LWin