# CapsLockX - 像**黑客**一样操作电脑

CapsLockX 是一款基于 AutoHotkey 的模块化热键脚本引擎。 让你可以轻轻松松像电影里的**黑客**一样，双手不离开键盘，**高效率**地操作电脑。这里有超多一摸就懂超好上手的功能：编辑增强、虚拟桌面与窗口管理、鼠标模拟、应用内热键增强、JS 数学表达式计算、等超多功能等你来亲自定义。

[![jsdelivr_NPM](https://data.jsdelivr.com/v1/package/npm/capslockx/badge)](https://www.jsdelivr.com/package/npm/capslockx)
[![jsdelivr_GITHUB](https://data.jsdelivr.com/v1/package/gh/snolab/capslockx/badge)](https://www.jsdelivr.com/package/gh/snolab/capslockx)
[![gh-pages](https://github.com/snolab/CapsLockX/actions/workflows/release-github.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/release-github.yml)
[![NPM](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml)
<!-- [![Chocolatey](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml) -->
<!-- [![Packages Test](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml) -->

## 入门教程第一

### 太长不看版 TL;DR

下这个： [下载 JSDelivrCDN-发布程序包.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)

装上之后，按住 CapsLockX，然后 WASD 鼠标移动，QE 点击 RF 滚轮，HJKL 光标移动，YOUI 页面移动，ZXCV 窗口管理，M 打开配置。

### 安装与使用

#### 绿色程序包（新手适用，稳定版）

源码打包就是软件本体，无需编译，纯绿色软件解压即用。源码 + 程序包，推荐第一个（最快）。

- [下载 JSDelivrCDN-发布程序包.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)
- [下载 CloudFlareCDN-发布程序包.zip](https://capslockx.snomiao.com/CapsLockX-latest.zip)
- [下载 全球用户-GitHub-发布程序包.zip](https://github.com/snolab/CapsLockX/raw/gh-pages/CapsLockX-latest.zip)
- [下载 全球用户-GitHub-仓库程序包.zip](https://github.com/snolab/CapsLockX/archive/master.zip)
- [下载 灾难备份-BitBucket-仓库程序包.zip](https://bitbucket.org/snomiao/capslockx/get/master.zip)
- [下载 大陆用户-Gitee-仓库程序包.zip （需登录）](https://gitee.com/snomiao/CapslockX/repository/archive/master.zip)

解压后使用即可，启动与自启动的方法： 双击 `CapsLockX.exe` 即可启动脚本，如需添加启动项，请在开始菜单 - 运行输入 shell:startup 然后给本程序创建快捷方式，扔进去就可以了。

#### 命令行方式安装（进阶用户推荐，可自动更新）

以下几种任选其一，推荐第一个（最快）

1. `git clone https://gitee.com/snomiao/CapslockX && .\CapsLockX\CapsLockX.exe`
2. `git clone https://github.com/snolab/CapsLockX && .\CapsLockX\CapsLockX.exe`
3. `npm i -g capslockx && capslockx`
4. `npx capslockx`
<!-- 5. `choco install capslockx && capslockx` -->

### 使用方式

启动脚本后, 你可以尝试：按下 `Alt`, 按住 `Tab`, 然后按 `WASD` 切换选中窗口, 再按 `C` 关闭窗口，

> 某些功能需要使用管理员身份运行（如鼠标模拟）
> 在 CapsLockX-Settings.ahk 中
> 手动修改 global T_AskRunAsAdmin := 1 即可
> 如不需要，可以改为 0

### CapsLockX + 键位简述

按住 CapsLock 或 空格 然后再按……

- 鼠标模拟 `WASD EQ RF` 分别是鼠标的 移动 左右键 滚轮
- 编辑增强 `HJKL YUIO TG` 分别是光标的 左上下右 页面键 退格回车
- 窗口管理 `ZXCV` 分别是窗口的 切换 关闭 排列 透视
- 剩下的 `BNM P` 目前保留中

## 目录第二

## 使用手册第三

<!-- * 按 `CapsLock` 切换 CapsLockX 模式 -->

- 按住 `CapsLock` 进入 CapsLockX 模式，此时你的键盘会成为像 Vim 默认模式那样的功能键盘，（键位见下

CapsLockX 默认加载了一些常用的模块，功能与使用方法已在下方列出。
对于不需要的模块，你也可以直接删除 `./Modules` 目录下对应的文件，然后按 `Ctrl + Alt + \` 重新加载即可。

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

12 个虚拟桌面豪华配置、多显示器自动排列窗口、半透明置顶、

1. 快速排列窗口：CapsLockX + C
2. 左手窗口管理：在 Alt + Tab 的时候，用 `WASD` 切换窗口，X 关掉窗口
3. 高效使用虚拟桌面：`CapsLockX + 0123456789-=` 切换、增减虚拟桌面，加上 Alt 键可以转移当前窗口

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

| 模式 | 秒打时间戳 | 说明                                          |
| ---- | ---------- | --------------------------------------------- |
| 全局 | /d 或 (d   | 插入日期, 类似 (20190115) 这样的时间戳        |
| 全局 | /t 或 (t   | 插入时间, 类似 (20190115.164744) 这样的时间戳 |
| 全局 | /s 或 (s   | 插入时间, 类似 (1647) 这样的时间戳            |
| 全局 | /v         | 插入版本号, 类似 v2019.01.15 这样的版本号     |

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

<!-- 模块文件名：应用-mstsc远程桌面增强.ahk-->

### MSTSC 远程桌面增强

| 模式     | 按键                    | 功能说明                                                         |
| -------- | :---------------------- | ---------------------------------------------------------------- |
| 任意窗口 | 双击左边 Shift+Ctrl+Alt | 后置当前窗口，用于远程桌面与虚拟机，使其可与本机桌面窗口同时显示 |

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

<!-- 模块文件名：插件-雪星转屏.ahk-->

### 雪星转屏模块

功能：同步旋转你所有的屏幕，自动对齐屏幕边界，不会错位

使用方式如下

| 模式 | 按键                                 | 功能                           |
| ---- | ------------------------------------ | ------------------------------ |
| 全局 | CapsLockX + Alt + 方向键 上 下 左 右 | 同时旋转所有屏幕到你指定的方向 |
<!-- 结束：抽取模块帮助 -->

## 过去与未来第四

### 制作背景 ( 2017 年秋)

> 本人比较经常写代码…
> 起初我是右鼠……后来觉得鼠标放右边有点远……改成了左鼠
> 左鼠之后发现手还是要离开键盘……于是做了个 WASD 模拟鼠标的东西。（然后就能一直用右手托着下巴玩电脑了）
> 后来写的脚本越来越多，就把其中一些常用的放到一起加载……

### 发展路线

本项目的核心理念是：简化系统操作逻辑，提升操作效率，且尽量不与原有习惯键位冲突。

1. [x] 按 CapsLockX + / 键显示对应帮助（目前的显示样式相当草率）
2. [ ] 自动更新（虽然git pull一下也不是不行）
3. [ ] 初次使用上手教程（这个真没有……）
4. [ ] 插件管理器（虽然文件系统也可以搞定）
5. [ ] 自动配置同步功能（虽然一般来说扔onedrive就够）
6. [ ] 易用的选项配置的 UI 界面（虽然改ini也不是什么难事）
7. [ ] 执行外部代码（Python、Nodejs、外部AHK、Bash、……）（虽然写个脚本run一下也并不算麻烦）

如果你有任何想法或建议，请在这里提出：
[Issues · snomiao/CapslockX](https://github.com/snolab/CapsLockX/issues)

### 组合键含义设计

Win + 系列 通常为操作系统功能、桌面窗口应用进程管理等、输入法、输出设备（显示器、多屏）管理

Alt + 系列 通常表述为调用应用内功能，其含义应相当于按下功能相同的按钮，或跳转到特定功能界面。

Ctrl + 系列 同上，但使用上更为频繁、且很可能不存在功能相同的按钮。

Ctrl + Alt + 同上，但一般为全局热键

而 Shift 键 用来在以上功能的基础上稍微改变按键的含义（例如反向操作如Shift+Alt+Tab，或功能范围扩大如Shift+方向键调整选区等）

### 本项目与类似项目的功能对比 / 更新于(20200627) 其中的信息可能慢慢过时

| 功能\项目        | [CapsLockX](https://github.com/snolab/CapsLockX) | [Vonng/CapsLock](https://github.com/Vonng/CapsLock) | [coralsw/CapsEz](https://github.com/coralsw/CapsEz) | [CapsLock+](https://capslox.com/capslock-plus/) |
| :--------------- | :----------------------------------------------- | :-------------------------------------------------- | :-------------------------------------------------- | :---------------------------------------------- |
| 鼠标模拟         | ✅ 流畅完整                                       | ✅ 无滚轮                                            | 🈚 无                                                | 🈚 无                                            |
| 表达式计算       | ✅ Nodejs 或 JScript                              | 🈚 无                                                | 🈚 无                                                | ✅ TabScript (Snippet + Javascript)              |
| 窗口管理         | ✅ 强                                             | ✅ 有                                                | ✅ 有                                                | ✅ 强                                            |
| 虚拟桌面管理     | ✅ 有                                             | 🈚 无                                                | 🈚 无                                                | 🈚 无                                            |
| 编辑增强         | ✅ 有（抛物模型）                                 | ✅ 有                                                | ✅ 有                                                | ✅ 有（很全）                                    |
| 绿色免安装       | ✅ 是                                             | ✅ 是                                                | ✅ 是                                                | ✅ 是                                            |
| 增强媒体键       | 不全                                             | ✅ 全                                                | 🈚 无                                                | 🈚 无                                            |
| 强化的剪贴板     | 弱                                               | 🈚 无                                                | 🈚 无                                                | ✅ 有                                            |
| 快速启动应用     | ✅ 插件                                           | ✅ 有                                                | ✅ 有                                                | ✅ 有                                            |
| 应用功能增强     | ✅ 丰富                                           | 🈚 无                                                | ✅ 有                                                | 🈚 无                                            |
| Bash 控制        | 🈚 无                                             | ✅ 有                                                | 🈚 无                                                | 🈚 无                                            |
| 快速启动语音输入 | ✅ 讯飞                                           | 🈚 无                                                | 🈚 无                                                | 🈚 无                                            |
| 快速输入时间日期 | ✅ 有                                             |                                                     | ✅ 有                                                |                                                 |
| 窗口绑定到热键   | 🈚 无                                             | 🈚 无                                                | 🈚 无                                                | ✅ 有                                            |
| 快速旋转屏幕     | ✅ 有                                             | 🈚 无                                                | 🈚 无                                                | 🈚 无                                            |
| 二次开发         | ✅ 文档友好                                       | ✅ 可                                                | ✅ 可                                                | ✅ 可                                            |
| 内存占用         | ✅ 约 2~3M                                        |                                                     |                                                     |                                                 |
| 模块化           | ✅                                                | 🈚 无                                                | 🈚 无                                                | 🈚 无                                            |
| 系统             | Win                                              | Mac（主），Win（次）                                | Win                                                 | Win, [Mac](https://capslox.com/)                |
| 支持语言         | 中文                                             | 中文 / English                                      | 中文                                                | 中文 / English                                  |

#### 本项目地址

以下几个仓库同步：

- GitHub: [https://github.com/snolab/CapsLockX](https://github.com/snolab/CapsLockX)
- Gitee: [https://gitee.com/snomiao/CapslockX](https://gitee.com/snomiao/CapslockX)
- Bitbucket: [https://bitbucket.org/snomiao/capslockx](https://bitbucket.org/snomiao/capslockx)

#### 相似项目地址

- 源码：[Vonng/CapsLock: Make CapsLock Great Again!](https://github.com/Vonng/CapsLock)
  设计：[Capslock/design.md at master · Vonng/Capslock](https://github.com/Vonng/Capslock/blob/master/design.md)
- [coralsw/CapsEz: KeyMouse Tools](https://github.com/coralsw/CapsEz)
- [CapsLock+](https://capslox.com/CapsLock-plus/)
- [Capslox](https://capslox.com/cn/)
- CapsLock++ [matrix1001/CapsLock-plus-plus: ⌨Amazing, extendable, readable autohotkey scripts framework utilized by CapsLock.](https://github.com/matrix1001/CapsLock-plus-plus)

#### 其它效率软件推荐

- [Quicker](https://getquicker.net/) 也是一个提高电脑操作效率的软件，与本项目可以互补。<!-- （雪星的推荐码： 55396-2857） -->
- [Everything](https://www.voidtools.com/zh-cn/)

## 答疑相关第五

本项目使用协议： [GNU 通用公共许可证 v3.0 - GNU 工程 - 自由软件基金会](https://www.gnu.org/licenses/gpl-3.0.html)。

相关社群：

- CapsLockX 用户电报群：[t.me/CapsLockX_users](https://t.me/CapsLockX_users)
- CapsLockX 用户 QQ 群：[100949388](https://jq.qq.com/?_wv=1027&k=56lsK8ko)
- QZ/VimD/TC/AHK QQ 群： 271105729
- AHK 高级 QQ 群： 717947647
- [本项目的 issues （可作论坛使用）](https://github.com/snolab/CapsLockX/issues)

本项目相关答疑直接进群 [@雪星](tencent://message?uin=997596439) 或私聊提问也可。

## 支持第六

如何帮助本项目生存下去？如果本项目有帮助到你：

1. 欢迎在 Github 上点星
2. 欢迎把我转发分享给你身边的朋友们。
3. 欢迎帮我翻译 readme.md 到各国语言。
4. 欢迎提交 bug、提出完善建议 [issues](https://github.com/snolab/CapsLockX/issues)
5. 欢迎提交代码 PR，哪怕是修改错别字也是可以的～
6. 欢迎在此捐助本项目的开发，每一笔捐赠都会记录到下方的列表中：
   - 爱发电：[https://afdian.net/@snomiao](https://afdian.net/@snomiao)
   - PAYPAL: [https://paypal.me/snomiao](https://paypal.me/snomiao)
   - 支付宝捐助账号： [snomiao@gmail.com （点击查看二维码）](./支付宝捐助.png)

你的支持会实际地，帮助到本项目的未来开发工作，开发计划见此：[发展路线](#发展路线)

### 捐赠记录

暂无

### 鸣谢

- 感谢 [秦金伟](http://rsytes.coding-pages.com/) 的引用推荐文章、和发展建议：[2020-02-23 当键盘模拟鼠标 - 简书](https://www.jianshu.com/p/f757f56a7de6)
- 感谢 @河许人 帮助推广： [CapsLockX – 像黑客一样操作电脑！【雪星】 – AutoAHK]( https://www.autoahk.com/archives/34996 )

### 相关话题

- [(10) What are some good career alternatives for a computer programmer with RSI? - Quora]( https://www.quora.com/Repetitive-Strain-Injury-RSI/What-are-some-good-career-alternatives-for-a-computer-programmer-with-RSI )
- [如何将电脑桌面划分为独立的两半？ - 知乎]( https://www.zhihu.com/questionz/23443944/answer/1670521971 )
- [有哪位残友用的是单手键盘？ - 知乎]( https://www.zhihu.com/question/50621709/answer/1681247637 )
- [(5 封私信 / 50 条消息) 怎么样才能只用键盘不用鼠标，包括任何指针触控设备，并优雅地使用电脑？ - 知乎]( https://www.zhihu.com/question/21281518/answer/1770669886 )
- [(5 封私信 / 50 条消息) 如何将电脑桌面划分为独立的两半？ - 知乎]( https://www.zhihu.com/question/23443944/answer/1670521971 )
- [我是职场达人，AutoHotKey让我成为职场超人 - 知乎]( https://zhuanlan.zhihu.com/p/60372361 )
- [AutoHotKey 中文网专栏 - 知乎]( https://www.zhihu.com/column/autoahk )