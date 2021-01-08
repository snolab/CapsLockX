/*****************************************************************************
  IME����p �֐��Q (IME.ahk)

    �O���[�o���ϐ� : �Ȃ�
    �e�֐��̈ˑ��� : �Ȃ�(�K�v�֐������؏o���ăR�s�y�ł��g���܂�)

    AutoHotkey:     L 1.1.08.01
    Language:       Japanease
    Platform:       NT�n
    Author:         eamat.      http://www6.atwiki.jp/eamat/
*****************************************************************************
����
    2008.07.11 v1.0.47�ȍ~�� �֐����C�u�����X�N���v�g�Ή��p�Ƀt�@�C������ύX
    2008.12.10 �R�����g�C��
    2009.07.03 IME_GetConverting() �ǉ� 
               Last Found Window���L���ɂȂ�Ȃ����C���A���B
    2009.12.03
      �EIME ��ԃ`�F�b�N GUIThreadInfo ���p�� ���ꍞ��
       �iIE��G��8���ł�IME��Ԃ�����悤�Ɂj
        http://blechmusik.xrea.jp/resources/keyboard_layout/DvorakJ/inc/IME.ahk
      �EGoogle���{����̓� ��������
        ���̓��[�h �y�� �ϊ����[�h�͎��Ȃ����ۂ�
        IME_GET/SET() �� IME_GetConverting()�͗L��

    2012.11.10 x64 & Unicode�Ή�
      ���s���� AHK_L U64�� (�{�Ƃ����A32,U32�łƂ̌݊����͈ێ���������)
      �ELongPtr�΍�F�|�C���^�T�C�Y��A_PtrSize�Ō���悤�ɂ���

                ;==================================
                ;  GUIThreadInfo 
                ;=================================
                ; �\���� GUITreadInfo
                ;typedef struct tagGUITHREADINFO {(x86) (x64)
                ;	DWORD   cbSize;                 0    0
                ;	DWORD   flags;                  4    4   ��
                ;	HWND	hwndActive;             8    8
                ;	HWND	hwndFocus;             12    16  ��
                ;	HWND	hwndCapture;           16    24
                ;	HWND	hwndMenuOwner;         20    32
                ;	HWND	hwndMoveSize;          24    40
                ;	HWND	hwndCaret;             28    48
                ;	RECT	rcCaret;               32    56
                ;} GUITHREADINFO, *PGUITHREADINFO;

      �EWinTitle�p�����[�^���������Ӗ������Ă����̂��C��
        �Ώۂ��A�N�e�B�u�E�B���h�E�̎��̂� GetGUIThreadInfo���g��
        �����łȂ��Ƃ���Control�n���h�����g�p
        �ꉞ�o�b�N�O���E���h��IME��������悤�ɖ߂���
        (�擾�n���h����Window����Control�ɕς������ƂŃu���E�U�ȊO�̑唼��
        �A�v���ł̓o�b�N�O���E���h�ł��������l������悤�ɂȂ����B
        ���u���E�U�n�ł��A�N�e�B�u���݂̂ł̎g�p�Ȃ���Ȃ��Ǝv���A���Ԃ�)

*/

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;; 
;
; ����m�F�p �������[�`�� (�}�E�X�J�[�\���ʒu�̃E�B���h�E��IME��Ԃ�����)
;  �P�̋N�����̃e�X�g�p�Ȃ̂ō폜���Ă����Ȃ�
_ImeAutoExecuteSample:
    Hotkey,#1,_ImeGetTest
    Hotkey,#2,_ImeSetTest
    Hotkey,#3,_ImeIsConvertingTest
    Hotkey,+ESC,_ImeTestExt
    SetTimer,_ImeInfoTimer,ON
Return

;--- IME��ԕ\���^�C�} ---
_ImeInfoTimer:
    Tooltip,% "IME_GET			: "     . IME_GET(_mhwnd())             . "`n"
          .  "IME_GetConvMode		: " . IME_GetConvMode(_mhwnd())     . "`n"
          .  "IME_GetSentenceMode	: " . IME_GetSentenceMode(_mhwnd()) . "`n"
          .  "IME_GetConverting	: "     . IME_GetConverting(_mhwnd())
Return

;--- IME Get Test [Win]+[1] ---
_ImeGetTest:
    MsgBox,% "IME_GET			: "     . IME_GET(_mhwnd())             . "`n"
          .  "IME_GetConvMode		: " . IME_GetConvMode(_mhwnd())     . "`n"
          .  "IME_GetSentenceMode	: " . IME_GetSentenceMode(_mhwnd()) . "`n"
Return
;--- IME Get Test [Win]+[2] ---
_ImeSetTest:
    MsgBox,% "IME_SET			: "     . IME_SET(1,_mhwnd())             . "`n"
          .  "IME_SetConvMode		: " . IME_SetConvMode(0x08,_mhwnd())  . "`n"
          .  "IME_SetSentenceMode	: " . IME_SetSentenceMode(1,_mhwnd()) . "`n"
Return

_mhwnd(){	;background test
	MouseGetPos,x,,hwnd
	Return "ahk_id " . hwnd
}

;------------------------------------------------------------------
; IME���̃N���X���𒲂ׂ�e�X�g���[�`��
;   ����or�ϊ���Ԃ�Ime���Ƀ}�E�X�J�[�\�������Ă��� [Win]+[3]����
;   Clipboard�� Class�����R�s�[�����B���͑�/��⑋ ���ꂼ�꒲�ׂ�B
;   ���ׂ��N���X���� ���K�\���ɂȂ�����
;      IME_GetConverting("A","���͑��N���X","��⑋�N���X")
;   �Ƃ�����Ďg���B(�������� IME_GetConverting()�̒��ɒ��ڒǉ�����)
;
;   ������    �� ���͑��̏�� �}�E�X�J�[�\�������Ă��� [Win]+[3]����
;   �P�P�P       Clipboard�� Class�����R�s�[�����B
;                �� MS Office�n�̃V�[�����X���͏�Ԃł͎��Ȃ����ۂ�
;                   DetectHiddenWindows,ON�ł��_���B�V�[�����XOFF�ɂ��Ȃ��Ɩ���
;
;   ��
;  |���@�@�@| �� ��⑋�̏�Ƀ}�E�X�J�[�\�������Ă��� [Win]+[3]����
;  |���ˁ@�@|    Clipboard�� Class�����R�s�[�����B
;  |�����@�@|
;  |���@�@�@|
;  |�@�F�@�@|
;  �P�P�P�P
;------------------------------------------------------------------
_ImeIsConvertingTest:
    _ImeTestClassCheck()
Return
_ImeTestClassCheck()  {
    MouseGetPos,,,hwnd
    WinGetClass,Imeclass,ahk_id %hwnd%
    Clipboard := Imeclass
    ;IME_GetConverting() ����`�F�b�N & IME ���͑�/��⑋ Class���m�F
    MsgBox,% Imeclass "`n" IME_GetConverting()
}
;--- �풓�e�X�g�I�� [Shift]+[ESC] ---
_ImeTestExt:
ExitApp
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;; 

;---------------------------------------------------------------------------
;  �ėp�֐� (�����ǂ�IME�ł�������͂�)

;-----------------------------------------------------------
; IME�̏�Ԃ̎擾
;   WinTitle="A"    �Ώ�Window
;   �߂�l          1:ON / 0:OFF
;-----------------------------------------------------------
IME_GET(WinTitle="A")  {
	ControlGet,hwnd,HWND,,,%WinTitle%
	if	(WinActive(WinTitle))	{
		ptrSize := !A_PtrSize ? 4 : A_PtrSize
	    VarSetCapacity(stGTI, cbSize:=4+4+(PtrSize*6)+16, 0)
	    NumPut(cbSize, stGTI,  0, "UInt")   ;	DWORD   cbSize;
		hwnd := DllCall("GetGUIThreadInfo", Uint,0, Uint,&stGTI)
	             ? NumGet(stGTI,8+PtrSize,"UInt") : hwnd
	}

    Return DllCall("SendMessage"
          , UInt, DllCall("imm32\ImmGetDefaultIMEWnd", Uint,hwnd)
          , UInt, 0x0283  ;Message : WM_IME_CONTROL
          ,  Int, 0x0005  ;wParam  : IMC_GETOPENSTATUS
          ,  Int, 0)      ;lParam  : 0
}

;-----------------------------------------------------------
; IME�̏�Ԃ��Z�b�g
;   SetSts          1:ON / 0:OFF
;   WinTitle="A"    �Ώ�Window
;   �߂�l          0:���� / 0�ȊO:���s
;-----------------------------------------------------------
IME_SET(SetSts, WinTitle="A")    {
	ControlGet,hwnd,HWND,,,%WinTitle%
	if	(WinActive(WinTitle))	{
		ptrSize := !A_PtrSize ? 4 : A_PtrSize
	    VarSetCapacity(stGTI, cbSize:=4+4+(PtrSize*6)+16, 0)
	    NumPut(cbSize, stGTI,  0, "UInt")   ;	DWORD   cbSize;
		hwnd := DllCall("GetGUIThreadInfo", Uint,0, Uint,&stGTI)
	             ? NumGet(stGTI,8+PtrSize,"UInt") : hwnd
	}

    Return DllCall("SendMessage"
          , UInt, DllCall("imm32\ImmGetDefaultIMEWnd", Uint,hwnd)
          , UInt, 0x0283  ;Message : WM_IME_CONTROL
          ,  Int, 0x006   ;wParam  : IMC_SETOPENSTATUS
          ,  Int, SetSts) ;lParam  : 0 or 1
}

;===========================================================================
; IME ���̓��[�h (�ǂ� IME�ł����ʂ��ۂ�)
;   DEC  HEX    BIN
;     0 (0x00  0000 0000) ����    ���p��
;     3 (0x03  0000 0011)         ����
;     8 (0x08  0000 1000)         �S�p��
;     9 (0x09  0000 1001)         �Ђ炪��
;    11 (0x0B  0000 1011)         �S�J�^�J�i
;    16 (0x10  0001 0000) ���[�}�����p��
;    19 (0x13  0001 0011)         ����
;    24 (0x18  0001 1000)         �S�p��
;    25 (0x19  0001 1001)         �Ђ炪��
;    27 (0x1B  0001 1011)         �S�J�^�J�i

;  �� �n��ƌ���̃I�v�V���� - [�ڍ�] - �ڍאݒ�
;     - �ڍׂȃe�L�X�g�T�[�r�X�̃T�|�[�g���v���O�����̂��ׂĂɊg������
;    �� ON�ɂȂ��Ă�ƒl�����Ȃ��͗l 
;    (Google���{����̓��͂�����ON�ɂ��Ȃ��ƑʖڂȂ̂Œl�����Ȃ����ۂ�)

;-------------------------------------------------------
; IME ���̓��[�h�擾
;   WinTitle="A"    �Ώ�Window
;   �߂�l          ���̓��[�h
;--------------------------------------------------------
IME_GetConvMode(WinTitle="A")   {
	ControlGet,hwnd,HWND,,,%WinTitle%
	if	(WinActive(WinTitle))	{
		ptrSize := !A_PtrSize ? 4 : A_PtrSize
	    VarSetCapacity(stGTI, cbSize:=4+4+(PtrSize*6)+16, 0)
	    NumPut(cbSize, stGTI,  0, "UInt")   ;	DWORD   cbSize;
		hwnd := DllCall("GetGUIThreadInfo", Uint,0, Uint,&stGTI)
	             ? NumGet(stGTI,8+PtrSize,"UInt") : hwnd
	}
    Return DllCall("SendMessage"
          , UInt, DllCall("imm32\ImmGetDefaultIMEWnd", Uint,hwnd)
          , UInt, 0x0283  ;Message : WM_IME_CONTROL
          ,  Int, 0x001   ;wParam  : IMC_GETCONVERSIONMODE
          ,  Int, 0)      ;lParam  : 0
}

;-------------------------------------------------------
; IME ���̓��[�h�Z�b�g
;   ConvMode        ���̓��[�h
;   WinTitle="A"    �Ώ�Window
;   �߂�l          0:���� / 0�ȊO:���s
;--------------------------------------------------------
IME_SetConvMode(ConvMode,WinTitle="A")   {
	ControlGet,hwnd,HWND,,,%WinTitle%
	if	(WinActive(WinTitle))	{
		ptrSize := !A_PtrSize ? 4 : A_PtrSize
	    VarSetCapacity(stGTI, cbSize:=4+4+(PtrSize*6)+16, 0)
	    NumPut(cbSize, stGTI,  0, "UInt")   ;	DWORD   cbSize;
		hwnd := DllCall("GetGUIThreadInfo", Uint,0, Uint,&stGTI)
	             ? NumGet(stGTI,8+PtrSize,"UInt") : hwnd
	}
    Return DllCall("SendMessage"
          , UInt, DllCall("imm32\ImmGetDefaultIMEWnd", Uint,hwnd)
          , UInt, 0x0283      ;Message : WM_IME_CONTROL
          ,  Int, 0x002       ;wParam  : IMC_SETCONVERSIONMODE
          ,  Int, ConvMode)   ;lParam  : CONVERSIONMODE
}

;===========================================================================
; IME �ϊ����[�h (ATOK��ver.16�Œ����A�o�[�W�����ő����Ⴄ����)

;   MS-IME  0:���ϊ� / 1:�l��/�n��                    / 8:���    /16:�b�����t
;   ATOK�n  0:�Œ�   / 1:������              / 4:���� / 8:�A����
;   WXG              / 1:������  / 2:���ϊ�  / 4:���� / 8:�A����
;   SKK�n            / 1:�m�[�}�� (���̃��[�h�͑��݂��Ȃ��H)
;   Google��                                          / 8:�m�[�}��
;------------------------------------------------------------------
; IME �ϊ����[�h�擾
;   WinTitle="A"    �Ώ�Window
;   �߂�l MS-IME  0:���ϊ� 1:�l��/�n��               8:���    16:�b�����t
;          ATOK�n  0:�Œ�   1:������           4:���� 8:�A����
;          WXG4             1:������  2:���ϊ� 4:���� 8:�A����
;------------------------------------------------------------------
IME_GetSentenceMode(WinTitle="A")   {
	ControlGet,hwnd,HWND,,,%WinTitle%
	if	(WinActive(WinTitle))	{
		ptrSize := !A_PtrSize ? 4 : A_PtrSize
	    VarSetCapacity(stGTI, cbSize:=4+4+(PtrSize*6)+16, 0)
	    NumPut(cbSize, stGTI,  0, "UInt")   ;	DWORD   cbSize;
		hwnd := DllCall("GetGUIThreadInfo", Uint,0, Uint,&stGTI)
	             ? NumGet(stGTI,8+PtrSize,"UInt") : hwnd
	}
    Return DllCall("SendMessage"
          , UInt, DllCall("imm32\ImmGetDefaultIMEWnd", Uint,hwnd)
          , UInt, 0x0283  ;Message : WM_IME_CONTROL
          ,  Int, 0x003   ;wParam  : IMC_GETSENTENCEMODE
          ,  Int, 0)      ;lParam  : 0
}

;----------------------------------------------------------------
; IME �ϊ����[�h�Z�b�g
;   SentenceMode
;       MS-IME  0:���ϊ� 1:�l��/�n��               8:���    16:�b�����t
;       ATOK�n  0:�Œ�   1:������           4:���� 8:�A����
;       WXG              1:������  2:���ϊ� 4:���� 8:�A����
;   WinTitle="A"    �Ώ�Window
;   �߂�l          0:���� / 0�ȊO:���s
;-----------------------------------------------------------------
IME_SetSentenceMode(SentenceMode,WinTitle="A")  {
	ControlGet,hwnd,HWND,,,%WinTitle%
	if	(WinActive(WinTitle))	{
		ptrSize := !A_PtrSize ? 4 : A_PtrSize
	    VarSetCapacity(stGTI, cbSize:=4+4+(PtrSize*6)+16, 0)
	    NumPut(cbSize, stGTI,  0, "UInt")   ;	DWORD   cbSize;
		hwnd := DllCall("GetGUIThreadInfo", Uint,0, Uint,&stGTI)
	             ? NumGet(stGTI,8+PtrSize,"UInt") : hwnd
	}
    Return DllCall("SendMessage"
          , UInt, DllCall("imm32\ImmGetDefaultIMEWnd", Uint,hwnd)
          , UInt, 0x0283          ;Message : WM_IME_CONTROL
          ,  Int, 0x004           ;wParam  : IMC_SETSENTENCEMODE
          ,  Int, SentenceMode)   ;lParam  : SentenceMode
}


;---------------------------------------------------------------------------
;  IME�̎�ނ�I�Ԃ�������Ȃ��֐�

;==========================================================================
;  IME �������͂̏�Ԃ�Ԃ�
;  (�p�N���� : http://sites.google.com/site/agkh6mze/scripts#TOC-IME- )
;    �W���Ή�IME : ATOK�n / MS-IME2002 2007 / WXG / SKKIME
;    ���̑���IME�� ���͑�/�ϊ�����ǉ��w�肷�邱�ƂőΉ��\
;
;       WinTitle="A"   �Ώ�Window
;       ConvCls=""     ���͑��̃N���X�� (���K�\���\�L)
;       CandCls=""     ��⑋�̃N���X�� (���K�\���\�L)
;       �߂�l      1 : �������͒� or �ϊ���
;                   2 : �ϊ���⑋���o�Ă���
;                   0 : ���̑��̏��
;
;   �� MS-Office�n�� ���͑��̃N���X�� �𐳂����擾����ɂ�IME�̃V�[�����X�\����
;      OFF�ɂ���K�v������
;      �I�v�V����-�ҏW�Ɠ��{�����-�ҏW���̕�����𕶏��ɑ}�����[�h�œ��͂���
;      �̃`�F�b�N���O��
;==========================================================================
IME_GetConverting(WinTitle="A",ConvCls="",CandCls="") {

    ;IME���� ���͑�/��⑋Class�ꗗ ("|" ��؂�œK���ɑ����Ă���OK)
    ConvCls .= (ConvCls ? "|" : "")                 ;--- ���͑� ---
            .  "ATOK\d+CompStr"                     ; ATOK�n
            .  "|imejpstcnv\d+"                     ; MS-IME�n
            .  "|WXGIMEConv"                        ; WXG
            .  "|SKKIME\d+\.*\d+UCompStr"           ; SKKIME Unicode
            .  "|MSCTFIME Composition"              ; Google���{�����

    CandCls .= (CandCls ? "|" : "")                 ;--- ��⑋ ---
            .  "ATOK\d+Cand"                        ; ATOK�n
            .  "|imejpstCandList\d+|imejpstcand\d+" ; MS-IME 2002(8.1)XP�t��
            .  "|mscandui\d+\.candidate"            ; MS Office IME-2007
            .  "|WXGIMECand"                        ; WXG
            .  "|SKKIME\d+\.*\d+UCand"              ; SKKIME Unicode
   CandGCls := "GoogleJapaneseInputCandidateWindow" ;Google���{�����

	ControlGet,hwnd,HWND,,,%WinTitle%
	if	(WinActive(WinTitle))	{
		ptrSize := !A_PtrSize ? 4 : A_PtrSize
	    VarSetCapacity(stGTI, cbSize:=4+4+(PtrSize*6)+16, 0)
	    NumPut(cbSize, stGTI,  0, "UInt")   ;	DWORD   cbSize;
		hwnd := DllCall("GetGUIThreadInfo", Uint,0, Uint,&stGTI)
	             ? NumGet(stGTI,8+PtrSize,"UInt") : hwnd
	}

    WinGet, pid, PID,% "ahk_id " hwnd
    tmm:=A_TitleMatchMode
    SetTitleMatchMode, RegEx
    ret := WinExist("ahk_class " . CandCls . " ahk_pid " pid) ? 2
        :  WinExist("ahk_class " . CandGCls                 ) ? 2
        :  WinExist("ahk_class " . ConvCls . " ahk_pid " pid) ? 1
        :  0
    SetTitleMatchMode, %tmm%
    Return ret
}
