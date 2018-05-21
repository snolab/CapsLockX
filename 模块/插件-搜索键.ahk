Return
#If CapsXMode == CM_CAPSX
    ; Google 搜索
    Search(q){
        Run, https://www.google.com/search?q=%q%
    }
    TryCopy(retry = 0){
        Clipboard =
        Send, ^c
        ClipWait, 0.1, 1
        If(ErrorLevel && !retry){
            Send {Click 2}
            Return TryCopy(retry+1)
        }Else{
            Return Clipboard
        }
    }
    Search2(){
        clip := TryCopy()
        If(clip)
            Search(clip)
    }
    ; 装有GoldenDict的，用g代替
    g:: Send ^c^c