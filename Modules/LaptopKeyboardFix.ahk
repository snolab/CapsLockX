; ========== CapsLockX ==========
; 描述：拯救笔记本
; 名称：提供常见缺键补全，例如缺失 Esc 键、右 Ctrl 键、左 Win 键、Pause键、PrtScn键等
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2017-2022 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

if !CapsLockX
    ExitApp
global WinKeySimulate := CapsLockX_Config("LKF", "WinKeySimulate", 1, "右手 \][ 模拟Windows键和 Alt + Tab， 具体用法参见LaptopKeyboardFix 模块说明，默认启用")
global FLAG_SWAP_ESC_STROKE := false
CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))

Return
; 专治 Surface 的残破键盘，合并右Ctrl与Menu键！
; 单击 Menu 键为 Menu 键功能，按住 Menu 键再按别的键则表现为 Ctrl 组合键
$*AppsKey:: Send {Blind}{RControl Down}
$*AppsKey Up::
    if ("AppsKey" == A_PriorKey) {
        Send {Blind}{RControl Up}{AppsKey}
    } else {
        Send {Blind}{RControl Up}
    }
Return
~*RControl Up::
    if ("RControl" == A_PriorKey) {
        Send {AppsKey}
    }
Return

; Win+Alt+P 打开系统设定 (模拟 pause 键)
$#!p::
    Send #{Pause}
Return

; 针对安卓对Windows的远程桌面 RD Client

; 对于没有 Win 键的环境，用 Ctrl + ESC 一起按来模拟 Win 键（不行，Win键和其它修饰键一起按打不开开始菜单。）

#if WinKeySimulate && !CapsLockXMode

; Windows 键模拟于 ] + [
] & [:: LWin
*+]:: Send {Blind}]
*!]:: Send {Blind}]
*^]:: Send {Blind}]
] Up:: Send {Blind}]

; Alt+Tab 模拟
RAlt & \:: Send {Blind}{Tab}

; 对于没有Esc或没有 Stroke 键的键
#if CapsLockXMode & CM_FN

`::   FLAG_SWAP_ESC_STROKE := CapsLockX_ConfigSet("CLX_LKF", "FLAG_SWAP_ESC_STROKE", !FLAG_SWAP_ESC_STROKE, "交换ESC和~键，你可以按CLX+Esc来切换这个选项")
Esc:: FLAG_SWAP_ESC_STROKE := CapsLockX_ConfigSet("CLX_LKF", "FLAG_SWAP_ESC_STROKE", !FLAG_SWAP_ESC_STROKE, "交换ESC和~键，你可以按CLX+Esc来切换这个选项")

#if FLAG_SWAP_ESC_STROKE
    
*`:: Esc
<^`:: LWin
*Esc:: `

#if !FLAG_SWAP_ESC_STROKE
    
; <^Esc:: LWin
