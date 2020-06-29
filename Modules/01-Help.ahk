; ========== CapsLockX ==========
; 名称：帮助模块及模块编写教程
; 描述：打开 CapsLockX 的 Github 页面
; 作者：snomiao
; 联系：snomiao@gmail.com
; 支持：https://github.com/snomiao/CapsLockX
; 版本：v2020.06.27
;
; 注释：在这一节里你可以填写你的脚本的元信息，需要以 CapsLockX 开头和结尾才能识别
;
; ========== CapsLockX ==========
; 
; 注释：代码从这里开始运行
; 
; 应用：可以脱离 CapsLockX 独立运行的 AHK 文件
; 扩展：你的模块需要 CapsLockX 才能执行，且不需要下载外部组件
; 插件：你的模块需要 CapsLockX 才能执行，且不需要下载外部组件
; 
; 以下条件语句表示这个模块只能在 CapsLockX 下工作
If(!CapsLockX){
    MsgBox, % "本模块只在 CapsLockX 下工作"
    ExitApp
}

; 
; 注释：在这里，你可以使用 CapslockXAddHelp 添加帮助信息
; 
CapslockXAddHelp("
(
= 显示帮助
| CapsLockX + Alt + /    | 🔗 打开 CapsLockX 的 README.md 页面
| CapsLockX + Shift + /  | 🕷 提交bug、建议等
)")

; 
; 也可以定义一些变量（需要注意不要和其它模块冲突）
; 
CapsLockX_IssuesPage := "https://github.com/snomiao/CapsLockX/issues"

; 
; 初始化完成之后就可以返回了
; 
Return
; 
; 在这个 Return 下面，你可以以不同的模式添加各种热键
; 
; 比如这一行，指的是当前在 CapsLockX 模式时，生效的热键
#If CapsLockXMode

; 你可以按住 CapsLockX 键观察托盘的 CapsLockX 图标，当它变蓝时，按下 Alt + / 就可以快速打开 CapsLockX 的首页
; 也就是 CapsLockX + Alt + /
!/::
    WinGetActiveTitle, title
    Run https://github.com/snomiao/CapsLockX#readme
Return

; 同理，这个热键可以使用 CapsLockX + Ctrl + / 触发
^/::
    WinGetActiveTitle, title
    Run % CapsLockX_IssuesPage
Return

