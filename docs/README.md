# CapsLockX - 💻 Get Hacker's Keyboard. 像**黑客**一样操作电脑

CapsLockX is a modular hotkey script engine based on AutoHotkey. Allows you to easily operate the computer efficiently like a hacker in a movie without leaving the keyboard with both hands . There are a lot of functions that you can understand at a touch and are super easy to use: editing enhancement, virtual desktop and window management, mouse simulation, in-app hotkey enhancement, JS mathematical expression calculation, and other super multi-functions are waiting for you to personally define.

**[See English Docs (Google Translated)](https://capslockx.snomiao.com/)**

---

CapsLockX 是一款基于 AutoHotkey 的模块化热键脚本引擎。 让你可以轻轻松松像电影里的**黑客**一样，双手不离开键盘，**高效率**地操作电脑。这里有超多一摸就懂超好上手的功能：编辑增强、虚拟桌面与窗口管理、鼠标模拟、应用内热键增强、JS 数学表达式计算、等超多功能等你来亲自定义。主仓库地址 🏠：[https://github.com/snolab/CapsLockX](https://github.com/snolab/CapsLockX)

<!-- clicks / downloads -->

[![jsdelivr_NPM](https://data.jsdelivr.com/v1/package/npm/capslockx/badge)](https://www.jsdelivr.com/package/npm/capslockx)
[![jsdelivr_GITHUB](https://data.jsdelivr.com/v1/package/gh/snolab/capslockx/badge)](https://www.jsdelivr.com/package/gh/snolab/capslockx)
[![Downloads-From-GitHub-Releases](https://img.shields.io/github/downloads/snolab/CapsLockX/total.svg?style=flat-square&label=Downloads-From-GitHub-Releases)](https://github.com/snolab/CapsLockX/releases)

<!-- building status -->

[![gh-pages](https://github.com/snolab/CapsLockX/actions/workflows/release-github.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/release-github.yml)
[![NPM](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml)

<!-- [![Chocolatey](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml) -->
<!-- [![Packages Test](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml) -->

## 入门教程 📖

### 太长不看 / TL;DR 📄

下载这个压缩包： [下载 JSDelivrCDN-发布程序包.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)

解压之后，打开 `CapsLockX.exe`，过掉简单的新手教程，然后，按住 CapsLockX，然后按 `WASD` 鼠标移动，`QE` 点击 `RF` 滚轮，`HJKL` 光标移动，`YOUI` 页面移动，`ZXCV` 窗口管理，`1234567890` 切换虚拟桌面，`M` 打开配置。

### 安装与使用 🛠

#### 绿色便携程序包（新手适用，稳定版） 📦

源码打包就是软件本体，无需编译，纯绿色便携软件解压即用。源码 + 程序包，推荐第一个（最快）。

1. [下载 JSDelivrCDN-发布程序包.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)
1. [备选下载 CloudFlareCDN-发布程序包.zip](https://capslockx.snomiao.com/CapsLockX-latest.zip)
1. [备选下载 GitHub-发布程序包.zip](https://github.com/snolab/CapsLockX/raw/gh-pages/CapsLockX-latest.zip)
1. [备选下载 GitHub-仓库程序包.zip](https://github.com/snolab/CapsLockX/archive/master.zip)
1. [备选下载 BitBucket-仓库程序包.zip](https://bitbucket.org/snomiao/capslockx/get/master.zip)
1. [备选下载 中国大陆用户-Gitee-仓库程序包.zip （需登录）](https://gitee.com/snomiao/CapslockX/repository/archive/master.zip)

解压后使用即可，启动与自启动的方法： 双击 `CapsLockX.exe` 即可启动脚本，如需添加启动项，请在开始菜单 - 运行输入 shell:startup 然后给本程序创建快捷方式，扔进去就可以了。

#### 命令行方式安装（进阶用户推荐，可自动更新）🖥️

以下几种任选其一，其它地区用户推荐第 2 个

1. `npx capslockx`, -- NPX 直接运行，可以一直运行最新版，推荐（需要安装 NodeJS ）
2. `choco update capslockx && capslockx` -- [Chocolatey](https://community.chocolatey.org/packages/CapsLockX/) 安装可使用 cup 自动更新，推荐
3. `npm i -g capslockx && npx capslockx` -- npm 全局安装
4. `git clone https://gitee.com/snomiao/CapslockX && .\CapsLockX\CapsLockX.exe` -- 大陆源码包（绿色软件包）解压使用，中国大陆地区用户推荐
5. `git clone https://github.com/snolab/CapsLockX && .\CapsLockX\CapsLockX.exe` -- github 源码包（绿色软件包）解压使用
6. `winget capslockx` -- TODO
7. `scoop capslockx` -- TODO

## 使用手册 📖

<!-- * 按 `CapsLock` 切换 CapsLockX 模式 -->

- 按住 `CapsLockX` 进入 CapsLockX 模式，此时你的键盘会成为像 Vim 默认模式那样的功能键盘，（键位见下
- 长按 `CapsLockX` 键将锁定 `CLX` 模式，此时弹起 `CapsLockX` 键将保持 `CLX` 到下一次按下 `CaspLockX` 键为止。[功能由来](https://github.com/snolab/CapsLockX/issues/21)

CapsLockX 默认加载了一些常用的模块，功能与使用方法已在下方列出。
对于不需要的模块，你也可以直接删除 `./Modules` 目录下对应的 `.ahk` 文件，然后按 `Ctrl + Alt + \` 重新加载即可。

你也可以编写自己的 `my-ahk.user.ahk` 然后放到 `./User/` 目录下，CapsLockX 会自动识别并加载它们。

<!-- 下面这堆东西是自动从各个模块里抽取的，如需改动请到对应模块.md 里操作, 在这里修改会被覆盖 -->
<!-- 开始：抽取模块帮助 -->

<!-- 结束：抽取模块帮助 -->

## 过去与未来 🛰

### 制作背景 ( 2017 年秋) 🍁

> 本人比较经常写代码…
> 起初我习惯右手用鼠标……后来觉得鼠标放右边有点远……改成了左手用鼠标
> 左手用鼠标之后发现手还是要离开键盘……于是做了个 WASD 模拟鼠标的脚本。（然后就能一直用右手托着下巴玩电脑了）
> 后来写的脚本越来越多，就把其中一些常用的放到一起加载……

### 发展路线 🛰️

本项目的核心理念是：简化系统操作逻辑，提升操作效率，且尽量不与原有习惯键位冲突。

1. [x] 按 CapsLockX + / 键显示对应帮助（目前的显示样式相当草率）
2. [ ] 自动更新（虽然 git pull 一下也不是不行）
3. [ ] 初次使用上手教程（这个现在有点简陋……）
4. [ ] 插件管理器（虽然文件系统也可以搞定）
5. [ ] 自动配置同步功能（虽然一般来说扔 onedrive 就够）
6. [ ] 易用的选项配置的 UI 界面（虽然改 ini 也不是什么难事）
7. [ ] 执行外部代码（Python、Nodejs、外部 AHK、Bash、……）（虽然写个脚本 run 一下也并不算麻烦）

如果你有任何想法或建议，请在这里提出：
[Issues · snomiao/CapslockX](https://github.com/snolab/CapsLockX/issues)

### 组合键含义设计

Win + 系列 通常为操作系统功能、桌面窗口应用进程管理等、输入法、输出设备（显示器、多屏）管理

Alt + 系列 通常表述为调用应用内功能，其含义应相当于按下功能相同的按钮，或跳转到特定功能界面。

Ctrl + 系列 同上，但使用上更为频繁、且很可能不存在功能相同的按钮。

Ctrl + Alt + 同上，但一般为全局热键

而 Shift 键 用来在以上功能的基础上稍微改变按键的含义（例如反向操作如 Shift+Alt+Tab，或功能范围扩大如 Shift+方向键调整选区等）

### 本项目与类似项目的功能对比 / 更新于(20200627) 其中的信息可能慢慢过时

| 功能\项目        | [CapsLockX](https://github.com/snolab/CapsLockX) | [Vonng/CapsLock](https://github.com/Vonng/CapsLock) | [coralsw/CapsEz](https://github.com/coralsw/CapsEz) | [CapsLock+](https://capslox.com/capslock-plus/) |
| :--------------- | :----------------------------------------------- | :-------------------------------------------------- | :-------------------------------------------------- | :---------------------------------------------- |
| 鼠标模拟         | ✅ 流畅完整                                      | ✅ 无滚轮                                           | 🈚 无                                               | 🈚 无                                           |
| 表达式计算       | ✅ Nodejs 或 JScript                             | 🈚 无                                               | 🈚 无                                               | ✅ TabScript (Snippet + Javascript)             |
| 窗口管理         | ✅ 强                                            | ✅ 有                                               | ✅ 有                                               | ✅ 强                                           |
| 虚拟桌面管理     | ✅ 有                                            | 🈚 无                                               | 🈚 无                                               | 🈚 无                                           |
| 编辑增强         | ✅ 有（抛物模型）                                | ✅ 有                                               | ✅ 有                                               | ✅ 有（很全）                                   |
| 绿色免安装       | ✅ 是                                            | ✅ 是                                               | ✅ 是                                               | ✅ 是                                           |
| 增强媒体键       | 不全                                             | ✅ 全                                               | 🈚 无                                               | 🈚 无                                           |
| 强化的剪贴板     | 弱                                               | 🈚 无                                               | 🈚 无                                               | ✅ 有                                           |
| 快速启动应用     | ✅ 插件                                          | ✅ 有                                               | ✅ 有                                               | ✅ 有                                           |
| 应用功能增强     | ✅ 丰富                                          | 🈚 无                                               | ✅ 有                                               | 🈚 无                                           |
| Bash 控制        | 🈚 无                                            | ✅ 有                                               | 🈚 无                                               | 🈚 无                                           |
| 快速启动语音输入 | ✅ 讯飞                                          | 🈚 无                                               | 🈚 无                                               | 🈚 无                                           |
| 快速输入时间日期 | ✅ 有                                            |                                                     | ✅ 有                                               |                                                 |
| 窗口绑定到热键   | 🈚 无                                            | 🈚 无                                               | 🈚 无                                               | ✅ 有                                           |
| 快速旋转屏幕     | ✅ 有                                            | 🈚 无                                               | 🈚 无                                               | 🈚 无                                           |
| 二次开发         | ✅ 文档友好                                      | ✅ 可                                               | ✅ 可                                               | ✅ 可                                           |
| 内存占用         | ✅ 约 2~3M                                       |                                                     |                                                     |                                                 |
| 模块化           | ✅                                               | 🈚 无                                               | 🈚 无                                               | 🈚 无                                           |
| 系统             | Win                                              | Mac（主），Win（次）                                | Win                                                 | Win, [Mac](https://capslox.com/)                |
| 支持语言         | 中文                                             | 中文 / English                                      | 中文                                                | 中文 / English                                  |

#### 本项目地址 🔗

以下几个仓库同步更新：

- GitHub: [https://github.com/snolab/CapsLockX](https://github.com/snolab/CapsLockX)
- Gitee: [https://gitee.com/snomiao/CapslockX](https://gitee.com/snomiao/CapslockX)
- Bitbucket: [https://bitbucket.org/snomiao/capslockx](https://bitbucket.org/snomiao/capslockx)
- Gitlab: [https://gitlab.com/snomiao/CapsLockX/](https://gitlab.com/snomiao/CapsLockX/)

文档地址 📄

- 自动翻译文档 Netlify CDN：[https://capslockx.netlify.com](https://capslockx.netlify.com)
- 自动翻译文档 CloudFlare CDN：[https://capslockx.snomiao.com](https://capslockx.snomiao.com)
- 自动翻译文档 CloudFlare CDN：[https://capslockx.snomiao.com](https://capslockx.snomiao.com)

星图 ⭐️

- [![Stargazers over time](https://starchart.cc/snolab/CapsLockX.svg)](https://starchart.cc/snolab/CapsLockX)

#### 相似项目地址 🔗

- [Star Historys](https://star-history.t9t.io/#snolab/CapsLockX&wo52616111/capslock-plus&coralsw/CapsEz&Vonng/CapsLock)
- 源码：[Vonng/CapsLock: Make CapsLock Great Again!](https://github.com/Vonng/CapsLock)
  设计：[Capslock/design.md at master · Vonng/Capslock](https://github.com/Vonng/Capslock/blob/master/design.md)
- [coralsw/CapsEz: KeyMouse Tools](https://github.com/coralsw/CapsEz)
- [CapsLock+](https://capslox.com/CapsLock-plus/)
- [Capslox](https://capslox.com/cn/)
- CapsLock++ [matrix1001/CapsLock-plus-plus: ⌨Amazing, extendable, readable autohotkey scripts framework utilized by CapsLock.](https://github.com/matrix1001/CapsLock-plus-plus)

## 答疑相关 ❓

本项目使用协议： [GNU 通用公共许可证 v3.0 - GNU 工程 - 自由软件基金会](https://www.gnu.org/licenses/gpl-3.0.html)。

相关社群：

- [本项目的 issues （可作论坛使用）](https://github.com/snolab/CapsLockX/issues) ✉️
- CapsLockX 用户电报群：[t.me/CapsLockX_users](https://t.me/CapsLockX_users)📱
- CapsLockX 用户 QQ 群 🐧：[100949388](https://jq.qq.com/?_wv=1027&k=56lsK8ko)
- QZ/VimD/TC/AHK QQ 群 🐧： 271105729
- AHK 高级 QQ 群 🐧： 717947647

本项目相关答疑直接进群 [@雪星](tencent://message?uin=997596439) 或私聊提问也可。

## 支持 ⭐️

如何帮助本项目生存下去？如果本项目有帮助到你：

1. 欢迎在 Github 上点星 ⭐️
2. 欢迎把我转发分享给你身边的朋友们。
3. 欢迎帮我翻译 readme.md 到各国语言。 🌐
4. 欢迎提交 bug、提出完善建议 [issues](https://github.com/snolab/CapsLockX/issues) 🐞
5. 欢迎提交代码 PR，哪怕是修改错别字也是可以的～
6. 欢迎创作关于本软件的作品，比如录制使用教学视频投稿到 Youtube 或 Bilibili ，雪星会去给你点赞的哦。
7. 欢迎在此捐助本项目的开发，每一笔捐赠都会记录到下方的列表中：💰
   - 爱发电 ⚡️：[https://afdian.net/@snomiao](https://afdian.net/@snomiao)
   - PAYPAL: [https://paypal.me/snomiao](https://paypal.me/snomiao)
   - 支付宝捐助账号： [snomiao@gmail.com （点击查看二维码）](./支付宝捐助.png)

[发展路线](#发展路线)

### 捐赠记录(截至 20210821) 📄

| 捐赠时间 | 名称   | 渠道   | 金额       | 备注                     |
| -------- | ------ | ------ | ---------- | ------------------------ |
| 20210619 | \*\*煜 | 支付宝 | +50.00 CNY | 小小資助，支持獨立開發者 |

### 鸣谢 🙏🏻

- 感谢来自以上捐赠者的经济支持。
- 感谢 [秦金伟](http://rsytes.coding-pages.com/) 的引用推荐文章、和发展建议：[2020-02-23 当键盘模拟鼠标 - 简书](https://www.jianshu.com/p/f757f56a7de6)
- 感谢 @河许人 帮助转载推广： [CapsLockX – 像黑客一样操作电脑！【雪星】 – AutoAHK](https://www.autoahk.com/archives/34996)
- 感谢在 issues 里和群里提问并帮助完善本项目的各位。

### 相关话题

- [CapsLockX - 像黑客一样操作电脑 - V2EX](https://v2ex.com/t/772052#reply1)
- [CapsLockX - 像黑客一样操作电脑！ - AutoHotkey Community](https://www.autohotkey.com/boards/viewtopic.php?f=28&t=88593)
- [(10) What are some good career alternatives for a computer programmer with RSI? - Quora](https://www.quora.com/Repetitive-Strain-Injury-RSI/What-are-some-good-career-alternatives-for-a-computer-programmer-with-RSI)
- [如何将电脑桌面划分为独立的两半？ - 知乎](https://www.zhihu.com/questionz/23443944/answer/1670521971)
- [有哪位残友用的是单手键盘？ - 知乎](https://www.zhihu.com/question/50621709/answer/1681247637)
- [(5 封私信 / 50 条消息) 怎么样才能只用键盘不用鼠标，包括任何指针触控设备，并优雅地使用电脑？ - 知乎](https://www.zhihu.com/question/21281518/answer/1770669886)
- [(5 封私信 / 50 条消息) 如何将电脑桌面划分为独立的两半？ - 知乎](https://www.zhihu.com/question/23443944/answer/1670521971)
- [我是职场达人，AutoHotKey 让我成为职场超人 - 知乎](https://zhuanlan.zhihu.com/p/60372361)
- [AutoHotKey 中文网专栏 - 知乎](https://www.zhihu.com/column/autoahk)
