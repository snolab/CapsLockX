# CapsLockX - 💻 Get Hacker's Keyboard. Operate your computer like a **hacker**

CapsLockX is a modular hotkey script engine based on AutoHotkey. It enables you to effortlessly operate your computer with high efficiency, like a hacker in a movie, without taking your hands off the keyboard. There are numerous intuitive functions that are super easy to learn: editing enhancement, virtual desktop and window management, mouse simulation, in-app hotkey enhancement, JS mathematical expression calculation, and many other multifunctional features waiting for you to define them personally.

**[See English Docs (Google Translated)](https://capslockx.snomiao.com/)**

---

CapsLockX is a modular hotkey script engine built upon AutoHotkey, enabling you to operate your computer with ease, just like hackers from movies, without taking your hands off the keyboard. It incorporates a wide array of instantly intuitive and easily accessible functions, such as enhanced editing, management of virtual desktops and windows, mouse simulation, in-application hotkey improvements, calculation of JavaScript mathematical expressions, and many other highly versatile features for you to define on your own. Primary repository address 🏠: [https://github.com/snolab/CapsLockX](https://github.com/snolab/CapsLockX)

---

## Version Wall - Badge Wall 📛 Badges

<!-- culture badges -->

[![Chinese Programming](https://github.com/Program-in-Chinese/overview/raw/master/%E4%B8%AD%E6%96%87%E7%BC%96%E7%A8%8B.svg)](https://github.com/Program-in-Chinese/overview),
[![996.icu](https://img.shields.io/badge/link-996.icu-red.svg)](https://996.icu)
[![GitHub license](https://img.shields.io/github/license/snolab/CapsLockX)](https://github.com/snolab/CapsLockX/blob/master/LICENSE.md)
![GitHub top language](https://img.shields.io/github/languages/top/snolab/CapsLockX)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/snolab/CapsLockX)
[![GitHub stars](https://img.shields.io/github/stars/snolab/CapsLockX)](https://github.com/snolab/CapsLockX/stargazers)

<!-- build and publish status -->

![GitHub release (latest by date)](https://img.shields.io/github/v/release/snolab/CapsLockX)
[![gh-pages](https://github.com/snolab/CapsLockX/actions/workflows/gh-pages-release.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/gh-pages-release.yml)
![GitHub all releases](https://img.shields.io/github/downloads/snolab/CapsLockX/total)
[![jsdelivr_GITHUB](https://data.jsdelivr.com/v1/package/gh/snolab/capslockx/badge)](https://www.jsdelivr.com/package/gh/snolab/capslockx)

[![npm](https://img.shields.io/npm/v/capslockx)](https://www.npmjs.com/capslockx)
[![npm publish](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/npm-publish.yml)
![npm](https://img.shields.io/npm/dt/capslockx)
![jsDelivr hits (npm)](https://img.shields.io/jsdelivr/npm/hy/capslockx)

[![Chocolatey version](https://img.shields.io/chocolatey/v/capslockx)](https://community.chocolatey.org/packages/CapsLockX/)
[![Chocolatey Publish](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/choco-push.yml)
[![Chocolatey Downloads](https://img.shields.io/chocolatey/dt/CapsLockX)](https://community.chocolatey.org/packages/CapsLockX/)

<!-- [![Packages Test](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml/badge.svg)](https://github.com/snolab/CapsLockX/actions/workflows/package-test.yml) -->

---

## Beginner's Quick Start Tutorial 📖 Tutorial

### Easy Start Tutorial (Completing this section qualifies you as a CapsLockX user)

CapsLockX has four core functionalities: **window management**, **mouse simulation**, **direction key simulation**, and in-app hotkeys. This introductory tutorial will teach you the first three core functionalities.

First, get CapsLockX: download this zip file: [Download JSDelivrCDN Release Package.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)

After extracting, open `CapsLockX.exe` in the CapsLockX folder, pass the simple beginner's tutorial, and then test CapsLockX's functions in the order of the following left and right hand functional areas.

After starting CapsLockX, it does **not affect** the functions of other keys on your keyboard. The following functions are only triggered when you press `CapsLockX + Combination Key`.

Left hand function area:

- Window management: `CapsLockX + 1234567890` switches to the `nth` virtual desktop, `CapsLockX + ZXCV` performs window operations (window switching, closing, arranging, transparency, and pinning on top).
- Mouse simulation: Press `CapsLockX + WASD` to move the mouse (as simple as operating character movement in a game), press `CapsLockX + QE` for mouse left and right clicks, `CapsLockX + RF` for mouse wheel up and down scrolling.

Right hand function area:

- Direction key simulation: Open any editor (such as Notepad), press `HJKL` to move the cursor, `YOUI` to scroll the page.

After you become familiar, if you want to learn more features, please refer to the following quick reference manual.

---

## Advanced Reference Manual 🦽 Manual

### Installation and Use 🛠 Installation

#### Portable Package (Recommended for Beginners, Stable Version) 📦 Packaged Bins

The source code package contains the software itself, requires no compilation, and is purely portable software ready to use immediately after extraction. It includes the source code and program packages, with the first option recommended (fastest).

1. [Download JSDelivrCDN Release Package.zip](https://cdn.jsdelivr.net/gh/snolab/CapsLockX@gh-pages/CapsLockX-latest.zip)
2. [Alternative Download CloudFlareCDN Release Package.zip](https://capslockx.snomiao.com/CapsLockX-latest.zip)
3. [Alternative Download GitHub Release Package.zip](https://github.com/snolab/CapsLockX/raw/gh-pages/CapsLockX-latest.zip)
4. [Alternative Download GitHub Repository Package.zip](https://github.com/snolab/CapsLockX/archive/master.zip)
5. [Alternative Download BitBucket Repository Package.zip](https://bitbucket.org/snomiao/capslockx/get/master.zip)
6. [Alternative Download for Mainland China Users: Gitee Repository Package.zip (Login Required)](https://gitee.com/snomiao/CapslockX/repository/archive/master.zip)

To use after extraction, start and autostart the program by double-clicking `CapsLockX.exe`. To add a startup item, go to the Start Menu - Run, enter shell:startup, and create a shortcut for this program to throw in there.

#### Installation via Command Line (Recommended for Advanced Users, Auto Update Available)🖥️ Install by command

Choose any of the following, with the fourth option recommended for users in Mainland China:

1. `npx capslockx@latest` - Run directly with NPX to always run the latest version, recommended (requires NodeJS installation).
2. `choco update capslockx && capslockx` - Install with [Chocolatey](https://community.chocolatey.org/packages/CapsLockX/) to use cup for automatic updates, recommended.
3. `npm i -g capslockx && npx capslockx` - Global installation with npm.
4. `git clone https://gitee.com/snomiao/CapslockX && .\CapsLockX\CapsLockX.exe` - Green package from China source (recommended for users in Mainland China).
5. `git clone https://github.com/snolab/CapsLockX && .\CapsLockX\CapsLockX.exe` - Green package from GitHub source code (ready to use).
6. `winget capslockx` - TODO #40
7. `scoop capslockx` - TODO #41

## Usage Manual 📖 - Usage Manual

### Basic Operations

- Hold `CapsLockX` to enter CapsLockX mode, and your keyboard will function like a regular keyboard in Vim's default mode (see keymap below).
- Pressing `CapsLockX+Space` at the same time will lock the `CLX` mode, and it will remain in `CLX` until the next time the `CapsLockX` key is pressed. [Feature Origin](https://github.com/snolab/CapsLockX/issues/21)

### Module Explanations

CapsLockX comes with several common modules loaded by default, with functionalities and usage methods listed below.
For modules that you do not need, you can directly delete the corresponding `.ahk` files in the `./Modules` directory and press `Ctrl + Alt + \` to reload.

You can also write your `my-ahk.user.ahk` and place them in the `./User/` directory, and CapsLockX will automatically recognize and load them.

### Nightmares of Multitasking

#### Virtual Desktop Overview: Scenarios, Work Desktop, Entertainment Desktop, Project Classification...

Typically, a user's active set of tasks includes multiple windows, which combined can form a usage scenario, with several scenarios possibly running simultaneously and independently for extended periods without interference. This involves a significant amount of window arrangement and virtual desktop switching operations, where CLX can offer a terrifying boost in efficiency for managing your windows.

The following are examples of scenario combinations: Suppose you can study, work on several different projects, chat with friends, play games, listen to background music, and have a movie paused and ready to watch with family at night.

- Virtual Desktop 1: Planning Scenario: Calendar window + Multi-platform synced notes, e.g., Google Calendar + Notion + Gmail, etc.
- Virtual Desktop 2: Study Scenario: Reading window, note-taking window, e.g., OneNote + Calibre, etc.
- Virtual Desktop 3: Work Scenario 1 (Frontend Development): Code editing + Documentation search + Browser, e.g., Chrome(dev) + VSCode + [stackoverflow](https://stackoverflow.com), etc.
- Virtual Desktop 4: Work Scenario 2 (Backend Development): Code editing + Documentation search + Backend terminal + Database browser, e.g., DBeaver + VSCode(+bash) + [stackoverflow](https://stackoverflow.com), etc.
- Virtual Desktop 5: Work Scenario 3 (Script Development): Code editing + Documentation search + Script target, e.g., VSCode(+bash) + [stackoverflow](https://stackoverflow.com)...
- Virtual Desktop 6: Work Scenario 4 (`3D` Modeling Rendering): `3D` Modeling Software + Material search, e.g., Blender + Chrome
- Virtual Desktop 7: Work Scenario 5 (`3D` Printing Slicing): Slicing software + Model search window, e.g., Cura + [thingiverse](https://thingiverse.com)
- Virtual Desktop 7: Work Scenario 6 (Video Processing): Editing + Material management, e.g., PR + Everything
- Virtual Desktop 7: Work Scenario 7 (Post-Production): Post-production + Documentation tutorial, e.g., AE + Chrome
- Virtual Desktop 8: Writing Scenario: Writing window, material search window, e.g., Obsidian + Chrome (Google Scholar Index), etc.
- Virtual Desktop 9: Communication Scenario 1: Casual chatting, e.g., Telegram + Reddit +
- Virtual Desktop 9: Communication Scenario 2: Work communication, e.g., Slack + Skype + GMail
- Virtual Desktop 9: Communication Scenario 3: Presentation, e.g., Google Meeting + (vscode | page app | requirement document | feedback document)
- Virtual Desktop 0: Entertainment Scenario 1: Gaming, e.g., Age of Empires, Minecraft, Skyrim, Overcooked 2, etc.
- Virtual Desktop 0: Entertainment Scenario 3: Watching movies, listening to music, e.g., PotPlayer, Youtube Music, etc.
- ... More examples are welcome to be added via Issue or PR.

While it is not recommended to handle too many tasks at once, CapsLockX can save your thinking environment and minimize the cognitive cost of switching between tasks, effectively saving you the time spent rearranging windows and the associated loss of attention.

(Note: If you enjoy multitasking, you might need not only a computer with a sizable amount of memory but also a considerably large brain capacity :D )

#### Windows Window Switching User Experience Improvement - Win+tab

When switching windows with Alt+Tab and if there are too many windows, a two-dimensional window preview layout is displayed.
Generally, Alt+Tab and Alt+Shift+Tab are pure left-handed hotkeys. If the user wants to select a window in the next line, they instinctively press Alt+Tab multiple times.
The arrow keys used for two-dimensional operations are often overlooked since the right hand is usually on the mouse or the J key.

Furthermore, users continue to hold the Alt key after releasing the Tab key to browse windows and choose the target window to switch to.
In CLX, the functionality of Alt+WASD is activated in its place to replace the arrow keys, allowing for direct multidirectional window switching with the left hand, eliminating the need to press Shift to reverse.
Additionally, if the user needs to close multiple windows, pressing Alt+X can close several target windows while remaining in the window browsing interface.

In CLX, these functions greatly enhance the usability of Alt+Tab.

#### TODO-Docs

<details>
<summary>Click to expand TODO-docs</summary>

#### Focus Quantity: Active Window, Default Active Window, ...

Each desktop has only one active focus window, but virtual desktops can automate switching to that virtual desktop's focus window upon switching, maintaining multiple task focuses (i.e., active windows).

#### Multi-screen Usage - Multi-screening

#### Window Arrange in 4K Era - Window arrange with 4k screen

Limitations of Windows 10's default window arrangement:

1. Not applicable to multiple desktops
2. Unnecessarily large gaps between windows

##### Linux and Mac Window Management - Window Manager in Linux and mac

TODO: i3 window manager

##### Android and iOS Window Management - Window Manage in android

System-level two solutions: left-right-top-bottom split screen, floating window; application level: floating components,

### Editing Troubles

#### The Distance Between Typing Area and Editing Control Area

TODO Discourse on ThinkPad and Mac arrow key positions, inspiration from VIM,

#### The Concept of Chords

TODO Various types of chords

TODO Information capacity increase of chords calculation

### Graphical User Interface Troubles

TODO: Document introduction to mouse simulation features, RPG game movement

### Human Speed Perception

TODO: Exponential growth world perception, focus, hearing, vision, touch, VS regular linear operations

### Software Hotkey Defects

TODO: Introduction to application enhancement module

### Usability of Portable Keyboards

TODO: FN key, arrow keys, editing operations, 61 layout vs 87-key layout,

</details>

<!-- The stuff below is automatically extracted from various modules. If you need to make changes, please go to the respective module.md. Changes here will be overwritten. -->
<!-- Start: Extract module help -->
<!-- Module file name: @Help.ahk-->

### Help Module

If you want to learn how to develop plugins for CapsLockX, please:

1. Open `Modules/@Help.ahk`, and you can learn the basic format of CapsLockX plugins.
2. Make a copy of it, naming it with your own plugin name.
3. Change its original functionality to suit your needs, and your plugin development is complete!

## The functionalities of this module are as follows

| Applies to | Hotkey                | Function                    |
| ---------- | --------------------- | --------------------------- |
| Global     | CapsLockX + /         | Temporarily display hotkeys |
| Global     | CapsLockX + Alt + /   | 🔗 Open full doc page of CapsLockX |
| Global     | CapsLockX + Shift + / | 🕷 Submit bugs, suggestions, etc. |

<!-- Module file name: App-AnkiEnhanced.ahk-->

### Anki Enhancement Module

Enhancements for Anki operations.

## Common Features

1. Use WASD or HJKL for quick and consecutive (and undoable) flashcard switching.
2. In Excel, make a list of words in 2 columns, select all and copy, then press Alt + i in Anki for quick import of the word list.
3. Simplify the 4 options to 3 directional keys, left is easy, down is medium, right is difficult, and up is undo.
4. Can be used with a game controller; configure the joystick with XPadder to map to the arrow keys. See Bilibili video by Snowstar: "How a Chuunibyou Memorizes Words - Why Not Use a Controller to Memorize Words!" (https://www.bilibili.com/video/av8456838/)

## Explanations

| Mode         j         | Anki Enhancement Module | Description                                                  |
| --------------------- | :---------------------: | ------------------------------------------------------------ |
| In Anki-Study Interface | `w or k or ↑`            | Press = undo, release to show answer                         |
| In Anki-Study Interface | `a or h or ←`            | Press = easy, release to show answer                         |
| In Anki-Study Interface | `s or j or ↓`            | Press = normal, release to show answer                      |
| In Anki-Study Interface | `d or l or →`            | Press = unfamiliar, release to show answer                   |
| In Anki-Study Interface |      `q`                | Return to the previous interface                             |
| In Anki-Study Interface |      `c`                | Add a new card                                               |
| In Anki-Study Interface | `1 or NumPad1`          | Difficult (original hotkey)                                  |
| In Anki-Study Interface | `2 or NumPad2`          | Awkward (original hotkey)                                    |
| In Anki-Study Interface | `3 or NumPad3`          | General (original hotkey)                                    |
| In Anki-Study Interface | `4 or NumPad4`          | Smooth (original hotkey)                                     |
| In Anki-Study Interface | `5 or NumPad5`          | Undo                                                          |
| In Anki-Study Interface | `6 or NumPad6`          | Pause card                                                    |
| In Anki-Study Interface |