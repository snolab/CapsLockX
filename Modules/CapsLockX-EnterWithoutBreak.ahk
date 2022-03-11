; ========== CapsLockX ==========
; 注：Save as UTF-8 with BOM please
; 名称：不折行回车
; 描述：不折行回车
; 作者：snomiao
; 支持：https://github.com/snolab/CapsLockX/issues/32
; ========== CapsLockX ==========

return

#if CapsLockXMode
    
; refs:
; https://github.com/snolab/CapsLockX/issues/32
; [[建议]增加backspace删除整行 · Issue #33 · snolab/CapsLockX]( https://github.com/snolab/CapsLockX/issues/33 )

; tested in vscode multiline mode
Enter:: Send {End 2}{Enter} ; 行尾回车
BackSpace:: Send {Home 2}{End}{Home 2}+{End 2}+{Right}{Delete} ; 删除整行
