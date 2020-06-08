# CapslockX

CapslockX 是 Windows 下的一套快捷键操作脚本，提供便捷的窗口管理、鼠标模拟、应用内热键增强等功能，基于AHK实现。

## 入门教程第一

下载源码 zip 解压
双击 `启动 CapsX (别名).lnk` 即可启动脚本

启动脚本后, 尝试:
按下`Alt`, 按住`Tab`, 然后按 `WASD` 切换选中窗口, 再按 `C` 关闭窗口，

> 某些功能需要使用管理员身份运行（如鼠标模拟）
> 在 CapslockX-Settings.ahk 中
> 手动修改 global T_AskRunAsAdmin := 1 即可
> 如不需要，可以改为 0

## 使用手册第二

按 `CapsLock` 切换 CapslockX 模式
按住 `CapsLock` 使用临时 CapslockX 模式

## 按键表第三

每个模块可以单独禁用，请见 `CapslockX-Settings.ahk` 文件

<!-- 下面这堆东西是自动从各个模块里抽取的，如需改动请到模块里操作, 在这里修改会被覆盖 -->
<!-- 开始：抽取模块帮助 -->
<!-- 模块文件名： 01-插件-模拟鼠标.ahk -->

### 模拟鼠标模块

| 作用窗口 | 模拟鼠标模块 | 说明 |
| - | :-: | - |
| 全局 | CapslockX + w a s d   | 鼠标平滑移动（上下左右） |
| 全局 | CapslockX + r f       | 垂直平滑滚轮（在chrome下有时失灵，原因未明) |
| 全局 | CapslockX + R F       | 水平平滑滚轮（在chrome下有时失灵，原因未明) |
| 全局 | CapslockX + rf        | r f 一起按相当于鼠标中键 |
| 全局 | CapslockX + e         | 鼠标左键 |
| 全局 | CapslockX + q         | 鼠标右键 |


<!-- 模块文件名：02-插件-窗口增强.ahk-->

### 窗口增强模块

效果如图
![02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif](https://github.com/snomiao/capslockx/raw/master/模块/02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif)
<!-- ![02-插件-窗口增强_一键排列窗口.gif](https://github.com/snomiao/capslockx/raw/master/模块/02-插件-窗口增强_一键排列窗口.gif) -->

| 作用域 | 窗口增强模块   | 说明 |
| - | :-: | - |
| 全局 | CapslockX + \ | 打开多桌面视图 |
| 全局 | CapslockX + [ ] | 切换到上一个/下一个桌面 |
| 全局 | CapslockX + Alt + [ ] | 把当前窗口移到上一个/下一个桌面 |
| 全局 | CapslockX + = | 新建桌面 |
| 全局 | CapslockX + - | 合并当前桌面 |
| 全局 | CapslockX + 0 | 新建桌面，并把当前窗口移过去 |
| 全局 | CapslockX + 1 2 ... 9 | 把当前窗口移到第1...9个桌面(如果有的话) |
| Alt+Tab界面 | w a s d | 上下左右切换窗口选择 |
| Alt+Tab界面 | x c     | 关闭选择的窗口 |
| Alt+Tab界面 | q e     | 左右切换多桌面 |
| Win+Tab视图 | w a s d | 切换窗口选择 |
| Win+Tab视图 | x | 关掉选择的窗口 |
| Win+Tab视图 | q e | 左右切换桌面概览 |
| Win+Tab视图 | z | 合并当前桌面与上一个桌面 |
| Win+Tab视图 | 0 | 新建桌面 |
| Win+Tab视图 | 1 2 ... 9 | 把窗口移到除了自己的第x个桌面（或新建桌面） |
| Win+Tab视图 | v | 新建桌面，并把当前窗口扔到新建桌面 |
| Win+Tab视图 | c | 新建桌面，并把当前窗口扔到新建桌面，并激活窗口 |
| Win+Tab视图 | [ ] | 切换到上一个/下一个桌面 |

<!-- [#ahk_class MultitaskingViewFrame](#ahk_class+MultitaskingViewFrame) -->

<!-- 模块文件名：03-应用-Anki增强.ahk-->

### Anki增强模块

| 模式 | Anki增强模块   | 说明 |
| - | :-: | - |
| 在Anki-学习界面 | w 或 k 或 ↑ | 按下=撤销，松开显示答案 |
| 在Anki-学习界面 | a 或 h 或 ← | 按下=顺利，松开显示答案 |
| 在Anki-学习界面 | s 或 j 或 ↓ | 按下=困难，松开显示答案 |
| 在Anki-学习界面 | d 或 l 或 → | 按下=生疏，松开显示答案 |
| 在Anki-学习界面 | q | 返回上个界面 |
| 在Anki-学习界面 | c | 添加新卡片 |
| 在Anki-学习界面 | 1 或 NumPad1 | 困难（原键位不动） |
| 在Anki-学习界面 | 2 或 NumPad2 | 生疏（原键位不动） |
| 在Anki-学习界面 | 3 或 NumPad3 | 一般（原键位不动） |
| 在Anki-学习界面 | 4 或 NumPad4 | 顺利（原键位不动） |
| 在Anki-学习界面 | 5 或 NumPad5 | 撤销 |
| 在Anki-学习界面 | 6 或 NumPad6 | 暂停卡片 |
| 在Anki-学习界面 | Alt + i | 快速导入剪贴版的内容（按 Tab 分割） / 比如可以从Excel复制 |
| 在Anki-添加卡片界面 | Alt + s | 按下 "添加" 按钮 |

>
> 此插件可配合手柄使用，请见 bilibili [中二雪星怎背词 - 手柄怎么可以不用来背单词！](https://www.bilibili.com/video/av8456838/)
>

<!-- 模块文件名：功能-秒打时间戳.ahk-->

### 秒打时间戳模块

| 模式 | 秒打时间戳   | 说明 |
| - | :-: | - |
| 全局 | [d 或 (d | 插入日期, 类似 (20190115) 这样的时间戳 |
| 全局 | [t 或 (t | 插入时间, 类似 (20190115.164744) 这样的时间戳 |
| 全局 | [s 或 (s | 插入时间, 类似 (1647) 这样的时间戳 |
| 全局 | [v | 插入版本号, 类似 v2019.01.15 这样的版本号 |

<!-- 模块文件名：应用-Edge增强.ahk-->

### Edge增强模块

模块测试中

| 模式 | 按键  | 说明 |
| - | :-: | - |
| 在Edge内 | Alt + w | 拿出笔（全屏模式暂时不支持）|
| 在Edge内 | Alt + q | 换左边的笔/橡皮（全屏模式暂时不支持） |
| 在Edge内 | Alt + e | 换右边的笔/橡皮（全屏模式暂时不支持） |
| 在Edge内 | Alt + , | 上一章/节 |
| 在Edge内 | Alt + . | 下一章/节 |
| 在Edge内 | Alt + / | 显示目录 |
| 在Edge内 | Alt + ; | 切换自适应页面大小模式 |
| 在Edge内 | Alt + ' | 切换双页布局模式 |

<!-- 模块文件名：应用-mstsc远程桌面增强.ahk-->

### mstsc远程桌面增强模块

| 模式 | 按键| 功能说明 |
| - | :-: | - |
| 在远程桌面窗口中 | 无需按键自动触发 | 自动置底：如果当前操作的远程桌面窗口是最大化的窗口，则自动置底，这样可以跟当前电脑桌面上的窗口共同操作 |
| 在远程桌面窗口中 | RAlt + RCtrl | 切换焦点：右边的 Alt+Ctrl 一起按可以切换焦点在不在远程桌面窗口 |
| 在远程桌面窗口中 | LAlt + RAlt | 最小化当前远程桌面窗口：回到系统操作界面 |

<!-- 模块文件名：应用-TIM添加常驻功能.ahk-->

### TIM添加常驻功能模块

|模式|按键|功能|
| - | :-: | - |
| 在Tim窗口内 |Alt + f| 焦点定位到左上角搜索框|
| 在Tim窗口内 |Ctrl + PgUp| 切换上一个窗口|
| 在Tim窗口内 |Ctrl + PgDn| 切换下一个窗口|

<!-- 模块文件名：应用-讯飞输入法语音悬浮窗.ahk-->

### 讯飞输入法语音悬浮窗模块
效果如下

| 模式 | 按键| 功能说明 |
| - | :-: | - |
| 任意 | Win + H | 启动/切换讯飞语音输入 |
*注1：原 Win+H 的功能是 Windows 自带听写，安装本插件后会变成 Win+Shift+H
*注2：若没有安装讯飞语音则会询问是否下载安装

<!-- **需要先进行以下配置**
1. 在C盘默认目录安装[讯飞语音输入法 Windows 版](https://srf.xunfei.cn/)（如果安装到其它目录请自己改代码……）
2. 当需要语音输入东西的时候按 Win+H 呼出即可
3. Enjoy it. -->

![应用-讯飞语音输入法悬浮窗演示.gif](https://github.com/snomiao/CapslockX/raw/master/模块/应用-讯飞语音输入法悬浮窗演示.gif)


<!-- 模块文件名：插件-OneNote剪贴板收集器.ahk-->

### OneNote剪贴板收集器模块
### OneNote 剪贴板收集插件

使用方法：

1. OneNote 2016 打开一个窗口，标题写成这样 "剪贴板收集"。
2. 然后再用 Ctrl + C 复制东西的时候就会自动记到 OneNote 里
3. 如图
   ![插件-OneNote剪贴板收集器.gif](https://github.com/snomiao/capslockx/raw/master/模块/插件-OneNote剪贴板收集器.gif)

<!-- 模块文件名：插件-合并右Ctrl与Menu键.ahk-->

### 合并右Ctrl与Menu键模块

专治各种（Surface 的 右 Ctrl 键）残破键盘，合并 Menu与 右Ctrl键，Menu当Ctrl用 或者 Ctrl当Menu用都可以

| 模式 | 操作 | 说明 |
| - | :-: | - |
| 全局 | 右Ctrl按一下 | 会按一下 Menu 弹出菜单 |
| 全局 | 按住右Menu | 会按住 Ctrl，此时可以与其它键组合 |

<!-- 模块文件名：插件-媒体键.ahk-->

### 媒体键模块

| 模式 | 媒体键模块   | 说明 （这个模块基本还是半成品。。欢迎push） |
| - | :-: | - |
| CapslockX | F5   | 暂停播放 |
| CapslockX | F6   | 上一首 |
| CapslockX | F7   | 下一首 |
| CapslockX | F8   | 停止播放 |
| CapslockX | F10  | 静音 |
| CapslockX | F11  | 音量减 |
| CapslockX | F12  | 音量加 |

<!-- 模块文件名：插件-搜索键.ahk-->

### 搜索键模块

| 模式 | 搜索键模块   | 说明 （这个模块基本还是半成品。。欢迎push） |
| - | :-: | - |
| CapslockX | g    | 用 google 搜索当前选择或鼠标所指的词 |

<!-- 模块文件名：插件-编辑增强.ahk-->

### 编辑增强模块
| 作用 | Edit模块   | 说明 （欢迎push）|
| - | :-: | - |
| 全局 | CapslockX + z         | 回车（单纯是为了把回车放到左手……以便右手可以一直撑着下巴玩电脑） |
| 全局 | CapslockX + k j h l   | 上下左右 方向键 |
| 全局 | CapslockX + n m       | Home End |
| 全局 | CapslockX + n + m     | n m 一起按选择当前行 |
| 全局 | CapslockX + b         | BackSpace |
| 全局 | CapslockX + Shift + b | Delete |

<!-- 模块文件名：插件-雪星转屏.ahk-->

### 雪星转屏模块

| 模式 | 按键 | 功能 |
| - | :-: | - |
| CapslockX | Alt + 方向键 上 下 左 右 | 同时旋转所有屏幕到你指定的方向 |
| CapslockX | Alt + 小键盘 8 2 4 6 | 同时旋转所有屏幕到你指定的方向 |
<!-- 结束：抽取模块帮助 -->

## 发展路线第四

简化电脑操作逻辑，提升效率，尽量不与习惯键位冲突

po主偷懒中:

1. [ ] 长按CapslockX键显示对应帮助
2. 

## 制作背景第五

本人比较经常写代码……

起初我是右鼠……后来觉得鼠标放右边有点远……改成了左鼠

左鼠之后发现手还是要离开键盘……于是做了个WASD模拟鼠标的东西。

后来写的脚本越来越多，就把其中一些常用的放到一起加载……然后就成这样了

## 答疑相关第六

GitHub: [https://github.com/snomiao/CapslockX](https://github.com/snomiao/CapslockX)

本人常驻 QQ群： 271105729

关于这个脚本，相关答疑直接进群 @这个QQ 997596439 提问即可

## 性能指标

内存占用：约2.2M
