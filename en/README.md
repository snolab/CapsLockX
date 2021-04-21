# CapsLockX - Use your computer Like a HACKER

CapsLockX is a hotkey scripting engine based on AutoHotkey. It allows you to easily and **efficiently** operate your computer like a movie **hacker**, without taking your hands off the keyboard. There are many features that are easy to understand and easy to use: editing enhancements, virtual desktop and window management, mouse emulation, in-app hotkey enhancements, JS mathematical expression calculations, and many more features for you to define yourself. Main repository address: [https://github.com/snolab/CapsLockX](https://github.com/snolab/CapsLockX)

[![jsdelivr_GITHUB](https://data.jsdelivr.com/v1/package/gh/snolab/capslockx/badge)](https://www.jsdelivr.com/package/gh/snolab/capslockx)
[![gh-pages](https://github.com/snolab/CapsLockX/actions/workflows/release-github.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/release-github.yml)
[![NPM](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml)
[![all downloads](https://img.shields.io/github/downloads/snolab/CapsLockX/total.svg?style=flat-square&label=All%20downloads)](https://github.com/snolab/CapsLockX/releases)
<!-- [![Chocolatey](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml) -->
<!-- [![Packages Test](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml) -->

## Tutorials

### TL;DR - Quick Start

Download this [Downlaod JSDelivrCDN-PublishPackage.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)

Once installed, you can press and hold CapsLockX, then press `WASD` to move the mouse, `QE` to click the `RF` wheel, `HJKL` to move the cursor, `YOUI` to move the page, `ZXCV` to manage the window, 1234567890 to switch the virtual desktop, and M to open the configuration.

### Install & Usage

#### Portable Package

源码打包就是软件本体，无需编译，纯绿色软件解压即用。源码 + 程序包，推荐第一个（最快）。

- [Download JSDelivrCDN-PublishPackage.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)
- [Alternative Download CloudFlareCDN-PublishPackage.zip](https://capslockx.snomiao.com/CapsLockX-latest.zip)
- [Alternative Download 全球用户-GitHub-PublishPackage.zip](https://github.com/snolab/CapsLockX/raw/gh-pages/CapsLockX-latest.zip)
- [Alternative Download 全球用户-GitHub-仓库程序包.zip](https://github.com/snolab/CapsLockX/archive/master.zip)
- [Alternative Download 灾难备份-BitBucket-仓库程序包.zip](https://bitbucket.org/snomiao/capslockx/get/master.zip)
- [Alternative Download 大陆用户-Gitee-仓库程序包.zip （需登录）](https://gitee.com/snomiao/CapslockX/repository/archive/master.zip)

解压后使用即可，启动与自启动的方法： 双击 `CapsLockX.exe` 即可启动脚本，如需添加启动项，请在开始菜单 - 运行输入 shell:startup 然后给本程序创建快捷方式，扔进去就可以了。

#### Command line installation (recommended for advanced users, can be updated automatically)

Choose any one of the following, the first one is recommended (the fastest)

- `git clone https://github.com/snolab/CapsLockX && .\CapsLockX\CapsLockX.exe`
- `git clone https://gitee.com/snomiao/CapslockX && .\CapsLockX\CapsLockX.exe`
- `npm i -g capslockx && capslockx`
- `npx capslockx`
<!-- - `choco install capslockx && capslockx` （注：2021-04-17 程序包审核中） -->

### Usage

Once installed, you can press and hold CapsLockX, then press `WASD` to move the mouse, `QE` to click the `RF` wheel, `HJKL` to move the cursor, `YOUI` to move the page, `ZXCV` to manage the window, 1234567890 to switch the virtual desktop, and M to open the configuration.

## Manual

Press and hold `CapsLock` to enter CapsLockX mode, your keyboard will become a functional keyboard like the Vim default mode, (see below for key positions)

CapsLockX is loaded with some common modules by default, the functions and usage are listed below.
For modules you don't need, you can also delete the `. /Modules` directory and press `Ctrl + Alt + \` to reload it.

<!-- 下面这堆东西是自动从各个模块里抽取的，如需改动请到对应模块.md 里操作, 在这里修改会被覆盖 -->
<!-- 开始：抽取模块帮助 -->
<!-- 模块文件名：00-Help.ahk-->

### 帮助模块

如果你想学习如何开发 CapsLockX 的插件，请：

1. 打开 `Modules/01-Help.ahk` ， 你可以了解到 CapsLockX 插件的基本格式
2. 将它复制一份，命名为你自己的插件名称
3. 将它本来的功能改成你自己需要的功能，插件的开发就完成啦！

#### 本模块功能见下

| 作用于 | 按键                  | 功能                               |
| ------ | --------------------- | ---------------------------------- |
| 全局   | CapsLockX + /         | 临时显示热键提示                   |
| 全局   | CapsLockX（长按）     | 临时显示热键提示                   |
| 全局   | CapsLockX + Alt + /   | 🔗 打开 CapsLockX 的 README.md 页面 |
| 全局   | CapsLockX + Shift + / | 🕷 提交 bug、建议等                 |

<!-- 模块文件名：01.1-插件-鼠标模拟.ahk-->

### 模拟鼠标插件（ WASD QERF ）

#### 功能

- 本模块使用按键区：CapsLockX + QWER ASDF
- 非常舒适地使用 WASD QE RF 来模拟【完整的】鼠标功能，相信我，试过这种手感之后，你会喜欢上它的。
- 指针移动时会自动黏附各种按钮、超链接。滚轮的指数级增长的加速度滚动机制使你再也不惧怕超级长的文章和网页。
- 效果如图：
  ![鼠标模拟.gif]( ./media/鼠标模拟.gif )

#### 使用方法如下

| 作用于 | 按键                  | 说明                      |
| ------ | --------------------- | ------------------------- |
| 全局   | CapsLockX + w a s d   | 鼠标移动（上下左右）      |
| 全局   | CapsLockX + r f       | 垂直滚轮（上下）          |
| 全局   | CapsLockX + R F       | 水平滚轮（左右）          |
| 全局   | CapsLockX + Alt + r f | 垂直滚轮自动滚动（上 下） |
| 全局   | CapsLockX + Alt + R F | 水平滚轮自动滚动（左 右） |
| 全局   | CapsLockX + rf        | rf 同时按相当于鼠标中键   |
| 全局   | CapsLockX + e         | 鼠标左键                  |
| 全局   | CapsLockX + q         | 鼠标右键                  |

<!-- 模块文件名：01.2-插件-编辑增强.ahk-->

### 编辑增强插件（ TG YUIO HJKL ）

这个世界上还有比 Vim 模式的 HJKL 移动光标更棒的东西吗？
这个必须有！
那就是带加速度的 HJKL 流畅编辑体验！想不想试试让你的光标来一次排水沟过弯的高端操作？装它！

![光标移动.gif]( ./media/光标移动.gif )

| 作用域     | Edit 模块             | 说明                |
| ---------- | --------------------- | ------------------- |
| 全局(基本) | CapsLockX + h j k l   | 上下左右 方向键     |
| 全局(基本) | CapsLockX + y o       | Home End            |
| 全局(基本) | CapsLockX + u i       | PageUp PageDown     |
| 全局(基本) | CapsLockX + g         | 回车                |
| 全局(基本) | CapsLockX + t         | BackSpace           |
| 全局(进阶) | CapsLockX + Shift + t | Delete              |
| 全局(进阶) | CapsLockX + hl        | hl 一起按选择当前词 |
| 全局(进阶) | CapsLockX + kj        | kj 一起按选择当前行 |

<!-- 模块文件名：03.1-插件-窗口增强.ahk-->

### 窗口增强插件 (123456789-=XC)

#### 功能简述

用好 Win 10 自带的 10 个虚拟桌面豪华配置、多显示器自动排列窗口、半透明置顶、

1. 窗口切换：`CapsLockX + [Shift] + Z`
1. 窗口关闭：`CapsLockX + [Shift] + X`
1. 窗口排列：`CapsLockX + [Shift] + C`
1. 窗口置顶：`CapsLockX + [Shift] + V`
1. 左手窗口管理：在 `Alt + Tab` 的界面，用 `WASD` 切换窗口，`X` 关掉窗口。
1. 高效使用虚拟桌面：`CapsLockX + 0123456789` 切换、增减虚拟桌面，加上 Alt 键可以转移当前窗口
1. 虚拟机与远程桌面快速脱离：双击左边 `Shift+Ctrl+Alt`。

#### 效果图

- Alt + Tab 管理窗口增强
  ![02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif]( ./media/02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif )
- CapsLockX + C 一键排列窗口（这 GIF 是旧版本录的看起来比较卡，新版本优化过 API 就不卡了）
  ![02-插件-窗口增强_一键排列窗口.gif]( ./media/02-插件-窗口增强_一键排列窗口.gif )

#### 使用方法如下 ( Alt+Tab 与 CapsLockX )

| 作用域       | 窗口增强模块                        | 说明                                       |
| ------------ | ----------------------------------- | ------------------------------------------ |
| Alt+Tab 界面 | Q E                                 | 左右切换多桌面                             |
| Alt+Tab 界面 | W A S D                             | 上下左右切换窗口选择                       |
| Alt+Tab 界面 | X C                                 | 关闭选择的窗口（目前 X 和 C 没有区别）     |
| Win+Tab 视图 | Alt + W A S D                       | 切换窗口选择                               |
| 全局         | CapsLockX + 1 2 ... 9 0 - =         | 切换到第 1 .. 12 个桌面                    |
| 全局         | CapsLockX + Backspace               | 删除当前桌面（会把所有窗口移到上一个桌面） |
| 全局         | CapsLockX + Shift + 1 2 ... 9 0 - = | 把当前窗口移到第 n 个桌面(如果有的话)      |
| 全局         | CapsLockX + C                       | 快速排列当前桌面的窗口                     |
| 全局         | CapsLockX + Ctrl + C                | 快速排列当前桌面的窗口（包括最小化的窗口） |
| 全局         | CapsLockX + Shift + C               | 快速堆叠当前桌面的窗口                     |
| 全局         | CapsLockX + Shift + Ctrl + C        | 快速堆叠当前桌面的窗口（包括最小化的窗口） |
| 全局         | CapsLockX + Shift + [ ]             | 把当前窗口移到上一个/下一个桌面            |
| 全局         | CapsLockX + Z                       | 循环切到最近使用的窗口                     |
| 全局         | CapsLockX + Shift+ Z                | 循环切到最不近使用的窗口                   |
| 全局         | CapsLockX + X                       | 关掉当前标签页 Ctrl+W                      |
| 全局         | CapsLockX + Shift+ X                | 关掉当前窗口 Alt+F4                        |
| 全局         | CapsLockX + V                       | 让窗口透明                                 |
| 全局         | CapsLockX + Shift+ V                | 让窗口保持透明（并置顶）                   |
| 任意窗口     | 双击左边 Shift+Ctrl+Alt             | 后置当前窗口， * 见下方注                  |

*注： 双击左边 Shift+Ctrl+Alt 设计用于远程桌面与虚拟机，使其可与本机桌面窗口同时显示。
例如 mstsc.exe、TeamViewer、VirtualBox、HyperV、VMWare 等远程桌面或虚拟机程序，配合 CapsLockX + Shift + V 透明置顶功能，让你在 Windows 的界面上同时使用 Linux 界面或 MacOS 界面再也不是难题。
  
此处借用 [@yangbin9317 的评论]( https://v2ex.com/t/772052#r_10458792 )
> 以 CapsLock 为抓手,打通底层逻辑,拉齐 Windows 和 Linux WM,解决了 Windows 难用的痛点

<!-- 模块文件名：LaptopKeyboardFix.ahk-->

### Surface 笔记本扩充功能键

专治各种笔记本残破键盘

1. 没有右 Ctrl 键？合并 Menu 与 右 Ctrl 键，Menu 当 Ctrl 用 或者 Ctrl 当 Menu 用都可以
2. 没有 Pause 键？Win + Alt + P 也能打开系统设定信息。
3. 待补充

| 模式 | 按键           | 功能                               |
| ---- | :------------- | ---------------------------------- |
| 全局 | Win + Alt + P  | 相当于 Win + Pause，专为笔记本定制 |
| 全局 | 右 Ctrl 按一下 | 会按一下 Menu 弹出菜单             |
| 全局 | 按住右 Menu    | 会按住 Ctrl，此时可以与其它键组合  |

<!-- 模块文件名：功能-秒打时间戳.ahk-->

### 秒打时间戳

| 模式 | 秒打时间戳 | 说明                                                |
| ---- | ---------- | --------------------------------------------------- |
| 全局 | #D#        | 插入日期, 类似 2021-04-19- 这样的时间戳             |
| 全局 | #DD#       | 插入日期, 类似 (20190115) 这样的时间戳              |
| 全局 | #T#        | 插入日期时间, 类似 20190115.164744 这样的时间戳     |
| 全局 | #TT#       | 插入日期时间, 类似 (20190115.164744) 这样的时间戳   |
| 全局 | #DT#       | 插入日期时间, 类似 2021-04-19 04:30:35 这样的时间戳 |

<!-- 模块文件名：应用-Anki增强.ahk-->

### Anki 增强模块

| 模式                 | Anki 增强模块 | 说明                                                        |
| -------------------- | :-----------: | ----------------------------------------------------------- |
| 在 Anki-学习界面     |  w 或 k 或 ↑  | 按下=撤销，松开显示答案                                     |
| 在 Anki-学习界面     |  a 或 h 或 ←  | 按下=顺利，松开显示答案                                     |
| 在 Anki-学习界面     |  s 或 j 或 ↓  | 按下=困难，松开显示答案                                     |
| 在 Anki-学习界面     |  d 或 l 或 →  | 按下=生疏，松开显示答案                                     |
| 在 Anki-学习界面     |       q       | 返回上个界面                                                |
| 在 Anki-学习界面     |       c       | 添加新卡片                                                  |
| 在 Anki-学习界面     | 1 或 NumPad1  | 困难（原键位不动）                                          |
| 在 Anki-学习界面     | 2 或 NumPad2  | 生疏（原键位不动）                                          |
| 在 Anki-学习界面     | 3 或 NumPad3  | 一般（原键位不动）                                          |
| 在 Anki-学习界面     | 4 或 NumPad4  | 顺利（原键位不动）                                          |
| 在 Anki-学习界面     | 5 或 NumPad5  | 撤销                                                        |
| 在 Anki-学习界面     | 6 或 NumPad6  | 暂停卡片                                                    |
| 在 Anki-学习界面     |    Alt + i    | 快速导入剪贴版的内容（按 Tab 分割） / 比如可以从 Excel 复制 |
| 在 Anki-添加卡片界面 |    Alt + s    | 按下 添加 按钮                                              |

> 此插件可配合手柄使用，使用 XPadder 配置手柄摇杆映射到方向键即可。
>
> 效果请见 bilibili [中二雪星怎背词 - 手柄怎么可以不用来背单词！](https://www.bilibili.com/video/av8456838/)

<!-- 模块文件名：应用-Edge增强.ahk-->

### Edge 增强模块（测试中）

| 模式     |  按键   | 说明                                  |
| -------- | :-----: | ------------------------------------- |
| 在Edge内 | Alt + w | 拿出笔（全屏模式暂时不支持）          |
| 在Edge内 | Alt + q | 换左边的笔/橡皮（全屏模式暂时不支持） |
| 在Edge内 | Alt + e | 换右边的笔/橡皮（全屏模式暂时不支持） |
| 在Edge内 | Alt + , | 上一章/节                             |
| 在Edge内 | Alt + . | 下一章/节                             |
| 在Edge内 | Alt + / | 显示目录                              |
| 在Edge内 | Alt + ; | 切换自适应页面大小模式                |
| 在Edge内 | Alt + ' | 切换双页布局模式                      |

<!-- 模块文件名：应用-OneNote2016增强.ahk-->

### OneNote 2016

我很确定我们用的不是同一个 OneNote，因为，你没有装 CapsLockX ！

#### 按键分布设计（开发中）

| 按键描述              | 作用                    | 备注   |
| --------------------- | ----------------------- | ------ |
| 所有 OneNote 自带热键 | 原功能                  |        |
| 按一下 Alt 再按别的   | 触发 OneNote 原菜单功能 |        |
| Alt + /               | 热键帮助、提示          | 开发中 |
| Alt + 1234567         | 大纲折叠展开到 1-7 层级 |        |
| Alt + qwe asd rf      | 工具、换笔、视图        |        |
| Alt + -=              | 公式                    |        |
| Alt + m               | 移动笔记                |        |
| Alt + hjkl            | 各种链接功能            |        |
| Alt + zxcv            | 高级复制粘贴            |        |
| F2 F3                 | 重命名、查找笔记        |        |

#### 详细按键表 / CheatSheet

| 作用于                  | 格式热键                       | 功能                                                       |
| ----------------------- | ------------------------------ | ---------------------------------------------------------- |
| OneNote2016             | `Alt + 1234567`                | 大纲：大纲折叠展开到那层（强烈推荐，超好用）               |
| OneNote2016             | `Ctrl + Shift + c`             | 转换：复制（纯文本）                                       |
| OneNote2016             | `Ctrl + Shift + v`             | 转换：粘贴（纯文本）                                       |
| OneNote2016             | `F2`                           | 整理：重命名笔记                                           |
| OneNote2016             | `Shift + F2`                   | 整理：重命名分区                                           |
| OneNote2016             | `Alt + m`                      | 整理：移动笔记                                             |
| OneNote2016             | `Alt + Shift + m`              | 整理：移动分区                                             |
| OneNote2016             | `Ctrl + n`                     | 整理：新建笔记                                             |
| OneNote2016             | `Ctrl + Alt + n`               | 整理：在当前笔记下方新建笔记                               |
| OneNote2016             | `Alt + Delete`                 | 整理：快速删除当前页面                                     |
| OneNote2016             | `Ctrl + s`                     | 整理：立即同步此笔记本                                     |
| OneNote2016             | `Ctrl + w`                     | 整理：关闭窗口                                             |
| OneNote2016             | `Shift + Delete`               | 编辑：快速删除当前行                                       |
| OneNote2016             | `Alt + -`                      | 编辑：自动2维化公式                                        |
| OneNote2016             | `Alt + k`                      | 编辑：展开当前关键词的相关页面链接（快速关键词一对多链接） |
| OneNote2016             | `Alt + n`                      | 样式：切换页面为无色背景                                   |
| OneNote2016             | `Alt + v`                      | 样式：改变文字背景色                                       |
| OneNote2016             | `Alt + q`                      | 工具：拖动                                                 |
| OneNote2016             | `Alt + w`                      | 工具：套锁                                                 |
| OneNote2016             | `Alt + e`                      | 工具：橡皮                                                 |
| OneNote2016             | `Alt + s`                      | 工具：输入                                                 |
| OneNote2016             | `Alt + a`                      | 工具：换到第2支笔                                          |
| OneNote2016             | `Alt + d`                      | 工具：打开换笔盘（然后可可方向键选笔 （目前全屏无效）      |
| OneNote2016             | `Alt + d 然后 1234567`         | 工具：打开换笔盘（然后选第1行第x支笔） （目前全屏无效）    |
| OneNote2016             | `Alt + d 然后 Shift + 1234567` | 工具：打开换笔盘（然后选第2行第x支笔） （目前全屏无效）    |
| OneNote2016             | `Alt + r`                      | 视图：缩放到原始大小                                       |
| OneNote2016             | `Alt + y`                      | 视图：缩放到页面宽度                                       |
| OneNote2016             | `^!+- 或 ^!+=`                 | 视图：缩小页面 或 放大页面                                 |
| OneNote2016             | `Alt + f`                      | 视图：搜索标记                                             |
| OneNote2016创建链接窗口 | `Alt + s`                      | 转换：复制当前所有搜索结果页面的链接                       |
| OneNote2016创建链接窗口 | `Alt + Shift + s`              | 转换：复制当前所有搜索结果页面的链接并粘贴                 |
| `剪贴板` 笔记打开时     | `Ctrl + C`                     | 转换：追加复制的内容到该笔记                               |
| `Clipboard` 笔记打开时  | `Ctrl + C`                     | 转换：追加复制的内容到该笔记                               |

<!-- 模块文件名：应用-TIM添加常驻功能.ahk-->

### TIM添加常驻功能模块

| 模式        |    按键     | 功能                   |
| ----------- | :---------: | ---------------------- |
| 在Tim窗口内 |   Alt + f   | 焦点定位到左上角搜索框 |
| 在Tim窗口内 | Ctrl + PgUp | 切换上一个窗口         |
| 在Tim窗口内 | Ctrl + PgDn | 切换下一个窗口         |

<!-- 模块文件名：应用-讯飞输入法语音悬浮窗.ahk-->

### 讯飞输入法悬浮窗插件

#### 用法

| 作用于 |  按键   | 功能说明              |
| ------ | :-----: | --------------------- |
| 全局   | Win + H | 启动/切换讯飞语音输入 |

#### 注

1. 原 `Win + H` 的功能是 Windows 自带听写，安装本插件后，可通过 `Win + Shift + H` 使用原 Windows 的听写
2. 若没有安装讯飞语音则会自动询问是否引导下载安装

#### 效果如下图

![应用-讯飞语音输入法悬浮窗演示.gif]( ./media/应用-讯飞语音输入法悬浮窗演示.gif )

<!-- 模块文件名：插件-媒体键.ahk-->

### 媒体键模块

| 作用于 | 媒体键模块      | 说明                                        |
| ------ | --------------- | ------------------------------------------- |
| 全局   | CapsLockX + F1  | 打开：我的电脑                              |
| 全局   | CapsLockX + F2  | 打开：计算器                                |
| 全局   | CapsLockX + F3  | 打开：浏览器主页                            |
| 全局   | CapsLockX + F4  | 打开：媒体库（默认是 Windows Media Player） |
| 全局   | CapsLockX + F5  | 播放：暂停/播放                             |
| 全局   | CapsLockX + F6  | 播放：上一首                                |
| 全局   | CapsLockX + F7  | 播放：下一首                                |
| 全局   | CapsLockX + F8  | 播放：停止                                  |
| 全局   | CapsLockX + F9  | 音量加                                      |
| 全局   | CapsLockX + F10 | 音量减                                      |
| 全局   | CapsLockX + F11 | 静音                                        |
| 全局   | CapsLockX + F12 |                                             |

<!-- 模块文件名：插件-定时任务.ahk-->

### 定时任务

使用  CapsLockX + M 打开配置，然后修改 EnableScheduleTasks=1 即可启用本插件。

- 使用番茄报时（00分和30分播放工作铃声，每小时的25分和55分播放休息铃声）（需要先开启定时任务）

    ```ini
    UseTomatoLife=1
    ```

- 使用番茄报时时，自动切换桌面（使用番茄报时时，自动切换桌面（休息桌面为1，工作桌面为2）

    ```ini
    UseTomatoLifeSwitchVirtualDesktop=1
    ```

注：如果只需要声音而不需要自动切换桌面的话，也可试试这款 Chrome 插件 [Tomato Life - Chrome 网上应用店](https://chrome.google.com/webstore/detail/25min-tomato-life/kkacpbmkhbljebmpcopjlgfgbgeokbhn)

<!-- 模块文件名：插件-雪星转屏.ahk-->

### 雪星转屏模块

功能：同步旋转你所有的屏幕，自动对齐屏幕边界，不会错位

使用方式如下

| 模式 | 按键                                 | 功能                           |
| ---- | ------------------------------------ | ------------------------------ |
| 全局 | CapsLockX + Alt + 方向键 上 下 左 右 | 同时旋转所有屏幕到你指定的方向 |
<!-- 结束：抽取模块帮助 -->

## 过去与未来第四

### Development Stories ( Fall 2017 )

> I used write code often ...
> At first I used the right hand mouse ...... then I felt that the mouse was a bit far to the right ...... and changed to the left hand mouse
> After using the mouse with my left hand, I found that my hand still had to leave the keyboard ...... so I made a script that simulates the mouse with WASD. (Then I can always use my right hand to hold my chin to play computer)
> Later, I wrote more and more scripts, so I put some of the common ones together to load ......

### Development Future

The core idea of ​​this project is: simplify the system operation logic, improve operation efficiency, and try not to conflict with the original custom keys.

1. [x] Press CapsLockX + / to display the corresponding help (the current display style is rather sloppy)
2. [ ] Auto update
3. [ ] First-time use tutorial
4. [ ] Plugin manager
5. [ ] Automatic configuration synchronization function
6. [ ] Easy-to-use option configuration UI interface
7. [ ] Execute external code

If you have any ideas or suggestions, please put them here: [Issues · snomiao/CapslockX](https://github.com/snolab/CapsLockX/issues)

### Combination key meaning design

- `Win +`
  Usually operating system functions, desktop window application process management, etc., input method, output device (display, multi-screen) management
- `Alt +`
  It is usually expressed as calling an in-app function, and its meaning should be equivalent to pressing a button with the same function or jumping to a specific function interface.
- `Ctrl +`
  Same as above, but it is used more frequently, and there is probably no button with the same function.
- `Ctrl + Alt +`
  Same as above, but generally a global hotkey
- `+ Shift +`
  It is used to slightly change the meaning of the keys on the basis of the above functions (for example, reverse operation such as `Shift+Alt+Tab`, or expansion of the function range such as `Shift+direction keys` to adjust selection, etc. )

### Comparison of functions between this project and similar projects / updated in (20200627) The information in it may slowly become out of date

| Function\Project                 | [CapsLockX](https://github.com/snolab/CapsLockX) | [Vonng/CapsLock](https://github.com/Vonng/CapsLock) | [coralsw/CapsEz](https://github.com/coralsw/CapsEz) | [CapsLock+](https://capslox.com/capslock-plus/) |
| :------------------------------- | :----------------------------------------------- | :-------------------------------------------------- | :-------------------------------------------------- | :---------------------------------------------- |
| Mouse simulation                 | ✅ Smooth and complete                            | ✅ No roller                                         | 🈚 No                                                | 🈚 No                                            |
| Expression calculation           | ✅ Nodejs                                         | 🈚 No                                                | 🈚 No                                                | ✅ TabScript (Snippet + Javascript)              |
| Window management                | ✅ Fully                                          | ✅ Yes                                               | ✅ Yes                                               | ✅ Fully                                         |
| Virtual desktop management       | ✅ Yes                                            | 🈚 No                                                | 🈚 No                                                | 🈚 No                                            |
| Edit enhancement                 | ✅ Yes（F=ma）                                    | ✅ Yes                                               | ✅ Yes                                               | ✅ Yes（Fully）                                  |
| Green free installation          | ✅ Yes                                            | ✅ Yes                                               | ✅ Yes                                               | ✅ Yes                                           |
| Enhanced media key               | Weak                                             | ✅ Fully                                             | 🈚 No                                                | 🈚 No                                            |
| Enhanced clipboard               | Weak                                             | 🈚 No                                                | 🈚 No                                                | ✅ Yes                                           |
| Quick start application          | ✅ 插件                                           | ✅ Yes                                               | ✅ Yes                                               | ✅ Yes                                           |
| Application function enhancement | ✅ 丰富                                           | 🈚 No                                                | ✅ Yes                                               | 🈚 No                                            |
| Bash control                     | 🈚 No                                             | ✅ Yes                                               | 🈚 No                                                | 🈚 No                                            |
| Quick start voice input          | ✅ 讯飞                                           | 🈚 No                                                | 🈚 No                                                | 🈚 No                                            |
| Quickly enter the time and date  | ✅ Yes                                            |                                                     | ✅ Yes                                               |                                                 |
| Window bound to hotkey           | 🈚 No                                             | 🈚 No                                                | 🈚 No                                                | ✅ Yes                                           |
| Rotate the screen quickly        | ✅ Yes                                            | 🈚 No                                                | 🈚 No                                                | 🈚 No                                            |
| Secondary development            | ✅ Document friendly                              | ✅ Yes                                               | ✅ Yes                                               | ✅ Yes                                           |
| Memory footprint                 | ✅ About 2~3M                                     |                                                     |                                                     |                                                 |
| Modular                          | ✅                                                | 🈚 No                                                | 🈚 No                                                | 🈚 No                                            |
| system                           | Win                                              | Mac（Main），Win（Secondary）                       | Win                                                 | Win, [Mac](https://capslox.com/)                |
| Support language                 | 中文 / English(Doc)                              | 中文 / English                                      | 中文                                                | 中文 / English                                  |

#### 本项目地址

The following warehouses are updated simultaneously:

- GitHub: [https://github.com/snolab/CapsLockX](https://github.com/snolab/CapsLockX)
- Gitee: [https://gitee.com/snomiao/CapslockX](https://gitee.com/snomiao/CapslockX)
- Bitbucket: [https://bitbucket.org/snomiao/capslockx](https://bitbucket.org/snomiao/capslockx)
- Gitlab: [https://gitlab.com/snomiao/CapsLockX/](https://gitlab.com/snomiao/CapsLockX/)

Document links:

- 中文文档 Netlify CDN：[https://capslockx.netlify.com](https://capslockx.netlify.com)
- 中文文档 CloudFlare CDN：[https://capslockx.snomiao.com](https://capslockx.snomiao.com)
- 中文文档 CloudFlare CDN：[https://capslockx.snomiao.com](https://capslockx.snomiao.com)
- 中文文档 Github Pages：[http://snolab.github.io/CapsLockX](http://snolab.github.io/CapsLockX)

#### Similar project address

- Source code: [Vonng/CapsLock: Make CapsLock Great Again!](https://github.com/Vonng/CapsLock)
  design: [Capslock/design.md at master · Vonng/Capslock](https://github.com/Vonng/Capslock/blob/master/design.md)
- [coralsw/CapsEz: KeyMouse Tools](https://github.com/coralsw/CapsEz)
- [CapsLock+](https://capslox.com/CapsLock-plus/)
- [Capslox](https://capslox.com/cn/)
- CapsLock++ [matrix1001/CapsLock-plus-plus: ⌨Amazing, extendable, readable autohotkey scripts framework utilized by CapsLock.](https://github.com/matrix1001/CapsLock-plus-plus)

#### Other efficiency software recommendations

- [Quicker](https://getquicker.net/) 也是一个提高电脑操作效率的软件，与本项目可以互补。<!-- （雪星的推荐码： 55396-2857） -->
- [Everything](https://www.voidtools.com/zh-cn/)

## LICENSE

[GNU General Public License v3.0 - GNU Engineering-Free Software Foundation](https://www.gnu.org/licenses/gpl-3.0.html)。

## Q&A related

Related communities:

- CapsLockX User Telegram Group: [t.me/CapsLockX_users](https://t.me/CapsLockX_users)
- [Issues of this project (can be used as a forum)](https://github.com/snolab/CapsLockX/issues)

### Related Topics

- [CapsLockX - 像黑客一样操作电脑 - V2EX]( https://v2ex.com/t/772052#reply1 )
- [(10) What are some good career alternatives for a computer programmer with RSI? - Quora]( https://www.quora.com/Repetitive-Strain-Injury-RSI/What-are-some-good-career-alternatives-for-a-computer-programmer-with-RSI )
- [如何将电脑桌面划分为独立的两半？ - 知乎]( https://www.zhihu.com/questionz/23443944/answer/1670521971 )
- [有哪位残友用的是单手键盘？ - 知乎]( https://www.zhihu.com/question/50621709/answer/1681247637 )
- [(5 封私信 / 50 条消息) 怎么样才能只用键盘不用鼠标，包括任何指针触控设备，并优雅地使用电脑？ - 知乎]( https://www.zhihu.com/question/21281518/answer/1770669886 )
- [(5 封私信 / 50 条消息) 如何将电脑桌面划分为独立的两半？ - 知乎]( https://www.zhihu.com/question/23443944/answer/1670521971 )
- [我是职场达人，AutoHotKey让我成为职场超人 - 知乎]( https://zhuanlan.zhihu.com/p/60372361 )
- [AutoHotKey 中文网专栏 - 知乎]( https://www.zhihu.com/column/autoahk )

## Support the sixth

How can I help this program survive? If this project has helped you.

1. Welcome to star this project on Github
2. Feel free to share this project to your friends.
3. Feel free to help me translate readme.md to different languages. 4.
4. Welcome to submit bugs, suggestions for improvement [issues](https://github.com/snolab/CapsLockX/issues)
5. you are welcome to submit code PR, even if it is to fix typos ~ 6.
6. Welcome to donate to the development of this project here, each donation will be recorded in the list below:.
   - PAYPAL: [https://paypal.me/snomiao](https://paypal.me/snomiao)
   - Love Power: [https://afdian.net/@snomiao](https://afdian.net/@snomiao)
   - Alipay Donation Account: [snomiao@gmail.com (click for QR code)](../支付宝捐助.png)

Your support will practically, help to the future development work of this project, the development plan is here: [development route](#development route)

### Thanks

- Thanks to [秦金伟](http://rsytes.coding-pages.com/) Cited recommended articles and development suggestions: [2020-02-23 当键盘模拟鼠标 - 简书](https://www.jianshu.com/p/f757f56a7de6)
- Thanks to @河许人 help promotion: [CapsLockX – 像黑客一样操作电脑！【雪星】 – AutoAHK]( https://www.autoahk.com/archives/34996 )
