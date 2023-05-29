

Return

#if FigmaWindowActiveQ()

FigmaWindowActiveQ(){
    ; 作成中 📝 – Figma
    if(WinActive(".*– Figma.*")){
        return 1
    }
    return 0
}

; !h:: Send \+2
; !l:: Send {Enter}{Tab}+2
; !j:: Send {Tab}+2
; !k:: Send +{Tab}+2