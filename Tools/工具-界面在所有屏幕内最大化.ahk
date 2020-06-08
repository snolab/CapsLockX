SysGet, xborder, 32
SysGet, yborder, 33
SysGet, caption, 4

SysGet SM_XVIRTUALSCREEN,  76
SysGet SM_YVIRTUALSCREEN,  77
SysGet SM_CXVIRTUALSCREEN, 78
SysGet SM_CYVIRTUALSCREEN, 79

left   := SM_XVIRTUALSCREEN  - xborder
top    := SM_YVIRTUALSCREEN  - 1
width  := SM_CXVIRTUALSCREEN + 2 * xborder
height := SM_CYVIRTUALSCREEN + yborder + 1

WinMove, A, , left, top, width, height