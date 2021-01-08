; ========== CapsLockX ==========
; 名称：
; 描述：CapsLockX+P 打开系统设定，拯救笔记本
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 版权：Copyright © 2018-2020 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========

if (!CapsLockX)
    ExitApp


AppendHelp("
(
笔记本功能键补充
| Win + Alt + P | 打开系统设定 |
| 按住 Menu键 + 其它键 | 相当于按住 右Ctrl + 其它键 |
| 单击右Ctrl | 相当于单击Menu键 |
)")
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

; 对于没有 Win 键的环境，用 2个分号一起按来模拟 Win 键
*' Up:: Send {Blind}'
*`; Up:: Send {Blind}`;
' & `;:: LWin
`; & ':: LWin
