; ========== CapsLockX ==========
; 名称：帮助模块及模块编写教程
; 描述：用于提供帮助函数，打开 CapsLockX 的 Github 页面，不可禁用
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
; 注释：在这一节里你可以填写你的脚本的元信息，本模块提供按住 CapsLockX + / 键显示帮助的功能。
; ========== CapsLockX ==========
;
; = 初始化区 =====================================================
; 注释：代码从这里开始运行
;
; 给出一些名称定义规范
; 应用：可以脱离 CapsLockX 独立运行的 AHK 文件
; 扩展：你的模块需要 CapsLockX 才能执行，且没有其它依赖
; 插件：你的模块需要 CapsLockX 才能执行，且没有其它依赖
; 用户：其它用户分享的模块
; 名称约束：模块名内不能有这几个字符 ", ``"
;
; 以下条件语句表示这个模块只能在 CapsLockX 下工作，如果没有用到 CapsLockX 的变量则可以不写。

if (!CapsLockX) {
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}
;
; 你可以在这里定义一些变量
; 可以是全局变量或者本地变量（建议全局）
; 需要注意模块按照文件名排序先后加载，
; 所以后一个模块可以读取前一个模块定义的变量（包括全局和本地的）（但通常不建议这么做）。
;
global CapsLockX_HelpInfo := ""
CapsLockX_IssuesPage := "https://github.com/snolab/CapsLockX/issues"

; 注释：在这里，你可以使用 CapsLockX_AppendHelp 添加帮助信息
; 在 AHK 中，所有的函数都在编译时就定义好了，声明顺序是无所谓的。
; CapsLockX_THIS_MODULE_HELP_FILE_PATH 在当前模块中的值为 "./Modules/00-Help.md"
CapsLockX_AppendHelp( CapsLockX_LoadHelpFrom(CapsLockX_THIS_MODULE_HELP_FILE_PATH))
;
; 初始化完成之后就可以返回了, 在这个 Return 之后，可以定义函数和热键
; 注：CapsLockX 模块【必须】 Return，才能顺利地执行后面的模块。
Return
;
; = 函数声名和热键区 =====================================================
;
; 定义函数，这里定义了 2 个用来操作帮助的函数。
CapsLockX_LoadHelpFrom(file)
{
    FileEncoding UTF-8
    FileRead, helpStr, %file%
    helpStr := RegExReplace(helpStr, "m)^[^|#].*$")
    helpStr := RegExReplace(helpStr, "m)\r?\n(\r?\n)+", "`n")
    return helpStr
}
CapsLockX_AppendHelp(helpStr)
{
    if (helpStr) {
        CapsLockX_HelpInfo .= helpStr "`n`n"
    }
}
CapsLockX_ShowHelp(helpStr, inGlobal := 0, waitKey := "/")
{
    if (!inGlobal && !CapsLockXMode) {
        SendEvent, /
        Return
    }
    Gui, Help:Destroy
    Gui, Help:Font, , SimHei
    Gui, Help:Add, Edit, ReadOnly, ==== CapsLockX Help ====
    Gui, Help:Add, Edit, H768 ReadOnly, %helpStr%
    Gui, Help:Show, AutoSize Center
    
    KeyWait, %waitKey%, T60 ; wait for 60 seconds, then auto close
    ; Gui, Hide
    Gui, Help:Destroy
    ; ToolTip
}

; 你可以以不同的模式添加各种热键
;
; 比如这一行，指的是当前在 CapsLockX 模式时，生效的热键
#if CapsLockXMode
    
; #if CapsLockXMode
; 显示使用方法，直接调用前面定义的函数
; /:: CapsLockX_ShowHelp(CapsLockX_HelpInfo, 1)

; 你可以按住 CapsLockX 键观察托盘的 CapsLockX 图标，当它变蓝时，按下 Alt + / 就可以快速打开 CapsLockX 的首页
; 也就是 CapsLockX + Alt + /
!/:: Run https://capslockx.snomiao.com/

; 同理，这个热键可以使用 CapsLockX + Shift + / 触发
+/:: Run % CapsLockX_IssuesPage

#if
    
; 在这里你也可以定义无需按下 CapsLockX 就能触发的热键
