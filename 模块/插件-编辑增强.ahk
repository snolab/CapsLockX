Return
OnSwitch(){
    ; 这里改注册表是为了禁用 Win + L 锁定机器，让 Win+hjkl 可以挪窗口位置，不过只有用管理员运行才管用。
    value := !!(ModuleState & MF_EditX) ? 0 : 1
    RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, %value%
}
    
#If CapsXMode == CM_FN
    Space:: Enter

#If CapsXMode == CM_CAPSX || CapsXMode == CM_FN
    u:: PgDn
    i:: PgUp
    ; 上下左右
    h:: Left
    l:: Right

    ; 不知为啥这个kj在OneNote里有时候会不管用
    k:: Up
    j:: Down
    
    ; 试过下面这样子的还是不管用
    ; *k::    SendInput {Blind}{Up Down} 
    ; *k Up:: SendInput {Blind}{Up Up}
    ; *j::    SendInput {Blind}{Down Down}
    ; *j Up:: SendInput {Blind}{Down Up}

        
    n:: Home
    m:: End

    ; hl 一起按相当于选择当前词
    ; h & l:: Send ^{Left}^+{Right}
    ; l & h:: Send ^{Right}^+{Left}
    
    ; ,:: ^Left
    ; .:: ^Right

    ; mn 一起按相当于选择当前行，不同的顺序影响按完之后的光标位置（在前在后）
    n & m:: Send {Home}+{End}
    m & n:: Send {End}+{Home}

    ; 前删，后删
    b:: Send {Blind}{BackSpace}
    +b:: Send {Delete}
    ; ^b:: Send ^{BackSpace}
    ; ^+b:: Send ^{Delete}
    
    z:: Send {Enter}
    