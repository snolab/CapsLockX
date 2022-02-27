
global 环境热键提示 := 0

return

#if CapsLockXMode
    
/:: 环境热键提示切换()

环境热键提示切换()
{
    环境热键提示 := !环境热键提示
    if ( 环境热键提示) {
        SceneTips()
        SetTimer, SceneTips, 1000
    } else {
        SetTimer, SceneTips, off
    }
    KeyWait /
    ToolTip
}

; 新场景提示
; TODO 自动提示适用于场景的新热键
; 统计qt次数，优先显示显示次数少的
SceneTipsRemove()
{
    ToolTip
}
SceneTips()
{
    static showMsg
    msg := "=== 环境热键提示 ===`n"
    if (CapsLockXMode) {
        msg .= "环境：CapsLockX 模式（或按住 CapsLockX ）"  "`n"
    } else {
        msg .= "环境：普通模式 "  "`n"
    }
    msg .= QuickTips()
    if (showMsg != msg) {
        showMsg := msg
        ToolTip %msg%
    }
}
CapsLockX_LongPressDown()
{
    SceneTips()
}
CapsLockX_LongPressUp()
{
    SceneTipsRemove()
}