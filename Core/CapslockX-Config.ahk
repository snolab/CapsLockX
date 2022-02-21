; 保存为 save with UTF8 with DOM

; 用户创建目录
便携版配置目录   = ./User
用户目录配置目录 = %USERPROFILE%/.CapsLockX
APPDATA配置目录  = %APPDATA%/CapsLockX

; 默认值
启动配置目录 := APPDATA配置目录

if ( InStr(FileExist(APPDATA配置目录), "D")) {
    启动配置目录 := APPDATA配置目录
}
if ( InStr(FileExist(用户目录配置目录), "D")) {
    启动配置目录 := 用户目录配置目录
}
if ( InStr(FileExist(便携版配置目录), "D")) {
    启动配置目录 := 便携版配置目录
}
FileCreateDir %启动配置目录%

global CapsLockX_配置目录 := 启动配置目录
; msgbox %CapsLockX_配置目录%
global CapsLockX_配置路径 := CapsLockX_配置目录 "/CapsLockX-Config.ini"

; 初始化配置
if (!CapsLockX_配置路径) {
    Return
}
; 配置文件编码清洗
global CapsLockX_ConfigChangedTickCount
CapsLockX_ConfigChangedTickCount := A_TickCount
; msgbox 清洗为_UTF16_WITH_BOM_型编码1
清洗为_UTF16_WITH_BOM_型编码(CapsLockX_配置路径)
; msgbox 清洗为_UTF16_WITH_BOM_型编码2

CapsLockX_Config("_NOTICE_", "ENCODING_USING", "UTF16_LE", "")
; 基本设定
; [Core]
global T_XKeyAsSpace := CapsLockX_Config("Core", "T_XKeyAsSpace", 1, "使用 Space 作为引导键（默认启用，游戏玩家可在 .user.ignore.txt 里配置忽略游戏窗口）")
global T_XKeyAsCapsLock := CapsLockX_Config("Core", "T_XKeyAsCapsLock", 1, "使用 CapsLock 作为引导键（默认启用）")
global T_XKeyAsInsert := CapsLockX_Config("Core", "T_XKeyAsInsert", 0, "使用 Insert 作为引导键")
global T_XKeyAsScrollLock := CapsLockX_Config("Core", "T_XKeyAsScrollLock", 0, "使用 ScrollLock 作为引导键")
global T_XKeyAsRAlt := CapsLockX_Config("Core", "T_XKeyAsRAlt", 0, "使用 右 Alt 作为引导键")

global T_UseScrollLockLight := CapsLockX_Config("Advanced", "T_UseScrollLockLight", 0, "进阶： 是否使用 ScrollLock 灯来显示 CapsLockX 状态（不建议）")
global T_UseCapsLockLight := CapsLockX_Config("Advanced", "T_UseCapsLockLight", 0, "进阶： 是否使用 CapsLockX 灯来显示 CapsLockX 状态（强烈不建议）")
global T_SwitchSound := CapsLockX_Config("Advanced", "T_SwitchSound", 0, "进阶： 是否开启CapsLockX模式切换声音提示（默认不开）")
global T_SwitchSoundOn := CapsLockX_Config("Advanced", "T_SwitchSoundOn", "./Data/NoteG.mp3", "CapsLockX 按下声音提示路径")
global T_SwitchSoundOff := CapsLockX_Config("Advanced", "T_SwitchSoundOff", "./Data/NoteC.mp3", "CapsLockX 弹起声音提示路径")
; 不同模式下的拖盘图标
; global T_SwitchTrayIconDefault := CapsLockX_Config("Core", "T_SwitchTrayIconDefault", "./Data/XIconWhite.ico", "CapsLockX默认托盘显示图标，默认" "./Data/XIconWhite.ico")
global T_SwitchTrayIconOff := CapsLockX_Config("Advanced", "T_SwitchTrayIconOff", "./Data/XIconWhite.ico", "CapsLockX弹起托盘显示图标，默认为 " "./Data/XIconWhite.ico")
global T_SwitchTrayIconOn := CapsLockX_Config("Advanced", "T_SwitchTrayIconOn", "./Data/XIconBlue.ico", "CapsLockX按下托盘显示图标，默认为 " "./Data/XIconBlue.ico")

CapsLockX_ConfigSet(field, varName, setValue, comment := "")
{
    global CapsLockX_ConfigChangedTickCount
    CapsLockX_ConfigChangedTickCount := A_TickCount
    content := setValue
    
    ConfigLock()
    ; 不对配置自动重新排序
    if (comment) {
        ; IniDelete, %CapsLockX_配置路径%, %field%, %varName%#注释
        IniWrite, %comment%, %CapsLockX_配置路径%, %field%, %varName%#注释
    }
    ; IniDelete, %CapsLockX_配置路径%, %field%, %varName%
    IniWrite, %content%, %CapsLockX_配置路径%, %field%, %varName%
    ConfigUnlock()
    return content
}
CapsLockX_ConfigGet(field, varName, defaultValue)
{
    global CapsLockX_ConfigChangedTickCount
    CapsLockX_ConfigChangedTickCount := A_TickCount
    IniRead, content, %CapsLockX_配置路径%, %field%, %varName%, %defaultValue%
    return content
}
CapsLockX_Config(field, varName, defaultValue, comment := "")
{
    global CapsLockX_ConfigChangedTickCount
    CapsLockX_ConfigChangedTickCount := A_TickCount
    IniRead, content, %CapsLockX_配置路径%, %field%, %varName%, %defaultValue%
    
    ConfigLock(field varName)
    ; 对配置自动重新排序
    if (comment) {
        IniDelete, %CapsLockX_配置路径%, %field%, %varName%#注释
        IniWrite, %comment%, %CapsLockX_配置路径%, %field%, %varName%#注释
    }
    IniDelete, %CapsLockX_配置路径%, %field%, %varName%
    IniWrite, %content%, %CapsLockX_配置路径%, %field%, %varName%
    ConfigUnlock()
    return content
}

清洗为_UTF16_WITH_BOM_型编码(path){
    ConfigLock("UTF16_WITH_BOM 文件编码清洗：" path)
    
    FileEncoding, UTF-8
    FileRead content, %path%
    FileDelete %path%
    FileAppend %content%, %path%, UTF-16
    FileRead content, %path%
    
    ConfigUnlock()
}

清洗为_UTF8_WITH_BOM_型编码(path){
    ConfigLock("UTF8_WITH_BOM 文件编码清洗：" path)
    
    FileEncoding UTF-8
    FileRead content, %path%
    FileDelete %path%
    FileAppend %content%, %path%, UTF-8
    
    ConfigUnlock()
}

ConfigLock(名义:="")
{
    k:= 0
    while (FileExist(CapsLockX_配置路径 ".lock")) {
        k := k + 1
        if ( k > 10 ) {
            FileRead 上次名义, % CapsLockX_配置路径 ".lock"
            ; MsgBox, 程序退出, %名义% 配置写入失败，配置文件被意外锁定，上次锁定名义为 %上次名义%
            ConfigUnlock()
            Reload
            ExitApp
            ; return False
        }
        Sleep, 1000
    }
    CapsLockX_DontReload := 1
    
    FileAppend %名义%, % CapsLockX_配置路径 ".lock"
    return True
}
ConfigUnlock()
{
    FileDelete % CapsLockX_配置路径 ".lock"
    CapsLockX_DontReload := 0
    return True
}