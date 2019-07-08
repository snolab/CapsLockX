# CapslockX

## 入门教程第一
<!-- 
首先，你需要安装一个 [AutoHotKey](https://autohotkey.com/)。
推荐使用的版本： `AutoHotKey_L Unicode x32` （鼠标模拟模块不支持64位） -->

<!-- 然后双击 `CapslockX.ahk` 来启动 CapslockX -->

双击 `启动 CapsX (别名).lnk` 即可启动脚本


> 某些功能需要使用管理员身份运行（如鼠标模拟）
> 
> 在 CapslockX-Settings.ahk 中
> 
> 手动修改 global T_AskRunAsAdmin := 1 即可
> 如不需要，可以改为 0
> 

## 使用手册第二

按 `CapsLock` 切换 CapslockX 模式
按住 `CapsLock` 使用临时 CapslockX 模式

## 按键表第三
每个模块可以单独禁用，请见 `CapslockX-Settings.ahk` 文件

<!-- 下面这堆东西是自动从各个模块里抽取的，如需改动请到模块里操作, 在这里修改会被覆盖 -->
<!-- 开始：抽取模块帮助 -->
<!-- 模块帮助文件名：01-插件-模拟鼠标.ahk-->
#### 模拟鼠标模块
| 模式 | 模拟鼠标模块 | 说明 |
| - | :-: | - |
| CapslockX | w a s d   | 鼠标平滑移动（上下左右） |
| CapslockX | r f       | 垂直平滑滚轮（在chrome下有时失灵，原因未明) |
| CapslockX | R F       | 水平平滑滚轮（在chrome下有时失灵，原因未明) |
| CapslockX | rf        | r f一起按相当于鼠标中键 |
| CapslockX | e         | 鼠标左键 |
| CapslockX | q         | 鼠标右键 |

<!-- 模块帮助文件名：02-插件-窗口增强.ahk-->
#### 窗口增强模块效果如图
![02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif](https://github.com/snomiao/capslockx/raw/master/模块/02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif)
![02-插件-窗口增强_一键排列窗口.gif](https://github.com/snomiao/capslockx/raw/master/模块/02-插件-窗口增强_一键排列窗口.gif)

| 模式 | 窗口增强模块   | 说明 |
| - | :-: | - |
| 全局 | CapslockX键 + \ | 打开多桌面视图 |
| 全局 | CapslockX键 + [ | 切换到上一个桌面 |
| 全局 | CapslockX键 + ] | 切换到下一个桌面 |
| 全局 | CapslockX键 + Alt + [ | 把当前窗口移到上一个桌面 |
| 全局 | CapslockX键 + Alt + ] | 把当前窗口移到下一个桌面 |
| 全局 | CapslockX键 + = | 新建桌面 |
| 全局 | CapslockX键 + - | 合并当前桌面 |
| 全局 | CapslockX键 + 0 | 新建桌面，并把当前窗口移过去 |
| 全局 | CapslockX键 + 1 | 把当前窗口移到第1个桌面(如果有的话) |
| 全局 | CapslockX键 + 2 | 把当前窗口移到第2个桌面(如果有的话) |
| 全局 | CapslockX键 + 3 | 把当前窗口移到第3个桌面(如果有的话) |
| 全局 | CapslockX键 + 4 | 把当前窗口移到第4个桌面(如果有的话) |
| 全局 | CapslockX键 + 5 | 把当前窗口移到第5个桌面(如果有的话) |
| 全局 | CapslockX键 + 6 | 把当前窗口移到第6个桌面(如果有的话) |
| 全局 | CapslockX键 + 7 | 把当前窗口移到第7个桌面(如果有的话) |
| 全局 | CapslockX键 + 8 | 把当前窗口移到第8个桌面(如果有的话) |
| 全局 | CapslockX键 + 9 | 把当前窗口移到第9个桌面(如果有的话) |
| Alt+Tab界面下 | w a s d | 上下左右切换窗口选择 |
| Alt+Tab界面下 | x c     | 关闭选择的窗口 |
| Alt+Tab界面下 | q e     | 左右切换多桌面 |
| 窗口靠边界面下 | w a s d | 切换窗口选择 |
| 窗口靠边界面下 | x | 关掉选择的窗口 |
| Win+Tab界面下 | w a s d | 切换窗口选择 |
| Win+Tab界面下 | q e | 左右切换桌面概览 |
| Win+Tab界面下 | z | 合并当前桌面与上一个桌面 |
| Win+Tab界面下 | x | 关掉选择的窗口 |
| Win+Tab界面下 | 1 2 3 ... 9 0 | 把窗口移到除了自己的第x个桌面（或新建桌面） |
| Win+Tab界面下 | v | 新建桌面，并把当前窗口扔到新建桌面 |
| Win+Tab界面下 | c | 新建桌面，并把当前窗口扔到新建桌面，并激活窗口 |
| Win+Tab界面下 | \ | 相当于按下 Tab |
| Win+Tab界面下 | [ | 切换到上一个桌面 |
| Win+Tab界面下 | ] | 切换到下一个桌面 |


<!-- 模块帮助文件名：03-应用-Anki增强.ahk-->
#### Anki增强模块
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

<!-- 模块帮助文件名：功能-秒打时间戳.ahk-->
#### 秒打时间戳模块
| 模式 | 秒打时间戳   | 说明 |
| - | :-: | - |
| 全局 | [d 或 (d | 插入日期, 类似 (20190115) 这样的时间戳 |
| 全局 | [t 或 (t | 插入时间, 类似 (20190115.164744) 这样的时间戳 |
| 全局 | [s 或 (s | 插入时间, 类似 (1647) 这样的时间戳 |
| 全局 | [v | 插入版本号, 类似 v2019.01.15 这样的版本号 |

<!-- 模块帮助文件名：应用-Edge增强.ahk-->
#### Edge增强模块
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

<!-- 模块帮助文件名：应用-mstsc远程桌面增强.ahk-->
#### mstsc远程桌面增强模块
| 模式 | 按键| 功能说明 |
| - | :-: | - |
| 在远程桌面窗口中 | 无需按键自动触发 | 自动置底：如果当前操作的远程桌面窗口是最大化的窗口，则自动置底，这样可以跟当前电脑桌面上的窗口共同操作 |
| 在远程桌面窗口中 | RAlt + RCtrl | 切换焦点：右边的 Alt+Ctrl 一起按可以切换焦点在不在远程桌面窗口 |
| 在远程桌面窗口中 | LAlt + RAlt | 最小化当前远程桌面窗口：回到系统操作界面 |

<!-- 模块帮助文件名：应用-TIM添加常驻功能.ahk-->
#### TIM添加常驻功能模块
|模式|按键|功能|
| - | :-: | - |
| 在Tim窗口内 |Alt + f| 焦点定位到左上角搜索框|
| 在Tim窗口内 |Ctrl + PgUp| 切换上一个窗口|
| 在Tim窗口内 |Ctrl + PgDn| 切换下一个窗口|

<!-- 模块帮助文件名：应用-讯飞输入法语音悬浮窗.ahk-->
#### 讯飞输入法语音悬浮窗模块| 模式 | 按键| 功能说明 |
| - | :-: | - |
| 任意 | Win + H | 切换讯飞语音输入 |

**需要先进行以下配置**
1. 在C盘默认目录安装[讯飞语音输入法 Windows 版](https://srf.xunfei.cn/)（如果安装到其它目录请自己改代码……）
2. 把讯飞语音输入法的语音悬浮窗热键设置为 `Ctrl+Shift+H`
3. 当需要语音的时候按 Win+H 呼出即可
4. Enjoy it.

![应用-讯飞语音输入法悬浮窗演示.gif](https://github.com/snomiao/CapslockX/raw/master/模块/应用-讯飞语音输入法悬浮窗演示.gif)

<!-- 模块帮助文件名：插件-OneNote剪贴板收集器.ahk-->
#### OneNote剪贴板收集器模块
##### OneNote 剪贴板收集插件

使用方法：
1. OneNote 2016 打开一个窗口，标题写成这样“剪贴板收集”。
2. 然后再用 Ctrl + C 复制东西的时候就会自动记到 OneNote 里
3. 如图
![插件-OneNote剪贴板收集器.gif](https://github.com/snomiao/capslockx/raw/master/模块/插件-OneNote剪贴板收集器.gif)

<!-- 模块帮助文件名：插件-合并右Ctrl与Menu键.ahk-->
#### 合并右Ctrl与Menu键模块
专治各种（Surface 的 右 Ctrl 键）残破键盘，合并 Menu与 右Ctrl键，Menu当Ctrl用 或者 Ctrl当Menu用都可以

| 模式 | 操作 | 说明 |
| - | :-: | - |
| 全局 | 右Ctrl按一下 | 会按一下 Menu 弹出菜单 |
| 全局 | 按住右Menu | 会按住 Ctrl，此时可以与其它键组合 |

<!-- 模块帮助文件名：插件-媒体键.ahk-->
#### 媒体键模块
| 模式 | 媒体键模块   | 说明 （这个模块基本还是半成品。。欢迎push） |
| - | :-: | - |
| CapslockX | F5   | 暂停播放 |
| CapslockX | F6   | 上一首 |
| CapslockX | F7   | 下一首 |
| CapslockX | F8   | 停止播放 |
| CapslockX | F10  | 静音 |
| CapslockX | F11  | 音量减 |
| CapslockX | F12  | 音量加 |

<!-- 模块帮助文件名：插件-搜索键.ahk-->
#### 搜索键模块
| 模式 | 搜索键模块   | 说明 （这个模块基本还是半成品。。欢迎push） |
| - | :-: | - |
| CapslockX | g    | 用 google 搜索当前选择或鼠标所指的词 |

<!-- 模块帮助文件名：插件-编辑增强.ahk-->
#### 编辑增强模块
| 模式 | Edit模块   | 说明 （欢迎push）|
| - | :-: | - |
| CapslockX | z         | 回车（单纯是为了把回车放到左手……以便右手可以一直撑着下巴玩电脑） |
| CapslockX | h l       | 左右方向键 |
| CapslockX | k j       | 上下方向键 |
| CapslockX | hl        | h l 一起按选择光标下的单词词 |
| CapslockX | nm        | n m 一起按选择当前行 |
| CapslockX | b         | BackSpace |
| CapslockX | B         | Delete |
| CapslockX | u         | 撤销 |
| CapslockX | U         | 重做 |

<!-- 模块帮助文件名：插件-雪星转屏.ahk-->
#### 雪星转屏模块
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
