# CapsX

## 入门教程第一
首先，你需要安装了 AutoHotKey。
推荐使用的版本： `AutoHotKey_L Unicode x32`

然后双击 `CapsX.ahk` 来启动 CapsX

> 某些功能需要使用管理员身份运行
> 
> 在 CapsX-Settings.ahk 中
> 
> 手动修改 global T_AskRunAsAdmin := 1 即可



## 使用手册第二

按 `CapsLock` 切换 CapsX 模式

### 按键表
每个模块可以单独禁用，请见 `CapsX-Settings.ahk` 文件

<!-- 下面这堆东西是自动从各个模块里抽取的 -->
<!-- 开始：抽取模块帮助 -->
#### Mouse模块
| 模式 | Mouse模块 | 说明 |
| - | :-: | - |
| CapsX | w a s d   | 鼠标平滑移动 |
| CapsX | r f       | 垂直平滑滚轮（在chrome下有时失灵，原因未明) |
| CapsX | R F       | 水平平滑滚轮（在chrome下有时失灵，原因未明) |
| CapsX | rf        | r f一起按相当于鼠标中键 |
| CapsX | e         | 鼠标左键 |
| CapsX | q         | 鼠标右键 |

#### WinTab模块
| 模式 | WinTab模块   | 说明 |
| - | :-: | - |
| Alt+Tab界面下 | w a s d | 切换窗口选择 |
| 窗口靠边界面下 | w a s d | 切换窗口选择 |
| 窗口靠边界面下 | x | 关掉选择的窗口 |
| Win+Tab界面下 | w a s d | 切换窗口选择 |
| Win+Tab界面下 | q e | 左右切换桌面概览
| Win+Tab界面下 | z | 合并当前桌面与上一个桌面
| Win+Tab界面下 | x | 关掉选择的窗口
| Win+Tab界面下 | 1 2 3 ... 9 0 | 把窗口移到除了自己的第x个桌面（或新建桌面） |
| Win+Tab界面下 | v | 新建桌面，并把当前窗口扔到新建桌面 |
| Win+Tab界面下 | c | 新建桌面，并把当前窗口扔到新建桌面，并激活窗口 |

#### Edit模块
| 模式 | Edit模块   | 说明 （这个模块基本还是半成品。。欢迎来改）|
| - | :-: | - |
| CapsX | h l       | 左右方向键 |
| CapsX | k j       | 上下方向键 |
| CapsX | hl        | h l 一起按选择光标下的单词词 |
| CapsX | nm        | n m 一起按选择当前行 |
| CapsX | b         | BackSpace |
| CapsX | B         | Delete |
| CapsX | u         | 撤销 |
| CapsX | U         | 重做 |

#### Media模块
| 模式 | Media模块   | 说明 （这个模块基本还是半成品。。欢迎来改） |
| - | :-: | - |
| CapsX | F5   | 暂停播放 |
| CapsX | F6   | 上一首 |
| CapsX | F7   | 下一首 |
| CapsX | F8   | 停止播放 |
| CapsX | F10  | 静音 |
| CapsX | F11  | 音量减 |
| CapsX | F12  | 音量加 |

#### Search模块
| 模式 | Search模块   | 说明 （这个模块基本还是半成品。。欢迎来改） |
| - | :-: | - |
| CapsX | g    | 用 google 搜索当前选择或鼠标所指的词 |
<!-- 结束：抽取模块帮助 -->



## 发展路线第三

简化电脑操作逻辑，提升效率，尽量不与习惯键位冲突


## 制作背景第四

本人比较经常写代码……

起初我是右鼠……后来觉得鼠标放右边有点远……改成了左鼠

左鼠之后发现手还是要离开键盘……于是做了这个

## 相关答疑第五
本人常驻 QQ群：271105729

关于这个脚本，相关答疑直接进群提问即可