Return
OnSwitch(){
    ; 这里改注册表是为了禁用 Win + L 锁定机器，不过只有用管理员运行才管用。
    value := !!(ModuleState & MF_EditX) ? 0 : 1
    RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, %value%
}

#If CapsXMode == CM_FN
    Space:: Enter
#If CapsXMode == CM_CAPSX || CapsXMode == CM_FN
    ; z:: Send {Enter}
    h:: Left
    l:: Right
    ; hl 一起按相当于选择当前词
    ; h & l:: Send ^{Left}^+{Right}
    ; l & h:: Send ^{Right}^+{Left}
    k:: Up
    j:: Down
    n:: Home
    m:: End
    
    ,:: ^Left
    .:: ^Right

    ; mn 一起按相当于选择当前行
    ; n & m:: Send ^{Home}^+{End}
    ; m & n:: Send ^{End}^+{Home}

    ; 前删，后删
    b:: Send {BackSpace}
    +b:: Send {Delete}
    ^b:: Send ^{BackSpace}
    ^+b:: Send ^{Delete}
    
    z:: Send {Enter}
