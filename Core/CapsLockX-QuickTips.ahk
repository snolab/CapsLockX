
Return

; 新场景提示
SceneTips(){
    ; TODO 自动提示适用于场景的新热键
}

; 用户触发的
QuickHelp(){
    msg := ""

    if (多任务窗口切换界面内()) {
    

    }

    if (任务栏中()) {
        msg .= "|	Delete	|	任务栏中关闭窗口	|"
    }

    if (Blender窗口内()) {

    }

    if (AnkiAddWindowActiveQ()) {

    }

    if (OneNote2016搜索界面内()) {

    }

    if (OneNote2016创建链接窗口内()) {
        msg .= "|	Alt + Shift + s	|	笔记条目搜索结果复制整理向页面粘贴条数	|"
        msg .= "|	Alt + s	|	笔记条目搜索结果复制整理条数	|"
    }

    if (OneNote2016笔记编辑窗口内()) {
        msg .= "|	Ctrl + Shift + PgUp	|	上一个页面切换	|"
        msg .= "|	Ctrl + Shift + PgDn	|	下一个页面切换	|"
        msg .= "|	Shift + Alt + n	|	切换为无格子背景	|"
        msg .= "|	Alt + n	|	切换为无色背景	|"
        msg .= "|	Ctrl + s	|	同步此笔记本	|"
        msg .= "|	Alt + a	|	增加空白	|"
        msg .= "|	Alt + 1	|	大纲折叠展开到1	|"
        msg .= "|	Alt + 2	|	大纲折叠展开到2	|"
        msg .= "|	Alt + 3	|	大纲折叠展开到3	|"
        msg .= "|	Alt + 4	|	大纲折叠展开到4	|"
        msg .= "|	Alt + 5	|	大纲折叠展开到5	|"
        msg .= "|	Alt + 6	|	大纲折叠展开到6	|"
        msg .= "|	Alt + 7	|	大纲折叠展开到7	|"
        msg .= "|	Alt + w	|	套锁	|"
        msg .= "|	Alt + k	|	当前关键词相关页面链接展开	|"
        msg .= "|	Ctrl + w	|	快速关闭窗口	|"
        msg .= "|	Alt + Shift + Delete	|	快速删除当前分区（并要求确认）	|"
        msg .= "|	Shift + Delete	|	快速删除当前行	|"
        msg .= "|	Alt + Delete	|	快速删除当前页面	|"
        msg .= "|	Alt + Shift + k	|	快速将内容做成单独链接	|"
        msg .= "|	Alt + d	|	打开换笔盘，定位到第一支笔	|"
        msg .= "|	Alt + q	|	拖动	|"
        msg .= "|	Alt + f	|	搜索标记	|"
        msg .= "|	Alt + e	|	橡皮	|"
        msg .= "|	Alt + Shift + p	|	段落链接复制	|"
        msg .= "|	Alt + Shift + m	|	移动分区	|"
        msg .= "|	Alt + m	|	移动笔记	|"
        msg .= "|	Ctrl + Shift + c	|	纯文本复制	|"
        msg .= "|	Ctrl + Shift + v	|	纯文本粘贴	|"
        msg .= "|	Alt + -	|	自动2维化公式	|"
        msg .= "|	Alt + v	|	自定义颜色	|"
        msg .= "|	Alt + ]	|	调整缩放-	|"
        msg .= "|	Alt + [	|	调整缩放+	|"
        msg .= "|	Alt + \	|	调整缩放复原	|"
        msg .= "|	Alt + s	|	输入	|"
        msg .= "|	Ctrl + d	|	选中当前词（目前来说会带上词右边的空格）	|"
        msg .= "|	Ctrl + Shift + l	|	选中行	|"
        msg .= "|	Shift + F2	|	重命名分区	|"
        msg .= "|	F2	|	重命名笔记	|"
        msg .= "|	Enter	|	链接安全警告自动确认	|"
        msg .= "|	Alt + F2	|	页面链接复制	|"
    }

    if (OneNote2016换笔中()) {
        msg .= "|	1	|	向第1行第1支笔切换	|"
        msg .= "|	2	|	向第1行第2支笔切换	|"
        msg .= "|	3	|	向第1行第3支笔切换	|"
        msg .= "|	4	|	向第1行第4支笔切换	|"
        msg .= "|	5	|	向第1行第5支笔切换	|"
        msg .= "|	6	|	向第1行第6支笔切换	|"
        msg .= "|	7	|	向第1行第7支笔切换	|"
        msg .= "|	Shift + 1	|	向第2行第1支笔切换	|"
        msg .= "|	Shift + 2	|	向第2行第2支笔切换	|"
        msg .= "|	Shift + 3	|	向第2行第3支笔切换	|"
        msg .= "|	Shift + 4	|	向第2行第4支笔切换	|"
        msg .= "|	Shift + 5	|	向第2行第5支笔切换	|"
        msg .= "|	Shift + 6	|	向第2行第6支笔切换	|"
        msg .= "|	Shift + 7	|	向第2行第7支笔切换	|"
    }

    if (名为剪贴板的OneNote窗口存在()) {
        msg .= "|	Ctrl + c	|	OneNote剪贴板收集	|"
    }
    ToolTip %msg%
}

热键提示显示(){
    QuickTips()
    KeyWait /
    ToolTip
}

#if CapsLockXMode

/:: 热键提示显示()