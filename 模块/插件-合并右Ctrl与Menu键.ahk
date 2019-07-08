; 专治 Surface 的残破键盘，合并右Ctrl与Menu键！
; 单击 Menu 键为 Menu 键功能，按住 Menu 键再按别的键则表现为 Ctrl 组合键
$*AppsKey:: Send {Blind}{RControl Down}
$*AppsKey Up::
    If ("AppsKey" == A_PriorKey){
        Send {Blind}{RControl Up}{AppsKey}
    }Else{
        Send {Blind}{RControl Up}
    }
    Return
~*RControl Up::
    If ("RControl" == A_PriorKey) {
        Send {AppsKey}
    }
    Return