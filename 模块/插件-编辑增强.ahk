Return
OnSwitch(){
    ; 这里改注册表是为了禁用 Win + L 锁定机器，让 Win+hjkl 可以挪窗口位置，不过只有用管理员运行才管用。
    value := !!(ModuleState & MF_EditX) ? 0 : 1
    RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, %value%
}

; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
global ktl := 0, ktr := 0, ktu := 0, ktd := 0, kvx := 0, kvy := 0

kTicker:

    If ( 0 == GetKeyState("h", "P") ){
        ktl := 0
    }
    If ( 0 == GetKeyState("l", "P") ){
        ktr := 0
    }
    If ( 0 == GetKeyState("k", "P") ){
        ktu := 0
    }
    If ( 0 == GetKeyState("j", "P") ){
        ktd := 0
    }

    tNow := QPC()
    ; 计算用户操作时间
    tda := dt(ktl, tNow),          tdd := dt(ktr, tNow)
    tdw := dt(ktu, tNow),          tds := dt(ktd, tNow)

    ; 计算加速度
    kax := ma(tdd -tda)
    kay := ma(tds -tdw)

    ; 在非CapslockX模式下停止
    If (!(CapslockXMode == CM_CapslockX || CapslockXMode == CM_FN)){
        kax := 0
        kay := 0
    }
    ; 摩擦力不阻碍用户意志
    kvx := Friction(kvx + kax, kax), kvy := Friction(kvy + kay, kay)

    ; 稳定化
    If(Abs(kvx) < 0.01)
        kvx := 0
    If(Abs(kvy) < 0.01)
        kvy := 0

    ; TODO: 输出速度时间曲线，用于DEBUG

    If(kvx Or kvy){
        If(kvx >= 10){
            SendInput {Right}
            kvx := kvx - 10
        }
        If(kvx <= -10){
            SendInput {Left}
            kvx := kvx + 10
        }
        If(kvy >= 10){
            SendInput {Down}
            kvy := kvy - 10
        }
        If(kvy <= -10){
            SendInput {Up}
            kvy := kvy + 10
        }
    }Else{
        SetTimer, kTicker, Off
    }

    Return
; 时间处理
kTick(){
    SetTimer, kTicker, 0
}


#If CapslockXMode == CM_FN
    *Space:: Enter

#If CapslockXMode == CM_CapslockX || CapslockXMode == CM_FN
    *u:: PgDn
    *i:: PgUp
    ; 上下左右
    ; 不知为啥这个kj在OneNote里有时候会不管用

    ; 光标运动处理
    *h:: Left
    *j:: Down
    *k:: UP
    *l:: Right

    ; ; ; 光标运动处理
    ; *h:: ktl := (ktl ? ktl : QPC()), kTick()
    ; *l:: ktr := (ktr ? ktr : QPC()), kTick()
    ; *k:: ktu := (ktu ? ktu : QPC()), kTick()
    ; *j:: ktd := (ktd ? ktd : QPC()), kTick()
    ; *h Up:: ktl := 0, kTick()
    ; *l Up:: ktr := 0, kTick()
    ; *k Up:: ktu := 0, kTick()
    ; *j Up:: ktd := 0, kTick()
    
    ; 试过下面这样子的还是不管用
    ; *k::    SendInput {Blind}{Up Down} 
    ; *k Up:: SendInput {Blind}{Up Up}
    ; *j::    SendInput {Blind}{Down Down}
    ; *j Up:: SendInput {Blind}{Down Up}
    
    *n:: Home
    *m:: End

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
    
    *z:: Enter
    