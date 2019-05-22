Return
#h::
    Process, Exist, iFlyVoice.exe
    if(ErrorLevel){
        Send ^+h
        ; Process, Close, process.exe
    }Else{
        Run "C:\Program Files (x86)\iFly Info Tek\iFlyIME\2.1.1708\iFlyVoice.exe"
    }
    Return
