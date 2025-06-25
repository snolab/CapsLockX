# 窗口增强插件 (CLX + 1234567890 ZXCV)

## 功能简述

用好 Win 10 自带的 10 个虚拟桌面豪华配置、多显示器自动排列窗口、半透明置顶、（注：任务栏和 AltTab 相关功能暂不兼容 Win11，窗口排列功能正常。）

1. 窗口切换：`CapsLockX + [Shift] + Z`
2. 窗口关闭：`CapsLockX + [Shift] + X`
3. 窗口排列：`CapsLockX + [Shift] + C`
4. 窗口置顶：`CapsLockX + [Shift] + V`
5. 左手窗口管理：在 `Alt + Tab` 的界面，用 `WASD` 切换窗口，`X` 关掉窗口。
6. 高效使用虚拟桌面：`CapsLockX + 0123456789` 切换、增减虚拟桌面，加上 `Shift` 键可以转移当前窗口
7. 虚拟机与远程桌面快速脱离：双击左边 `Shift + Ctrl + Alt`。

## 效果图

- Alt + Tab 管理窗口增强
  ![02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif](./02-插件-窗口增强_Alt+Tab+WASD管理窗口.gif)
- CapsLockX + C 一键排列窗口（这 GIF 是旧版本录的看起来比较卡，新版本优化过 API 就不卡了）
  ![02-插件-窗口增强_一键排列窗口.gif](./02-插件-窗口增强_一键排列窗口.gif)

## 使用方法如下 ( Alt+Tab 与 CapsLockX )

| 作用域       | 窗口增强模块                          | 说明                                       |
| ------------ | ------------------------------------- | ------------------------------------------ |
| Alt+Tab 界面 | `Q E`                                 | 左右切换多桌面                             |
| Alt+Tab 界面 | `W A S D`                             | 上下左右切换窗口选择                       |
| Alt+Tab 界面 | `X C`                                 | 关闭选择的窗口（目前 X 和 C 没有区别）     |
| Win+Tab 视图 | `Alt + W A S D`                       | 切换窗口选择                               |
| 全局         | `Win + [Shift] + B`                   | 定位到托盘任务(windows 系統自帯熱鍵)       |
| 全局         | `Win + [Shift] + T`                   | 定位到任務栏任务(windows 系統自帯熱鍵)     |
| 全局         | `Win + Shift + hjkl`                  | 在窗口之间按方向切换焦点                   |
| 任务栏       | `Ctrl + W 或 Delete`                  | 在托盘图标或任务栏任务上，选择退出按钮     |
| 全局         | `CapsLockX + 1 2 ... 9 0`             | 切换到第 1 .. 12 个桌面                    |
| 全局         | `CapsLockX + Shift + 1 2 ... 9 0 - =` | 把当前窗口移到第 n 个桌面(如果有的话)      |
| 全局         | `CapsLockX + Alt + Backspace`         | 删除当前桌面（会把所有窗口移到上一个桌面） |
| 全局         | `CapsLockX + C`                       | 快速排列当前桌面的窗口                     |
| 全局         | `CapsLockX + Ctrl + C`                | 快速排列当前桌面的窗口（包括最小化的窗口） |
| 全局         | `CapsLockX + Shift + C`               | 快速堆叠当前桌面的窗口                     |
| 全局         | `CapsLockX + Shift + Ctrl + C`        | 快速堆叠当前桌面的窗口（包括最小化的窗口） |
| 全局         | `CapsLockX + Z`                       | 循环切到最近使用的窗口                     |
| 全局         | `CapsLockX + Shift + Z`               | 循环切到最不近使用的窗口                   |
| 全局         | `CapsLockX + X`                       | 关掉当前标签页 Ctrl+W                      |
| 全局         | `CapsLockX + Shift + X`               | 关掉当前窗口 Alt+F4                        |
| 全局         | `CapsLockX + V`                       | 让窗口透明                                 |
| 全局         | `CapsLockX + Shift + V`               | 让窗口保持透明（并置顶）                   |
| 任意窗口     | `双击左边 Shift+Ctrl+Alt`             | 后置当前窗口， \* 见下方注                 |

\*注： 双击左边 Shift+Ctrl+Alt 设计用于远程桌面与虚拟机，使其可与本机桌面窗口同时显示。
例如 mstsc.exe、TeamViewer、VirtualBox、HyperV、VMWare 等远程桌面或虚拟机程序，配合 CapsLockX + Shift + V 透明置顶功能，让你在 Windows 的界面上同时使用 Linux 界面或 MacOS 界面再也不是难题。

此处借用 [@yangbin9317 的评论](https://v2ex.com/t/772052#r_10458792)

> 以 CapsLock 为抓手,打通底层逻辑,拉齐 Windows 和 Linux WM,解决了 Windows 难用的痛点

(20220313) 对于两端都是 Windows 的情况，也可以考虑使用 [RemoteApp Tool - Kim Knight](http://www.kimknight.net/remoteapptool) 来代替远程桌面。
