; ========== CapsLockX ==========
; 名称：雪星并击 | snochorded
; 描述：启用雪星并击，创建于(20200722) 文档请见主页 [雪星并击 snochorded sno-chord-typing | snochorded]( https://snomiao.github.io/snochorded/ )
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：[雪星并击 snochorded sno-chord-typing | snochorded]( https://snomiao.github.io/snochorded/ )
; 版权：Copyright 2020-2022 snomiao@gmail.com
; 版本：v0.0.1
; ========== CapsLockX ==========

; update: https://github.com/snomiao/snochorded/raw/master/%E9%9B%AA%E6%98%9F%E5%B9%B6%E5%87%BB.ahk

FileEncoding, UTF-8

; 開關 默认关
global SnoChordTypingEnable := CapsLockX_Config("Plugins", "EnableSnoChordTyping", 0, "启用雪星并击（实验中），")
if (!T_SnoChordTypingEnable)
    Return

雪星并击_初始化()

Return

雪星并击_初始化(){
    #MaxHotkeysPerInterval, 200
    ; 配置
    global SnoChordTypingChordIntervalThreshold := 32
    global SnoChordTypingAllowRewriteString := "qwertasdfgzxcvbpyuiohjklnm"
    global SnoChordTypingAllowRewrite := 0
    global SnoChordTypingAppendSpace := 0
    global SnoChordTypingStageList := []
    
    ; 变量
    global SnoChordTypingPressedKeySet := {}
    global SnoChordTypingTypedKeys := ""
    global SnoChordTypingLastKeyDownTick := 0
    
    ; 读入配置
    ConfigPath := CapsLockX_配置目录 "/雪星并击配置.ini"
    IniRead, SnoChordTypingChordIntervalThreshold, %ConfigPath%, Common, SnoChordTypingChordIntervalThreshold, %SnoChordTypingChordIntervalThreshold%
    IniWrite, %SnoChordTypingChordIntervalThreshold%, %ConfigPath%, Common, SnoChordTypingChordIntervalThreshold
    IniRead, SnoChordTypingAllowRewriteString, %ConfigPath%, Common, SnoChordTypingAllowRewriteString, %SnoChordTypingAllowRewriteString%
    IniWrite, %SnoChordTypingAllowRewriteString%, %ConfigPath%, Common, SnoChordTypingAllowRewriteString
    IniRead, SnoChordTypingAllowRewrite, %ConfigPath%, Common, SnoChordTypingAllowRewrite, %SnoChordTypingAllowRewrite%
    IniWrite, %SnoChordTypingAllowRewrite%, %ConfigPath%, Common, SnoChordTypingAllowRewrite
    IniRead, SnoChordTypingAppendSpace, %ConfigPath%, Common, SnoChordTypingAppendSpace, %SnoChordTypingAppendSpace%
    IniWrite, %SnoChordTypingAppendSpace%, %ConfigPath%, Common, SnoChordTypingAppendSpace
    ; MsgBox, , , SnoChordTypingAllowRewriteString: %SnoChordTypingAllowRewriteString%
    
    ; 默认键位数据
    RuleStage1 :="
    (
    =| 组合-键 | 组合-键 | 组合-键 | 组合-键 | 组合-键 |
    =| ------- | ------- | ------- | ------- | ------- |
    =| _1 => 1 | _2 => 2 | _3 => 3 | _4 => 4 | _5 => 5 |
    =| _q => q | _w => w | _e => e | _r => r | _t => t |
    =| _a => a | _s => s | _d => d | _f => f | _g => g |
    =| _z => z | _x => x | _c => c | _v => v | _b => b |
    =| 12 => 0 | 13 => 9 | 23 => 8 | 24 => 7 | 34 => 6 |
    =| 21 => 0 | 31 => 9 | 32 => 8 | 42 => 7 | 43 => 6 |
    =| qw => p | qe => o | we => i | wr => u | er => y |
    =| wq => p | eq => o | ew => i | rw => u | re => y |
    =| as => ; | ad => l | sd => k | sf => j | df => h |
    =| sa => ; | da => l | ds => k | fs => j | fd => h |
    =| zx => / | zc => . | xc =>, | xv => m | cv => n |
    =| xz => / | cz => . | cx =>, | vx => m | vc => n |
    
    )"
    RuleStage2 := "
    (
    =| 组合-键 | 组合-键 | 组合-键 | 组合-键 | 组合-键 |
    =| ------- | ------- | ------- | ------- | ------- |
    =| _6 => 6 | _7 => 7 | _8 => 8 | _9 => 9 | _0 => 0 |
    =| _y => y | _u => u | _i => i | _o => o | _p => p |
    =| _h => h | _j => j | _k => k | _l => l | _; => ; |
    =| _n => n | _m => m | _, =>, | _. => . | _/ => / |
    =| 78 => 5 | 79 => 4 | 89 => 3 | 08 => 2 | 09 => 1 |
    =| 87 => 5 | 97 => 4 | 98 => 3 | 80 => 2 | 90 => 1 |
    =| ui => t | uo => r | io => e | pi => w | po => q |
    =| iu => t | ou => r | oi => e | ip => w | op => q |
    =| jk => g | jl => f | kl => d | ;k => s | ;l => a |
    =| kj => g | lj => f | lk => d | k; => s | l; => a |
    =| m, => b | m. => v |, . => c | /, => x | /. => z |
    =|, m => b | .m => v | ., => c |, / => x | ./ => z |
    
    )"
    RuleStage3 := "
    (
    =| 组合-键 | 组合-键 | 组合-键 | 组合-键 | 组合-键 |
    =| ------- | ------- | ------- | ------- | ------- |
    =| _- => - | _= => = | _[ => [ | _] => ] | _\ => \ |
    =| _' => ' | __ => _ |
    )"
    
    ; 编译键位数据
    StageIndex := 1
    while (1) {
        IniRead, RuleStage, %ConfigPath%, RuleStage%StageIndex%
        ; MsgBox, , , % "asdf" RuleStage "zxcv" StageIndex "asdf" !RuleStage
        if (!RuleStage)
            Break
        RuleStage%StageIndex% := RuleStage
        ; MsgBox, , , RuleStage: %RuleStage%
        
        objRule := {}
        FoundPos := 0
        while (FoundPos := RegExMatch(RuleStage, "O)\s_*?(\S+)\s*?=>\s*?(\S+)\s", SubPat, FoundPos+1)) {
            MapFrom := "" SubPat.Value(1)
            MapTo := SubPat.Value(2) ""
            
            ; MsgBox, , , % SubPat.Value(1) "=" SubPat.Value(2)
            
            MapFrom := StrReplace(MapFrom, "_", " ")
            MapTo := StrReplace(MapTo, "_", " ")
            
            objRule[MapFrom] := MapTo
        }
        
        objKeys := {}
        lsKey := []
        RightHandKeysSet := []
        for Keys, _ in objRule {
            ; MsgBox, , , % "Keys" Keys
            KeysList := StrSplit(Keys)
            Loop % KeysList.MaxIndex()
            {
                Key := KeysList[A_Index]
                Key := StrReplace(Key, "_", " ")
                ; MsgBox, , , % Key
                objKeys[Key] := 1
            }
        }
        for Key, _ in objKeys {
            ; MsgBox, , , % Key
            lsKey.Push(Key)
        }
        Stage := {"objKeys": objKeys, "lsKey": lsKey, "objRule": objRule, "Pressed": "", "Typed": ""}
        SnoChordTypingStageList.Push(Stage)
        StageIndex++
    }
    ; 将编译好的键位数据写入 INI
    IniWrite, %RuleStage1%, %ConfigPath%, RuleStage1
    IniWrite, %RuleStage2%, %ConfigPath%, RuleStage2
    IniWrite, %RuleStage3%, %ConfigPath%, RuleStage3
    
    ; 挂载并击热键
    Hotkey, if, !CapsLockXMode
    for _, Stage in SnoChordTypingStageList {
        for _, KeyName in Stage["lsKey"]{
            KeyName := StrReplace(KeyName, " ", "Space")
            ; 只有字母直接按下会不导致输入法上屏
            if (SnoChordTypingAllowRewrite && InStr(SnoChordTypingAllowRewriteString, KeyName)) {
                Hotkey, ~$%KeyName%, SnoChordKeyDown
                Hotkey, ~$+%KeyName%, SnoChordKeyDown
            } else {
                Hotkey, $%KeyName%, SnoChordKeyDown
                Hotkey, $+%KeyName%, SnoChordKeyDown
            }
            Hotkey, ~$*%KeyName% Up, SnoChordKeyUp
        }
    }
    Hotkey, if, 
    
}

snochorded_output_recored_keys()
{
    OutputKey := ""
    for _, Stage in SnoChordTypingStageList {
        StagePressed := Stage["Pressed"]
        Replaced := Stage["objRule"][StagePressed ""]
        OutputKey .= Replaced ? Replaced : StagePressed
        Stage["Pressed"] := ""
    }
    lenTyped := StrLen(SnoChordTypingTypedKeys)
    OutputLength := StrLen(OutputKey)
    OutputChanged := SnoChordTypingTypedKeys != OutputKey
    ; Clean
    SnoChordTypingLastKeyDownTick := 0
    SnoChordTypingTypedKeys := ""
    if (OutputChanged && OutputLength) {
        ; OutputKey .= " "
        OutputKey := "{blind}" . OutputKey
        loop, % lenTyped {
            OutputKey := "{BackSpace}" . OutputKey
        }
        ; send event is most stable
        SendEvent % OutputKey
    }
    if (SnoChordTypingAppendSpace) {
        SendEvent % " "
    }
}

SnoChordKeyDown()
{
    NowTick := A_TickCount
    ThisKey := A_ThisHotkey
    ThisKey := StrReplace(ThisKey, "~")
    ThisKey := StrReplace(ThisKey, "$")
    ThisKey := StrReplace(ThisKey, "+")
    ThisKey := StrReplace(ThisKey, "Space", " ")
    
    if ( SnoChordTypingLastKeyDownTick == 0
        || NowTick - SnoChordTypingLastKeyDownTick <= SnoChordTypingChordIntervalThreshold){
        if (SubStr(A_ThisHotkey, 1, 1)=="~") {
            SnoChordTypingTypedKeys .= ThisKey
        }
    } else {
        snochorded_output_recored_keys()
    }
    
    for StageIndex, Stage in SnoChordTypingStageList {
        if (Stage["objKeys"].HasKey(ThisKey)) {
            Stage["Pressed"] .= ThisKey
            Break
        }
    }
    SnoChordTypingLastKeyDownTick := NowTick
}

SnoChordKeyUp()
{
    snochorded_output_recored_keys()
    ; PressedKeys is only for debug
    ; if (PressedKeys)
    ;     ToolTip % SnoChordTypingTypedKeys " | " PressedKeys "(" lenTyped ")" " => " OutputKey "("  OutputLength ")"
    ; PressedKeys := ""
}

SnoChordKeyDown:
    SnoChordKeyDown()
return
SnoChordKeyUp:
    SnoChordKeyUp()
return

#if !CapsLockXMode