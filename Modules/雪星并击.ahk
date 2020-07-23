; 雪星并击 | snochorded
; Copyright 2020 snomiao@gmail.com
; LICENSE - GPLv3
; (20200722) 创建
; (20200723)

#MaxHotkeysPerInterval, 200
; 雪星并击规则集与分手按键表 {{
global LeftHandRule := {"1":"1","2":"2","3":"3","4":"4","5":"5","12":"0","13":"9","21":"0","23":"8","24":"7","31":"9","32":"8","34":"6","42":"7","43":"6","q":"q","w":"w","e":"e","r":"r","t":"t","qw":"p","wq":"p","qe":"o","eq":"o","we":"i","ew":"i","wr":"u","rw":"u","er":"y","re":"y","a":"a","s":"s","d":"d","f":"f","g":"g","as":";","sa":";","ad":"l","da":"l","sd":"k","ds":"k","sf":"j","fs":"j","df":"h","fd":"h","z":"z","x":"x","c":"c","v":"v","b":"b","zx":"/","xz":"/","zc":".","cz":".","xc":",","cx":",","xv":"m","vx":"m","cv":"n","vc":"n"}
global RightHandRule := {"0":"0","6":"6","7":"7","8":"8","9":"9","78":"5","79":"4","80":"2","87":"5","89":"3","90":"1","97":"4","98":"3","09":"1","08":"2","p":"p","y":"y","u":"u","i":"i","o":"o","ui":"t","iu":"t","uo":"r","ou":"r","io":"e","oi":"e","po":"q","op":"q","pi":"w","ip":"w",";":";","h":"h","j":"j","k":"k","l":"l","jk":"g","kj":"g","jl":"f","lj":"f","kl":"d","lk":"d",";l":"a","l;":"a",";k":"s","k;":"s","/":"/","n":"n","m":"m",",":",",".":".","m,":"b",",m":"b","m.":"v",".m":"v",",.":"c",".,":"c","/.":"z","./":"z","/,":"x",",/":"x"}
global LeftHandKeys := ["1","2","3","4","5","q","w","e","r","t","a","s","d","f","g","z","x","c","v","b"]
global RightHandKeys := ["0","6","7","8","9","p","y","u","i","o",";","h","j","k","l","/","n","m",",","."]
; }} /* 此区块由 index.js 自动生成。 */

global OtherKeys := "/.;"
global PressedKeySet := {}
global TypedKeys := ""
global LeftPressedKey := ""
global RightPressedKey := ""
global SpacePressed := 0

Hotkey, if, !CapsLockXMode

For _, KeyName in LeftHandKeys{
    Hotkey, $%KeyName%, LeftKeyDown
    Hotkey, $%KeyName% Up, KeyUp
}
For _, KeyName in RightHandKeys{
    Hotkey, $%KeyName%, RightKeyDown
    Hotkey, $%KeyName% Up, KeyUp
}
Hotkey, $Space, OtherKeyDown
Hotkey, $Space Up, KeyUp

Hotkey, if,


Return

#If !CapsLockXMode

LeftKeyDown:
StringRight, ThisKey, A_ThisHotkey, 1
if(StrLen(A_ThisHotkey)==3){
    TypedKeys .= ThisKey
}
LeftPressedKey .= ThisKey
Return

RightKeyDown:
StringRight, ThisKey, A_ThisHotkey, 1
if(StrLen(A_ThisHotkey)==3){
    TypedKeys .= ThisKey
}
RightPressedKey .= ThisKey
Return

OtherKeyDown:
if(A_ThisHotkey=="$Space"){
    SpacePressed := 1
}
Return

KeyUp:
    OutputKey := ""
    if(LeftHandRule[LeftPressedKey ""]){
        OutputKey .= LeftHandRule[LeftPressedKey ""]
    }else{
        OutputKey .= LeftPressedKey
    }
    if(RightHandRule[RightPressedKey ""]){
        OutputKey .= RightHandRule[RightPressedKey ""]
    }else{
        OutputKey .= RightPressedKey
    }
    PressedLength := StrLen(TypedKeys)
    OutputLength := StrLen(OutputKey)
    ; ToolTip % LeftPressedKey " " RightPressedKey " " OtherPressedKeys "(" PressedLength ")" " => " OutputKey "("  OutputLength ")"
    OutputChanged := TypedKeys != OutputKey
    ; Clean
    TypedKeys := LeftPressedKey := RightPressedKey := ""
    if(OutputChanged && OutputLength){
        if(SpacePressed){
            OutputKey .= " "
        }
        Loop, % PressedLength
            OutputKey := "{BackSpace}" . OutputKey
        SendEvent %OutputKey%
    }else{
        if(SpacePressed){
            SendEvent {Space}
        }
    }
    SpacePressed := 0
    OtherPressedKeyList := []
Return

#If (!CapslockX)
    F12:: ExitApp