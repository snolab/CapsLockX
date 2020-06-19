; Exit if running without CapslockX
If(!CapslockX)
    ExitApp

; setup done
Return


#If CapslockXMode == CM_CAPSX || CapslockXMode == CM_FN

; 打开系统设定
p::
    Send #{Pause}
Return