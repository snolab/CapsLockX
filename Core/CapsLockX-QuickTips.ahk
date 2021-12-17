
Return
 
; 用户触发的
QuickTips(){
    msg := ""

    try{
        if (CapsLockXMode) {
            msg .= "|	/	|	环境热键提示	|`n"
            msg .= "|	b	|	任务栏任务切换	|`n"
            msg .= "|	c	|	自动排列窗口	|`n"
            msg .= "|	v	|	当前窗口临时透明	|`n"
            msg .= "|	x	|	关闭标签	|`n"
            msg .= "|	z	|	最近1分钟内闪动窗口激活	|`n"
            msg .= "|	Ctrl + c	|	自动排列窗口（包括最小化的窗口）	|`n"
            msg .= "|	Shift + c	|	自动堆叠窗口	|`n"
            msg .= "|	Shift + v	|	当前窗口置顶透明切换	|`n"
            msg .= "|	Shift + x	|	关闭窗口并切到下一窗口	|`n"
            msg .= "|	Shift + z	|	下一个窗口激活	|`n"
            msg .= "|	Ctrl + Alt + x	|	杀死窗口并切到下一窗口	|`n"
            msg .= "|	Shift + Ctrl + c	|	自动堆叠窗口（包括最小化的窗口）	|`n"
            msg .= "|	\	|	CapsLockX_模块重载	|`n"
            msg .= "|	Shift + \	|	CapsLockX_Reload	|`n"
            msg .= "|	Shift + \	|	CapsLockX_重新启动	|`n"
            msg .= "|	Shift + Ctrl + \	|	CapsLockX_退出	|`n"
            msg .= "|	m	|	配置文件编辑	|`n"
        }
    }

    try{
        if (CapsLockXMode && 任务栏中()) {
            msg .= "|	x	|	任务栏中关闭窗口	|`n"
        }
    }

    try{
        if (Blender窗口内() && Blender增强模式) {
            msg .= "|	i + a	|	X全局缩放+	|`n"
            msg .= "|	i + d	|	X全局缩放-	|`n"
            msg .= "|	i + e	|	Z全局缩放-	|`n"
            msg .= "|	i + q	|	Z全局缩放+	|`n"
            msg .= "|	i + s	|	Y全局缩放+	|`n"
            msg .= "|	i + w	|	Y全局缩放-	|`n"
            msg .= "|	j + a	|	X本地平移+	|`n"
            msg .= "|	j + d	|	X本地平移-	|`n"
            msg .= "|	j + e	|	Z本地平移-	|`n"
            msg .= "|	j + q	|	Z本地平移+	|`n"
            msg .= "|	j + s	|	Y本地平移+	|`n"
            msg .= "|	j + w	|	Y本地平移-	|`n"
            msg .= "|	k + a	|	X本地缩放+	|`n"
            msg .= "|	k + d	|	X本地缩放-	|`n"
            msg .= "|	k + e	|	Z本地缩放-	|`n"
            msg .= "|	k + q	|	Z本地缩放+	|`n"
            msg .= "|	k + s	|	Y本地缩放+	|`n"
            msg .= "|	k + w	|	Y本地缩放-	|`n"
            msg .= "|	l + a	|	X本地旋转+	|`n"
            msg .= "|	l + d	|	X本地旋转-	|`n"
            msg .= "|	l + e	|	Z本地旋转-	|`n"
            msg .= "|	l + q	|	Z本地旋转+	|`n"
            msg .= "|	l + s	|	Y本地旋转+	|`n"
            msg .= "|	l + w	|	Y本地旋转-	|`n"
            msg .= "|	o + a	|	X全局旋转+	|`n"
            msg .= "|	o + d	|	X全局旋转-	|`n"
            msg .= "|	o + e	|	Z全局旋转-	|`n"
            msg .= "|	o + q	|	Z全局旋转+	|`n"
            msg .= "|	o + s	|	Y全局旋转+	|`n"
            msg .= "|	o + w	|	Y全局旋转-	|`n"
            msg .= "|	u + a	|	X全局平移+	|`n"
            msg .= "|	u + d	|	X全局平移-	|`n"
            msg .= "|	u + e	|	Z全局平移-	|`n"
            msg .= "|	u + q	|	Z全局平移+	|`n"
            msg .= "|	u + s	|	Y全局平移+	|`n"
            msg .= "|	u + w	|	Y全局平移-	|`n"
            msg .= "|	{os} + {ds}	|	${xyz[di/2|0]+lg[oi/3|0]+gsr[oi%3]+'+-'[di%2]}	|`n"
        }
    }

    try{
        if (WinActive(".*- Adobe Acrobat (Pro|Reader) DC ahk_class ahk_class AcrobatSDIWindow")) {
            msg .= "|	Ctrl + Alt + F12	|	退出脚本	|`n"
        }
    }

    try{
        if (!CapsLockXMode && (WinActive("Anki -.* ahk_class QWidget ahk_exe anki.exe") or WinActive("Anki - .*|.* - Anki ahk_class Qt5QWindowIcon ahk_exe anki.exe"))) {
            msg .= "|	c	|	create	|`n"
            msg .= "|	q	|	quit	|`n"
            msg .= "|	x	|	study	|`n"
            msg .= "|	Alt + i	|	AnkiImport	|`n"
        }
    }

    try{
        if ((!CapsLockX)) {
            msg .= "|	Shift + Ctrl + Alt + F12	|	退出脚本	|`n"
        }
    }

    try{
        if (WinActive(".*的资料 ahk_class TXGuiFoundation ahk_exe QQ.exe")) {
            msg .= "|	F2	|	改备注名	|`n"
            msg .= "|	F3	|	改分组	|`n"
            msg .= "|	F4	|	加备注（手机号等）	|`n"
        }
    }

    try{
        if (!CapsLockXMode) {
            msg .= "|	Win + h	|	讯飞语音输入法切换	|`n"
        }
    }

    try{
        if (!!(CapsLockXMode & CM_FN) || !!(CapsLockXMode & CM_CapsLockX)) {
            msg .= "|	F1	|	打开我的电脑	|`n"
            msg .= "|	F2	|	计算器	|`n"
            msg .= "|	F3	|	主页	|`n"
            msg .= "|	F4	|	启动播放器	|`n"
            msg .= "|	F5	|	暂停	|`n"
            msg .= "|	F6	|	上一首	|`n"
            msg .= "|	F7	|	下一首	|`n"
            msg .= "|	F8	|	停止	|`n"
            msg .= "|	F9	|	音量-	|`n"
            msg .= "|	F10	|	音量+	|`n"
            msg .= "|	F11	|	音量0	|`n"
            msg .= "|	F12	|	启动计算器	|`n"
        }
    }

    try{
        if (Func("多任务窗口切换界面内").Call()) {

            msg .= "|	Alt + 0	|	选中窗口移动到10号桌面	|`n"
            msg .= "|	Alt + 1	|	选中窗口移动到1号桌面	|`n"
            msg .= "|	Alt + 2	|	选中窗口移动到2号桌面	|`n"
            msg .= "|	Alt + 3	|	选中窗口移动到3号桌面	|`n"
            msg .= "|	Alt + 4	|	选中窗口移动到4号桌面	|`n"
            msg .= "|	Alt + 5	|	选中窗口移动到5号桌面	|`n"
            msg .= "|	Alt + 6	|	选中窗口移动到6号桌面	|`n"
            msg .= "|	Alt + 7	|	选中窗口移动到7号桌面	|`n"
            msg .= "|	Alt + 8	|	选中窗口移动到8号桌面	|`n"
            msg .= "|	Alt + 9	|	选中窗口移动到9号桌面	|`n"
            msg .= "|	Alt + a	|	左	|`n"
            msg .= "|	Alt + d	|	右	|`n"
            msg .= "|	Alt + e	|	向右切换多桌面	|`n"
            msg .= "|	Alt + f	|	音量-	|`n"
            msg .= "|	Alt + q	|	向左切换多桌面	|`n"
            msg .= "|	Alt + r	|	音量+	|`n"
            msg .= "|	Alt + s	|	下	|`n"
            msg .= "|	Alt + w	|	上	|`n"
            msg .= "|	Alt + x	|	关闭应用	|`n"
        }
    }

    try{
        if (Func("任务栏中").Call()) {

            msg .= "|	Delete	|	任务栏中关闭窗口	|`n"
        }
    }

    try{
        if (Func("OneNote2016创建链接窗口内").Call()) {

            msg .= "|	Alt + s	|	笔记条目搜索结果复制整理条数	|`n"
            msg .= "|	Alt + Shift + s	|	笔记条目搜索结果复制整理向页面粘贴条数	|`n"
        }
    }

    try{
        if (Func("OneNote2016笔记编辑窗口内").Call()) {

            msg .= "|	F2	|	重命名笔记	|`n"
            msg .= "|	F3	|	精确查找笔记	|`n"
            msg .= "|	Enter	|	链接安全警告自动确认	|`n"
            msg .= "|	Alt + -	|	自动2维化公式	|`n"
            msg .= "|	Alt + [	|	调整缩放+	|`n"
            msg .= "|	Alt + ]	|	调整缩放-	|`n"
            msg .= "|	Alt + \	|	调整缩放复原	|`n"
            msg .= "|	Alt + 1	|	大纲折叠展开到1	|`n"
            msg .= "|	Alt + 2	|	大纲折叠展开到2	|`n"
            msg .= "|	Alt + 3	|	大纲折叠展开到3	|`n"
            msg .= "|	Alt + 4	|	大纲折叠展开到4	|`n"
            msg .= "|	Alt + 5	|	大纲折叠展开到5	|`n"
            msg .= "|	Alt + 6	|	大纲折叠展开到6	|`n"
            msg .= "|	Alt + 7	|	大纲折叠展开到7	|`n"
            msg .= "|	Alt + a	|	增加空白	|`n"
            msg .= "|	Alt + d	|	打开换笔盘，定位到第一支笔	|`n"
            msg .= "|	Alt + e	|	橡皮	|`n"
            msg .= "|	Alt + f	|	搜索标记	|`n"
            msg .= "|	Alt + k	|	将当前关键词搜索到的相关页面链接在下方展开	|`n"
            msg .= "|	Alt + m	|	移动笔记	|`n"
            msg .= "|	Alt + n	|	切换为无色背景	|`n"
            msg .= "|	Alt + q	|	拖动	|`n"
            msg .= "|	Alt + s	|	输入	|`n"
            msg .= "|	Alt + t	|	把笔记时间显式填充到标题	|`n"
            msg .= "|	Alt + v	|	自定义颜色	|`n"
            msg .= "|	Alt + w	|	套锁	|`n"
            msg .= "|	Alt + F2	|	页面链接复制	|`n"
            msg .= "|	Ctrl + d	|	选中当前词（目前来说会带上词右边的空格）	|`n"
            msg .= "|	Ctrl + s	|	同步此笔记本	|`n"
            msg .= "|	Ctrl + w	|	快速关闭窗口	|`n"
            msg .= "|	Shift + F2	|	重命名分区	|`n"
            msg .= "|	Alt + Delete	|	快速删除当前页面	|`n"
            msg .= "|	Shift + Delete	|	快速删除当前行	|`n"
            msg .= "|	Alt + Shift + k	|	快速将内容做成单独链接	|`n"
            msg .= "|	Alt + Shift + m	|	移动分区	|`n"
            msg .= "|	Alt + Shift + p	|	段落链接复制	|`n"
            msg .= "|	Shift + Alt + n	|	切换为无格子背景	|`n"
            msg .= "|	Shift + Ctrl + c	|	纯文本复制	|`n"
            msg .= "|	Shift + Ctrl + l	|	选中行	|`n"
            msg .= "|	Shift + Ctrl + v	|	纯文本粘贴	|`n"
            msg .= "|	Shift + Ctrl + PgDn	|	下一个页面切换	|`n"
            msg .= "|	Alt + Shift + Delete	|	快速删除当前分区（并要求确认）	|`n"
        }
    }

    try{
        if (Func("OneNote2016换笔界面").Call()) {

            msg .= "|	1	|	向第1行第1支笔切换	|`n"
            msg .= "|	2	|	向第1行第2支笔切换	|`n"
            msg .= "|	3	|	向第1行第3支笔切换	|`n"
            msg .= "|	4	|	向第1行第4支笔切换	|`n"
            msg .= "|	5	|	向第1行第5支笔切换	|`n"
            msg .= "|	6	|	向第1行第6支笔切换	|`n"
            msg .= "|	7	|	向第1行第7支笔切换	|`n"
            msg .= "|	Shift + 1	|	向第2行第1支笔切换	|`n"
            msg .= "|	Shift + 2	|	向第2行第2支笔切换	|`n"
            msg .= "|	Shift + 3	|	向第2行第3支笔切换	|`n"
            msg .= "|	Shift + 4	|	向第2行第4支笔切换	|`n"
            msg .= "|	Shift + 5	|	向第2行第5支笔切换	|`n"
            msg .= "|	Shift + 6	|	向第2行第6支笔切换	|`n"
            msg .= "|	Shift + 7	|	向第2行第7支笔切换	|`n"
        }
    }

    try{
        if (Func("名为剪贴板的OneNote窗口存在").Call()) {

            msg .= "|	Ctrl + c	|	OneNote剪贴板收集	|`n"
        }
    }
    return msg
}
