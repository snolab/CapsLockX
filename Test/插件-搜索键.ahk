Return

; Google 搜索
Search(q){
    Run, https://www.google.com/search?q=%q%
}
TryCopy(retry = 0){
    Clipboard =
    Send, ^c
    ClipWait, 0.1, 1
    if (ErrorLevel && !retry){
        Send {Click 2}
        Return TryCopy(retry+1)
    }Else{
        Return Clipboard
    }
}
Search2(){
    clip := TryCopy()
    if (clip)
        Search(clip)
}

#If CapsLockXMode == CM_CapsLockX || CapsLockXMode == CM_FN
    ; g:: Send ^c^c
    g:: Search2()