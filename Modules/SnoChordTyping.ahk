; 雪星并击 | snochorded
; Copyright 2020 snomiao@gmail.com
; LICENSE - GPLv3
; (20200722) 创建
; Save this file as utf8 with bom
; update: https://github.com/snomiao/snochorded/raw/master/%E9%9B%AA%E6%98%9F%E5%B9%B6%E5%87%BB.ahk
#MaxHotkeysPerInterval, 200

FileEncoding, UTF-8

; 開關
global CapslockXConfigPath := "./CapsLockX-Config.ini"
global EnableSnoChordTyping := 0 ; 默认关
IniRead, EnableSnoChordTyping, %CapslockXConfigPath%, Plugins, EnableSnoChordTyping, %EnableSnoChordTyping%
IniWrite, %EnableSnoChordTyping%, %CapslockXConfigPath%, Plugins, EnableSnoChordTyping
if(!EnableSnoChordTyping){
    Return
}

; 配置
global ChordIntervalThreshold := 32
global AllowRewriteString := "qwertasdfgzxcvbpyuiohjklnm"
global AllowRewrite := 0
global AppendSpace := 0
global StageList := []

; 变量
global PressedKeySet := {}
global TypedKeys := ""
global LastKeyDownTick := 0

; 读入配置
ConfigPath := "./雪星并击配置.ini"
IniRead, ChordIntervalThreshold, %ConfigPath%, Common, ChordIntervalThreshold, %ChordIntervalThreshold%
IniWrite, %ChordIntervalThreshold%, %ConfigPath%, Common, ChordIntervalThreshold
IniRead, AllowRewriteString, %ConfigPath%, Common, AllowRewriteString, %AllowRewriteString%
IniWrite, %AllowRewriteString%, %ConfigPath%, Common, AllowRewriteString
IniRead, AllowRewrite, %ConfigPath%, Common, AllowRewrite, %AllowRewrite%
IniWrite, %AllowRewrite%, %ConfigPath%, Common, AllowRewrite
IniRead, AppendSpace, %ConfigPath%, Common, AppendSpace, %AppendSpace%
IniWrite, %AppendSpace%, %ConfigPath%, Common, AppendSpace
; MsgBox, , , AllowRewriteString: %AllowRewriteString%

RuleStage1 :="
(
=|         |         |         |         |         |
=|    -    |    -    |    -    |    -    |    -    |
=| 1  => 1 | 2  => 2 | 3  => 3 | 4  => 4 | 5  => 5 |
=| q  => q | w  => w | e  => e | r  => r | t  => t |
=| a  => a | s  => s | d  => d | f  => f | g  => g |
=| z  => z | x  => x | c  => c | v  => v | b  => b |
=| 12 => 0 | 13 => 9 | 23 => 8 | 24 => 7 | 34 => 6 |
=| 21 => 0 | 31 => 9 | 32 => 8 | 42 => 7 | 43 => 6 |
=| qw => p | qe => o | we => i | wr => u | er => y |
=| wq => p | eq => o | ew => i | rw => u | re => y |
=| as => ; | ad => l | sd => k | sf => j | df => h |
=| sa => ; | da => l | ds => k | fs => j | fd => h |
=| zx => / | zc => . | xc => , | xv => m | cv => n |
=| xz => / | cz => . | cx => , | vx => m | vc => n |

)"
RuleStage2 := "
(
=|         |         |         |         |         |
=|    -    |    -    |    -    |    -    |    -    |
=| 6  => 6 | 7  => 7 | 8  => 8 | 9  => 9 | 0  => 0 |
=| y  => y | u  => u | i  => i | o  => o | p  => p |
=| h  => h | j  => j | k  => k | l  => l | ;  => ; |
=| n  => n | m  => m | ,  => , | .  => . | /  => / |
=| 78 => 5 | 79 => 4 | 89 => 3 | 08 => 2 | 09 => 1 |
=| 87 => 5 | 97 => 4 | 98 => 3 | 80 => 2 | 90 => 1 |
=| ui => t | uo => r | io => e | pi => w | po => q |
=| iu => t | ou => r | oi => e | ip => w | op => q |
=| jk => g | jl => f | kl => d | ;k => s | ;l => a |
=| kj => g | lj => f | lk => d | k; => s | l; => a |
=| m, => b | m. => v | ,. => c | /, => x | /. => z |
=| ,m => b | .m => v | ., => c | ,/ => x | ./ => z |

)"
RuleStage3 := "
(
=|         |         |         |         |         |
=|    -    |    -    |    -    |    -    |    -    |
=| -  => - | =  => = | [  => [ | ]  => ] | \  => \ |
=| '  => ' | _  => _ |
)"

StageIndex := 1
while(1){
    IniRead, RuleStage, %ConfigPath%, RuleStage%StageIndex%
    ; MsgBox ,,, % "asdf" RuleStage "zxcv" StageIndex "asdf" !RuleStage
    If (!RuleStage)
        Break
    RuleStage%StageIndex% := RuleStage
    ; MsgBox, , , RuleStage: %RuleStage%
    
    objRule := {}
    FoundPos := 0
    while(FoundPos := RegExMatch(RuleStage, "O)\s(\S+)\s*?=>\s*?(\S+)\s", SubPat, FoundPos+1))
    {
        MapFrom := "" SubPat.Value(1)
        MapTo := SubPat.Value(2) ""

        ; MsgBox ,,, % SubPat.Value(1) "=" SubPat.Value(2)

        MapFrom := StrReplace(MapFrom, "_", " ")
        MapTo := StrReplace(MapTo, "_", " ")

        objRule[MapFrom] := MapTo
    }

    objKeys := {}
    lsKey := []
    RightHandKeysSet := []
    For Keys, _ in objRule{
        ; MsgBox ,,, % "Keys" Keys
        KeysList := StrSplit(Keys)
        Loop % KeysList.MaxIndex()
        {
            Key := KeysList[A_Index]
            Key := StrReplace(Key, "_", " ")
            ; MsgBox ,,, % Key
            objKeys[Key] := 1
        }
    }
    For Key, _ in objKeys{
        ; MsgBox ,,, % Key
        lsKey.Push(Key)
    }
    Stage := {"objKeys": objKeys, "lsKey": lsKey, "objRule": objRule, "Pressed": "", "Typed": ""}
    StageList.Push(Stage)
    StageIndex++
}

IniWrite, %RuleStage1%, %ConfigPath%, RuleStage1
IniWrite, %RuleStage2%, %ConfigPath%, RuleStage2
IniWrite, %RuleStage3%, %ConfigPath%, RuleStage3


; global PressedKeys := ""
Hotkey, if, (!CapsLockXMode)
For _, Stage in StageList{
    For _, KeyName in Stage["lsKey"]{
        KeyName := StrReplace(KeyName, " ", "Space")
        ; 只有字母直接按下会不导致输入法上屏
        if(AllowRewrite && InStr(AllowRewriteString, KeyName)){
            Hotkey, ~$%KeyName%, KeyDown
            Hotkey, ~$+%KeyName%, KeyDown
        }else{
            Hotkey, $%KeyName%, KeyDown
            Hotkey, $+%KeyName%, KeyDown
        }
        Hotkey, ~$*%KeyName% Up, KeyUp
    }
}

Hotkey, if,

Return

#If (!CapsLockXMode)
snochorded_output_recored_keys(){
    OutputKey := ""
    For _, Stage in StageList {
        StagePressed := Stage["Pressed"]
        Replaced := Stage["objRule"][StagePressed ""]
        OutputKey .= Replaced ? Replaced : StagePressed
        Stage["Pressed"] := ""
    }
    lenTyped := StrLen(TypedKeys)
    OutputLength := StrLen(OutputKey)
    OutputChanged := TypedKeys != OutputKey
    ; Clean
    LastKeyDownTick := 0
    TypedKeys := ""
    if(OutputChanged && OutputLength){
        ; OutputKey .= " "
        OutputKey := "{blind}" . OutputKey
        Loop, % lenTyped
            OutputKey := "{BackSpace}" . OutputKey
        ; send event is most stable
        SendEvent % OutputKey
    }
    If(AppendSpace){
        SendEvent % " "
    }
}

KeyDown:
    NowTick := A_TickCount
    ThisKey := A_ThisHotkey
    ThisKey := StrReplace(ThisKey, "~")
    ThisKey := StrReplace(ThisKey, "$")
    ThisKey := StrReplace(ThisKey, "+")
    ThisKey := StrReplace(ThisKey, "Space", " ")

    if( LastKeyDownTick == 0
    || NowTick - LastKeyDownTick <= ChordIntervalThreshold){
        if(SubStr(A_ThisHotkey, 1, 1)=="~"){
            TypedKeys .= ThisKey
        }
    }else{
        snochorded_output_recored_keys()
    }

    For StageIndex, Stage in StageList{
        if(Stage["objKeys"].HasKey(ThisKey)){
            Stage["Pressed"] .= ThisKey
            Break
        }
    }
    LastKeyDownTick := NowTick
Return

KeyUp:
    snochorded_output_recored_keys()
    ; PressedKeys is only for debug
    ; If(PressedKeys)
    ;     ToolTip % TypedKeys " | " PressedKeys "(" lenTyped ")" " => " OutputKey "("  OutputLength ")"
    ; PressedKeys := ""
Return

#If !CapslockX
F12:: ExitApp