; 雪星并击 | snochorded
; Copyright 2020 snomiao@gmail.com
; LICENSE - GPLv3
; (20200722) 创建
; Save this file as utf8 with bom
; update: https://github.com/snomiao/snochorded/raw/master/%E9%9B%AA%E6%98%9F%E5%B9%B6%E5%87%BB.ahk
#MaxHotkeysPerInterval, 200

FileEncoding, UTF-8

; 開關 默认关
global SnoChordTypingEnable := CapsLockX_Config("Plugins", "EnableSnoChordTyping", 0, "启用雪星并击（实验中），")
if (!SnoChordTypingEnable)
    Return

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
ConfigPath := "./雪星并击配置.ini"
IniRead, SnoChordTypingChordIntervalThreshold, %ConfigPath%, Common, SnoChordTypingChordIntervalThreshold, %SnoChordTypingChordIntervalThreshold%
IniWrite, %SnoChordTypingChordIntervalThreshold%, %ConfigPath%, Common, SnoChordTypingChordIntervalThreshold
IniRead, SnoChordTypingAllowRewriteString, %ConfigPath%, Common, SnoChordTypingAllowRewriteString, %SnoChordTypingAllowRewriteString%
IniWrite, %SnoChordTypingAllowRewriteString%, %ConfigPath%, Common, SnoChordTypingAllowRewriteString
IniRead, SnoChordTypingAllowRewrite, %ConfigPath%, Common, SnoChordTypingAllowRewrite, %SnoChordTypingAllowRewrite%
IniWrite, %SnoChordTypingAllowRewrite%, %ConfigPath%, Common, SnoChordTypingAllowRewrite
IniRead, SnoChordTypingAppendSpace, %ConfigPath%, Common, SnoChordTypingAppendSpace, %SnoChordTypingAppendSpace%
IniWrite, %SnoChordTypingAppendSpace%, %ConfigPath%, Common, SnoChordTypingAppendSpace
; MsgBox, , , SnoChordTypingAllowRewriteString: %SnoChordTypingAllowRewriteString%

RuleStage1 :="
(
=| | | | | |
=| - | - | - | - | - |
=| 1 => 1 | 2 => 2 | 3 => 3 | 4 => 4 | 5 => 5 |
=| q => q | w => w | e => e | r => r | t => t |
=| a => a | s => s | d => d | f => f | g => g |
=| z => z | x => x | c => c | v => v | b => b |
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
=| | | | | |
=| - | - | - | - | - |
=| 6 => 6 | 7 => 7 | 8 => 8 | 9 => 9 | 0 => 0 |
=| y => y | u => u | i => i | o => o | p => p |
=| h => h | j => j | k => k | l => l | ;  => ; |
=| n => n | m => m | , => , | . => . | / => / |
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
=| | | | | |
=| - | - | - | - | - |
=| - => - | = => = | [ => [ | ] => ] | \ => \ |
=| ' => ' | _ => _ |
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
    SnoChordTypingStageList.Push(Stage)
    StageIndex++
}

IniWrite, %RuleStage1%, %ConfigPath%, RuleStage1
IniWrite, %RuleStage2%, %ConfigPath%, RuleStage2
IniWrite, %RuleStage3%, %ConfigPath%, RuleStage3

Hotkey, if, (!CapsLockXMode)
For _, Stage in SnoChordTypingStageList{
    For _, KeyName in Stage["lsKey"]{
        KeyName := StrReplace(KeyName, " ", "Space")
        ; 只有字母直接按下会不导致输入法上屏
        if (SnoChordTypingAllowRewrite && InStr(SnoChordTypingAllowRewriteString, KeyName)){
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
    For _, Stage in SnoChordTypingStageList {
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
    if (OutputChanged && OutputLength){
        ; OutputKey .= " "
        OutputKey := "{blind}" . OutputKey
        Loop, % lenTyped
            OutputKey := "{BackSpace}" . OutputKey
        ; send event is most stable
        SendEvent % OutputKey
    }
    if (SnoChordTypingAppendSpace){
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

    if ( SnoChordTypingLastKeyDownTick == 0
    || NowTick - SnoChordTypingLastKeyDownTick <= SnoChordTypingChordIntervalThreshold){
        if (SubStr(A_ThisHotkey, 1, 1)=="~"){
            SnoChordTypingTypedKeys .= ThisKey
        }
    }else{
        snochorded_output_recored_keys()
    }

    For StageIndex, Stage in SnoChordTypingStageList{
        if (Stage["objKeys"].HasKey(ThisKey)){
            Stage["Pressed"] .= ThisKey
            Break
        }
    }
    SnoChordTypingLastKeyDownTick := NowTick
Return

KeyUp:
    snochorded_output_recored_keys()
    ; PressedKeys is only for debug
    ; if (PressedKeys)
    ;     ToolTip % SnoChordTypingTypedKeys " | " PressedKeys "(" lenTyped ")" " => " OutputKey "("  OutputLength ")"
    ; PressedKeys := ""
Return
