; ========== CapsLockX ==========
; 名称：时基加速模型
; 描述：用来模拟按键和鼠标，计算一个虚拟的光标运动物理模型。
; 版本：v2021.10.18
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版权：Copyright © 2017-2021 Snowstar Laboratory. All Rights Reserved.
; ========== CapsLockX ==========


global 鼠标模型 := new 向量加速模型(鼠标模型, [0.1, 1, 2, 3], [0])
; 鼠标模型.


return

#If

]:: 鼠标模型.按([1])
] Up:: 鼠标模型.放([1])
[:: 鼠标模型.按([-1])
[ Up:: 鼠标模型.放([-1])

鼠标模型(v, 事件){
    t := 列串化(v) . 事件
    tooltip %t%
}

class 向量加速模型
{
    __New(事件函数, 加速率, 初值)
    {
        this.QPS := new QPS()
        this.间隔 := 0
        this.动刻 := 0
        this.维数 := 初值.Length()
        this.单位 := 列化一映(初值)
        this.增刻 := 列化零映(初值)
        this.减刻 := 列化零映(初值)
        this.速度 := 列化零映(初值)
        this.位移 := 列化零映(初值)
        this.时钟 := ObjBindMethod(this, "运动循环")
        this.加速率 := 加速率
        this.衰减率 := 0.9
        this.事件函数 := 事件函数
        this.最大速度 := 列数直化(初值, 2147483.647)
        this.中键间隔 := 0.1 ; 100ms
    }
    运动循环(现刻:=0)
    {
        现刻 := 现刻 == 0 ? this.QPS.Tick() : 现刻
        动时 := this.动刻 == 0 ? 0 : 现刻 - this.动刻
        减时 := 列数直减(this.减刻, 现刻), 加时 := 列数直减(this.加刻, 现刻)
        时差 := 列列直减(增时,减时)
        绝对时差 := 列绝对映(时差)
        符号时差 := 列符号映(时差)
        ; 同时按下相当于中键（同时也会取消自动）
        Loop % this.维数{
            if (this.减刻[A_Index] && this.增刻[A_Index] && 绝对时差[A_Index] < this.中键间隔) {
                this.止动()
                this.事件函数.Call(符号时差[A_Index], 0, "中键#" . A_Index) ;维度
                return
            }
        }
        ; 处理移动
        if (0 === 动时){
            this.事件函数.Call(0, 0, "启动")
            this.位移 := this.列符号映(绝对时差) ; 启动位移 = 1
        }
        this.加速 := 列列直幂(绝对时差, 列序化(this.加速率))
        this.速度 := 列列直加(this.速度, this.加速)
        this.位移 := 列列直加(this.位移, this.速度)
        this.输出 := 列取整映(this.位移)
        this.位移 := 列列直减(this.位移, this.输出)
        
        ; debug
        ; msg := dt "`n" 现刻 "`n" this.动刻 "`n" 横加速 "`n" this.横速 "`n" this.横移 "`n" this.横输出
        ; tooltip %msg%
        
        if (横输出 || 纵输出) {
            this.事件函数.Call(this.输出, "移动")
        }
        ; 速度归 0，结束定时器
        if ( !this.横速 && !this.纵速 && !(横输出 || 纵输出)) {
            this.止动()
            Return
        }
    }
    始动() {
        this.动刻 := 0
        this.ticker()
        时钟 := this.时钟
        SetTimer % 时钟, % this.间隔
    }
    止动(){
        this.动刻 := 0, this.动中 := 0
        this.左刻 := 0, this.右刻 := 0
        this.上刻 := 0, this.下刻 := 0
        this.横速 := 0, this.横移 := 0
        this.纵速 := 0, this.纵移 := 0
        时钟 := this.时钟
        SetTimer % 时钟, Off
        this.事件函数.Call(0, 0, "止动")
    }
    ; 冲突止动(){
    ;     在动 := this.动刻 != 0
    ;     启动中 := this.启动中
    ;     if(在动 && !启动中){
    ;         this.止动()
    ;     }
    ; }
    ; 左按(){
    ;     this.左刻 := this.左刻 ? this.左刻 : this._QPC()
    ;     this.始动()
    ; }
    ; 左放(){
    ;     this.左刻 := 0
    ; }
    ; 右按(){
    ;     this.右刻 := this.右刻 ? this.右刻 : this._QPC()
    ;     this.始动()
    ; }
    ; 右放(){
    ;     this.右刻 := 0
    ; }
    ; 上按(){
    ;     this.上刻 := this.上刻 ? this.上刻 : this._QPC()
    ;     this.始动()
    ; }
    ; 上放(){
    ;     this.上刻 := 0
    ; }
    ; 下按(){
    ;     this.下刻 := this.下刻 ? this.下刻 : this._QPC()
    ;     this.始动()
    ; }
    ; 下放(){
    ;     this.下刻 := 0
    ; }
}

class QPS{
    __New(){
        this.QPF := DllCall("QueryPerformanceFrequency", "Int64*", QuadPart)
    }
    Tick(){
        DllCall("QueryPerformanceCounter", "Int64*", Counter)
        return Counter / this.QPF
    }
}
列函映(a,b){
    c:=a.Clone()
    Loop % a.Length()
        c[A_Index]:=b.Call(a[A_Index])
    return c
}
列序化(a){
    c:=a.Clone()
    Loop % a.Length()
        c[A_Index]:=A_Index
    return c
}
列函筛(a,b){
    c:=[]
    Loop % a.Length()
        if(b.Call(a[A_Index]))
            c.Push(a)
    return c
}
列函值归(a,b,c){
    Loop % a.Length()
        c := b.Call(a[A_Index], c)
    return c
}

列串化(a){
    s := "["
    loop % a.Length(){
        s .= a[A_Index]
        s .= ","
    }
    s.="]"
    return s
}

列自积(核, a){
    列 := a.Clone()
    loop % 列.Length(){
        列[A_Index] := 核.Call(a[A_Index])
    }
    return 列
}
列列函映(a, b, c){
    if (!IsObject(b)){
        return 列数函映(a, b, c)
    }
    列 := a.Clone()
    loop % 列.Length(){
        列[A_Index] := c.Call(a[A_Index], b[A_Index])
    }
    return 列
}
列数函映(a, b, c ){
    d := a.Clone()
    loop % d.Length(){
        d[A_Index] := c.Call(a[A_Index], b)
    }
    return 列
}
阵函映( a, b){
    c := a.Clone()
    loop % c.Length(){
        c[A_Index] := 列函映(a[A_Index], b)
    }
    return c
}
阵阵函映( a, b, c){
    if (!IsObject(b)){
        return 阵数函映(a, b, c)
    }
    d := a.Clone()
    loop % d.Length(){
        d[A_Index] := 列列函映(a[A_Index], b[A_Index], c)
    }
    return d
}
阵数函映(a, b, c){
    d := a.Clone()
    loop % d.Length(){
        d[A_Index] := 列数函映( a[A_Index], b, c)
    }
    return d
}

;;; 注：以下代码由 星核.mjs 自动生成
值值值判断(a,b,c){
    return a?b:c
}
值对象吗(a){
    return IsObject(a)
}
值函数吗(a){
    return IsFunc(a)
}
值非(a){
    return !a
}
值值化(a,b){
    return 0?a:b
}
数数化(a,b){
    return 0?a:b
}
数化假(a){
    return 0?a:False
}
数化真(a){
    return 0?a:True
}
数化零(a){
    return 0?a:0
}
数化一(a){
    return 0?a:1
}
数反(a){
    return -a
}
数倒(a){
    return 1/a
}
数Log(a){
    return Log(a)
}
数Exp(a){
    return Exp(a)
}
数符号(a){
    return a==0?0:a>0?1:-1
}
数绝对(a){
    return a<0?-a:a
}
数取整(a){
    return a|0
}
数取零(a){
    return a-a|0
}
列长度(a){
    return a.Length()
}
列克隆(a){
    return a.Clone()
}
串取函(a){
    return Func(a)
}
串长度(a){
    return StrLen(a)
}
数数加(a,b){
    return a+b
}
数数减(a,b){
    return a-b
}
数数乘(a,b){
    return a*b
}
数数除(a,b){
    return a/b
}
数数与(a,b){
    return a&&b
}
数数或(a,b){
    return a||b
}
数数大于(a,b){
    return a>b
}
数数小于(a,b){
    return a<b
}
数数等于(a,b){
    return a==b
}
数数严等(a,b){
    return a===b
}
数数幂(a,b){
    return a**b
}
数数位与(a,b){
    return a&b
}
数数位或(a){
    return a|0
}
列列直化(a,b){
    return 列列函映(a,b,Func("数数化"))
}
列列直加(a,b){
    return 列列函映(a,b,Func("数数加"))
}
列列直减(a,b){
    return 列列函映(a,b,Func("数数减"))
}
列列直乘(a,b){
    return 列列函映(a,b,Func("数数乘"))
}
列列直除(a,b){
    return 列列函映(a,b,Func("数数除"))
}
列列直与(a,b){
    return 列列函映(a,b,Func("数数与"))
}
列列直或(a,b){
    return 列列函映(a,b,Func("数数或"))
}
列列直大于(a,b){
    return 列列函映(a,b,Func("数数大于"))
}
列列直小于(a,b){
    return 列列函映(a,b,Func("数数小于"))
}
列列直等于(a,b){
    return 列列函映(a,b,Func("数数等于"))
}
列列直严等(a,b){
    return 列列函映(a,b,Func("数数严等"))
}
列列直幂(a,b){
    return 列列函映(a,b,Func("数数幂"))
}
列列直位与(a,b){
    return 列列函映(a,b,Func("数数位与"))
}
列列直位或(a,b){
    return 列列函映(a,b,Func("数数位或"))
}
阵阵直化(a,b){
    return 阵阵函映(a,b,Func("数数化"))
}
阵阵直加(a,b){
    return 阵阵函映(a,b,Func("数数加"))
}
阵阵直减(a,b){
    return 阵阵函映(a,b,Func("数数减"))
}
阵阵直乘(a,b){
    return 阵阵函映(a,b,Func("数数乘"))
}
阵阵直除(a,b){
    return 阵阵函映(a,b,Func("数数除"))
}
阵阵直与(a,b){
    return 阵阵函映(a,b,Func("数数与"))
}
阵阵直或(a,b){
    return 阵阵函映(a,b,Func("数数或"))
}
阵阵直大于(a,b){
    return 阵阵函映(a,b,Func("数数大于"))
}
阵阵直小于(a,b){
    return 阵阵函映(a,b,Func("数数小于"))
}
阵阵直等于(a,b){
    return 阵阵函映(a,b,Func("数数等于"))
}
阵阵直严等(a,b){
    return 阵阵函映(a,b,Func("数数严等"))
}
阵阵直幂(a,b){
    return 阵阵函映(a,b,Func("数数幂"))
}
阵阵直位与(a,b){
    return 阵阵函映(a,b,Func("数数位与"))
}
阵阵直位或(a,b){
    return 阵阵函映(a,b,Func("数数位或"))
}
列数直化(a,b){
    return 列数函映(a,b,Func("数数化"))
}
列数直加(a,b){
    return 列数函映(a,b,Func("数数加"))
}
列数直减(a,b){
    return 列数函映(a,b,Func("数数减"))
}
列数直乘(a,b){
    return 列数函映(a,b,Func("数数乘"))
}
列数直除(a,b){
    return 列数函映(a,b,Func("数数除"))
}
列数直与(a,b){
    return 列数函映(a,b,Func("数数与"))
}
列数直或(a,b){
    return 列数函映(a,b,Func("数数或"))
}
列数直大于(a,b){
    return 列数函映(a,b,Func("数数大于"))
}
列数直小于(a,b){
    return 列数函映(a,b,Func("数数小于"))
}
列数直等于(a,b){
    return 列数函映(a,b,Func("数数等于"))
}
列数直严等(a,b){
    return 列数函映(a,b,Func("数数严等"))
}
列数直幂(a,b){
    return 列数函映(a,b,Func("数数幂"))
}
列数直位与(a,b){
    return 列数函映(a,b,Func("数数位与"))
}
列数直位或(a,b){
    return 列数函映(a,b,Func("数数位或"))
}
阵数直化(a,b){
    return 阵数函映(a,b,Func("数数化"))
}
阵数直加(a,b){
    return 阵数函映(a,b,Func("数数加"))
}
阵数直减(a,b){
    return 阵数函映(a,b,Func("数数减"))
}
阵数直乘(a,b){
    return 阵数函映(a,b,Func("数数乘"))
}
阵数直除(a,b){
    return 阵数函映(a,b,Func("数数除"))
}
阵数直与(a,b){
    return 阵数函映(a,b,Func("数数与"))
}
阵数直或(a,b){
    return 阵数函映(a,b,Func("数数或"))
}
阵数直大于(a,b){
    return 阵数函映(a,b,Func("数数大于"))
}
阵数直小于(a,b){
    return 阵数函映(a,b,Func("数数小于"))
}
阵数直等于(a,b){
    return 阵数函映(a,b,Func("数数等于"))
}
阵数直严等(a,b){
    return 阵数函映(a,b,Func("数数严等"))
}
阵数直幂(a,b){
    return 阵数函映(a,b,Func("数数幂"))
}
阵数直位与(a,b){
    return 阵数函映(a,b,Func("数数位与"))
}
阵数直位或(a,b){
    return 阵数函映(a,b,Func("数数位或"))
}
列之化(a){
    return Func("数数化").Call(a[1],a[2])
}
列之加(a){
    return Func("数数加").Call(a[1],a[2])
}
列之减(a){
    return Func("数数减").Call(a[1],a[2])
}
列之乘(a){
    return Func("数数乘").Call(a[1],a[2])
}
列之除(a){
    return Func("数数除").Call(a[1],a[2])
}
列之与(a){
    return Func("数数与").Call(a[1],a[2])
}
列之或(a){
    return Func("数数或").Call(a[1],a[2])
}
列之大于(a){
    return Func("数数大于").Call(a[1],a[2])
}
列之小于(a){
    return Func("数数小于").Call(a[1],a[2])
}
列之等于(a){
    return Func("数数等于").Call(a[1],a[2])
}
列之严等(a){
    return Func("数数严等").Call(a[1],a[2])
}
列之幂(a){
    return Func("数数幂").Call(a[1],a[2])
}
列之位与(a){
    return Func("数数位与").Call(a[1],a[2])
}
列之位或(a){
    return Func("数数位或").Call(a[1],a[2])
}
列化假映(a){
    return 列函映(a,Func("数化假"))
}
列化真映(a){
    return 列函映(a,Func("数化真"))
}
列化零映(a){
    return 列函映(a,Func("数化零"))
}
列化一映(a){
    return 列函映(a,Func("数化一"))
}
列反映(a){
    return 列函映(a,Func("数反"))
}
列倒映(a){
    return 列函映(a,Func("数倒"))
}
列Log映(a){
    return 列函映(a,Func("数Log"))
}
列Exp映(a){
    return 列函映(a,Func("数Exp"))
}
列符号映(a){
    return 列函映(a,Func("数符号"))
}
列绝对映(a){
    return 列函映(a,Func("数绝对"))
}
列取整映(a){
    return 列函映(a,Func("数取整"))
}
列取零映(a){
    return 列函映(a,Func("数取零"))
}
列化假筛(a){
    return 列函筛(a,Func("数化假"))
}
列化真筛(a){
    return 列函筛(a,Func("数化真"))
}
列化零筛(a){
    return 列函筛(a,Func("数化零"))
}
列化一筛(a){
    return 列函筛(a,Func("数化一"))
}
列反筛(a){
    return 列函筛(a,Func("数反"))
}
列倒筛(a){
    return 列函筛(a,Func("数倒"))
}
列Log筛(a){
    return 列函筛(a,Func("数Log"))
}
列Exp筛(a){
    return 列函筛(a,Func("数Exp"))
}
列符号筛(a){
    return 列函筛(a,Func("数符号"))
}
列绝对筛(a){
    return 列函筛(a,Func("数绝对"))
}
列取整筛(a){
    return 列函筛(a,Func("数取整"))
}
列取零筛(a){
    return 列函筛(a,Func("数取零"))
}
列化归(a,b){
    return 列函值归(a,Func("数数化"),b)
}
列加归(a,b){
    return 列函值归(a,Func("数数加"),b)
}
列减归(a,b){
    return 列函值归(a,Func("数数减"),b)
}
列乘归(a,b){
    return 列函值归(a,Func("数数乘"),b)
}
列除归(a,b){
    return 列函值归(a,Func("数数除"),b)
}
列与归(a,b){
    return 列函值归(a,Func("数数与"),b)
}
列或归(a,b){
    return 列函值归(a,Func("数数或"),b)
}
列大于归(a,b){
    return 列函值归(a,Func("数数大于"),b)
}
列小于归(a,b){
    return 列函值归(a,Func("数数小于"),b)
}
列等于归(a,b){
    return 列函值归(a,Func("数数等于"),b)
}
列严等归(a,b){
    return 列函值归(a,Func("数数严等"),b)
}
列幂归(a,b){
    return 列函值归(a,Func("数数幂"),b)
}
列位与归(a,b){
    return 列函值归(a,Func("数数位与"),b)
}
列位或归(a,b){
    return 列函值归(a,Func("数数位或"),b)
}
阵化假映(a){
    return 阵函映(a,Func("数化假"))
}
阵化真映(a){
    return 阵函映(a,Func("数化真"))
}
阵化零映(a){
    return 阵函映(a,Func("数化零"))
}
阵化一映(a){
    return 阵函映(a,Func("数化一"))
}
阵反映(a){
    return 阵函映(a,Func("数反"))
}
阵倒映(a){
    return 阵函映(a,Func("数倒"))
}
阵Log映(a){
    return 阵函映(a,Func("数Log"))
}
阵Exp映(a){
    return 阵函映(a,Func("数Exp"))
}
阵符号映(a){
    return 阵函映(a,Func("数符号"))
}
阵绝对映(a){
    return 阵函映(a,Func("数绝对"))
}
阵取整映(a){
    return 阵函映(a,Func("数取整"))
}
阵取零映(a){
    return 阵函映(a,Func("数取零"))
}
;;; 注：以上代码由 星核.mjs 自动生成
