; ========== CapsLockX ==========
; 名称：雪星转屏
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v1.0.0
; ========== CapsLockX ==========

Return

#If CapsLockXMode

!Up:: CapsLockX_RunSilent(CapsLockX_模块路径 "/雪星转屏.exe =0")
!Left:: CapsLockX_RunSilent(CapsLockX_模块路径 "/雪星转屏.exe =90")
!Down:: CapsLockX_RunSilent(CapsLockX_模块路径 "/雪星转屏.exe =180")
!Right:: CapsLockX_RunSilent(CapsLockX_模块路径 "/雪星转屏.exe =270")
