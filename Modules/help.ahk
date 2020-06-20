; ========== CapslockX ==========
; 名称：帮助模块及模块编写教程
; 描述：打开 CapslockX 的 Github 页面
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapslockX
; 
; 注释：在这一节里你可以填写你的脚本的元信息，需要以 CapslockX 开头和结尾才能识别
; 
; ========== CapslockX ==========
; 
; 注释：代码从这里开始运行
; 
; 应用：可以脱离 CapslockX 独立运行的 AHK 文件
; 扩展：你的模块需要 CapslockX 才能执行，且不需要下载外部组件
; 插件：你的模块需要 CapslockX 才能执行，且不需要下载外部组件
; 
; 以下条件语句表示这个模块只能在 CapslockX 下工作
If(!CapslockX){
    MsgBox, % "本模块只在 CapslockX 下工作"
    ExitApp
}

; 
; 注释：在这里，你可以使用 CapslockAddHelp 添加帮助信息
; 
CapslockAddHelp("CapslockX + Alt + /    | 🔗 打开 CapslockX 的 README.md 页面")
CapslockAddHelp("CapslockX + Shift + /  | 🕷 提交bug、建议等 ")

; 
; 也可以定义一些变量（需要注意不要和其它模块冲突）
; 
CapslockX_IssuesPage := "https://github.com/snomiao/CapslockX/issues"

; 
; 初始化完成之后就可以返回了
; 
Return
; 
; 在这个 Return 下面，你可以以不同的模式添加各种热键
; 
; 比如这一行，指的是当前在 CapslockX 模式时，生效的热键
#If CapslockXMode

; 你可以按住 CapslockX 键观察托盘的 CapslockX 图标，当它变蓝时，按下 Alt + / 就可以快速打开 CapslockX 的首页
; 也就是 CapslockX + Alt + /
!/::
    WinGetActiveTitle, title
    Run https://github.com/snomiao/CapslockX#readme
Return

; 同理，这个热键可以使用 CapslockX + Ctrl + / 触发
^/::
    WinGetActiveTitle, title
    Run % CapslockX_IssuesPage
Return

