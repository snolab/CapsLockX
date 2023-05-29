
if (!CapsLockX)
    ExitApp
Return

#IfWinActive Loopback Exemption Manager ahk_exe WindowsLoopbackManager.exe ; WindowsLoopbackManager 窗口内

f1:: ; 勾选 1 个
Loop 1 {
    Send {Tab}{Space}{Down}
}
f2:: ; 勾选 2 个
Loop 2 {
    Send {Tab}{Space}{Down}
}
Return
f3:: ; 勾选 4 个
Loop 4 {
    Send {Tab}{Space}{Down}
}
Return
f4:: ; 勾选 8 个
Loop 8 {
    Send {Tab}{Space}{Down}
}
Return
f5:: ; 勾选 16 个
Loop 16 {
    Send {Tab}{Space}{Down}
}
Return
f6:: ; 勾选 32 个
Loop 32 {
    Send {Tab}{Space}{Down}
}
Return
f7:: ; 勾选 64 个
Loop 64 {
    Send {Tab}{Space}{Down}
}
Return
f8:: ; 勾选 128 个
Loop 128 {
    Send {Tab}{Space}{Down}
}
Return
