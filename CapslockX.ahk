CoordMode, Mouse, Screen

QPF()
{
    DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
    Return QuadPart
}

QPC()
{
    DllCall("QueryPerformanceCounter", "Int64*", Counter)
    Return Counter
}

ma(t){
    ; Return ma2(t) ; 二次函数运动模型
    ; Return ma3(t) ; 三次函数运动模型
    Return maPower(t) ; 指数函数运动模型
}
ma2(t){
    ; x-t 二次曲线加速运动模型
    ; 跟现实世界的运动一个感觉
    If(0 == t)
        Return 0
    If(t > 0)
        Return  6
    Else
        Return -6
}

ma3(t){
    ; x-t 三次曲线函数运动模型
    ; 与现实世界不同，
    ; 这个模型会让人感觉鼠标比较“重”
    ;
    If(0 == t)
        Return 0
    If(t > 0)
        Return t * 12
    Else
        Return t * 12
}

maPower(t){
    ; x-t 指数曲线运动的简化模型
    ; 这个模型可以满足精确定位需求，也不会感到鼠标“重”
    ; 但是因为跟现实世界的运动曲线不一样，凭直觉比较难判断落点，需要一定练习才能掌握。
    ;
    If(0 == t)
        Return 0
    If(t > 0)
        Return  ( Exp( t) - 0.95 ) * 16
    Else
        Return -( Exp(-t) - 0.95 ) * 16
}

; 时间计算
dt(t, tNow){
    Return t ? (tNow - t) / QPF() : 0
}

MoCaLi(v, a){ ; 摩擦力
    If((a > 0 And v > 0) Or (a < 0 And v < 0))
        Return v
    ; 简单粗暴倍数降速
    v *= 0.8
    If(v > 0)
        v -= 1
    If(v < 0)
        v += 1
    v //= 1
    Return v
}


^!F12:: ExitApp


CapsLock::
    ; 这里改注册表是为了禁用 Win + L，不过只有用管理员运行才管用。。。
    If(GetKeyState("ScrollLock", "T"))
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 0
    Else
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 1
    Send {ScrollLock}
    Return

!CapsLock:: CapsLock

; 鼠标加速度微分对称模型，每秒误差 2.5ms 以内
Global ta := 0, td := 0, tw := 0, ts := 0, mvx := 0, mvy := 0

; 滚轮加速度微分对称模型（不要在意名字hhhh
Global tr := 0, tf := 0, tz := 0, tc := 0, svx := 0, svy := 0

#If GetKeyState("ScrollLock", "T")
    Pause::
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 1
        DllCall("LockWorkStation")
        Sleep, 1000
        RegWrite, REG_DWORD, HKEY_CURRENT_USER, Software\Microsoft\Windows\CurrentVersion\Policies\System, DisableLockWorkstation, 0
        Return
    ~#Tab:: Send {ScrollLock}
    
    `:: Enter

    h:: Left
    l:: Right
    j:: Down
    k:: Up
    n:: Home
    m:: End
    b:: Send {Delete}

    ; 窗口
    x:: Send ^w
    !x:: Send !{F4}
    ; 撤销
    u:: Send ^z
    ; 重做
    +u:: Send ^z

    1:: Send #1
    2:: Send #2
    3:: Send #3
    4:: Send #4
    5:: Send #5
    6:: Send #6
    7:: Send #7
    8:: Send #8
    9:: Send #9

    F5:: Send {Media_Play_Pause}
    F6:: Send {Media_Prev}
    F7:: Send {Media_Next}
    F8:: Send {Media_Stop}

    
    F10:: Send {Volume_Mute}
    F11:: Send {Volume_Down}
    F12:: Send {Volume_Up}

    ; Google 搜索
    search(q)
    {
        Run, https://www.google.com/search?q=%q%
    }
    copySelected()
    {
        Send ^c
        ClipWait
        Return Clipboard
    }
    g:: search(copySelected())


    ; 鼠标运动处理
    mm:
        tNow := QPC()
        ; 计算用户操作时间
        tda := dt(ta, tNow),           tdd := dt(td, tNow)
        tdw := dt(tw, tNow),           tds := dt(ts, tNow)

        ; 计算加速度
        max := ma(tdd - tda),          may := ma(tds - tdw)

        ; 摩擦力不阻碍用户意志
        mvx := MoCaLi(mvx + max, max), mvy := MoCaLi(mvy + may, may)

        MouseMove, %mvx%, %mvy%, 0, R

        If(0 == mvx And 0 == mvy)
            SetTimer, mm, Off
        Return

    ; 时间处理
    mTick(){
        SetTimer, mm, 1
    }

    a:: ta := (ta ? ta : QPC()), mTick()
    d:: td := (td ? td : QPC()), mTick()
    w:: tw := (tw ? tw : QPC()), mTick()
    s:: ts := (ts ? ts : QPC()), mTick()
    a Up:: ta := 0, mTick()
    d Up:: td := 0, mTick()
    w Up:: tw := 0, mTick()
    s Up:: ts := 0, mTick()

    e:: LButton
    q:: RButton


    Pos2Long(x, y){
      return x | (y << 16)
    }

    ; 滚轮运动处理
    msx:
        tNow := QPC()
        ; 计算用户操作时间
        tdz := dt(tz, tNow), tdc := dt(tc, tNow)
        ; 计算加速度
        sax := ma(tdc - tdz)
        svx := MoCaLi(svx + sax, sax)

        MouseGetPos, mouseX, mouseY, wid, fcontrol
        wParam := svx << 16 ;zDelta
        lParam := Pos2Long(mouseX, mouseY)
        PostMessage, 0x20E, %wParam%, %lParam%, %fcontrol%, ahk_id %wid%

        If(0 == svx)
            SetTimer, msx, Off
        Return
    
    
    msy:
    {
        tNow := QPC()
        ; 计算用户操作时间
        tdr := dt(tr, tNow), tdf := dt(tf, tNow)
        ; 计算加速度
        say := ma(tdr - tdf)
        svy := MoCaLi(svy + say, say)

        MouseGetPos, mouseX, mouseY, id, fcontrol
        wParam := svy << 16 ;zDelta
        lParam := Pos2Long(mouseX, mouseY)
        PostMessage, 0x20A, %wParam%, %lParam%, %fcontrol%, ahk_id %id%

        If(0 == svy)
            SetTimer, msy, Off
        Return
    }

    ; 时间处理
    sTickx(){
        SetTimer, msx, 1
    }
    sTicky(){
        SetTimer, msy, 1
    }

    r:: tr := (tr ? tr : QPC()), sTicky()
    f:: tf := (tf ? tf : QPC()), sTicky()
    z:: tz := (tz ? tz : QPC()), sTickx()
    c:: tc := (tc ? tc : QPC()), sTickx()
    r Up:: tr := 0, sTicky()
    f Up:: tf := 0, sTicky()
    z Up:: tz := 0, sTickx()
    c Up:: tc := 0, sTickx()