; #Requires AutoHotkey v1.1.33+
; - [AutoHotkey - How to inject multi-touch (single touch works) - Stack Overflow]( https://stackoverflow.com/questions/67814959/autohotkey-how-to-inject-multi-touch-single-touch-works )
; - [How to trigger touch events? : r/AutoHotkey]( https://www.reddit.com/r/AutoHotkey/comments/al053y/how_to_trigger_touch_events/ )
; https://web.archive.org/web/20210531202618/https://social.technet.microsoft.com/wiki/contents/articles/6460.simulating-touch-input-in-windows-8-using-touch-injection-api.aspx sda
Return

; ^WheelUp:: pz()

pz(){
    
    ; https://web.archive.org/web/20210531202629/https://www.autohotkey.com/boards/viewtopic.php?f=13&t=2474
    ; https://web.archive.org/web/20210531202618/https://social.technet.microsoft.com/wiki/contents/articles/6460.simulating-touch-input-in-windows-8-using-touch-injection-api.aspx

    ; Simulating Tap
    ; By Tap we mean that it has simply touched any portion of screen. In this use case actor takes two action
    ; 	* Touch down on screen co-ordinate input
    ; 	* Lifts Up Touch input

    ; x86 | x64
    ; 136 | 144 sizeof POINTER_TOUCH_INFO
    ;   0 |   0   UInt pointerInfo.pointerType
    ;   4 |   4   UInt pointerInfo.pointerId
    ;   8 |   8   UInt pointerInfo.frameId
    ;  12 |  12   UInt pointerInfo.pointerFlags
    ;  16 |  16    Ptr pointerInfo.sourceDevice
    ;  20 |  24    Ptr pointerInfo.hwndTarget
    ;  24 |  32    Int pointerInfo.ptPixelLocation.x
    ;  28 |  36    Int pointerInfo.ptPixelLocation.y
    ;  32 |  40    Int pointerInfo.ptHimetricLocation.x
    ;  36 |  44    Int pointerInfo.ptHimetricLocation.y
    ;  40 |  48    Int pointerInfo.ptPixelLocationRaw.x
    ;  44 |  52    Int pointerInfo.ptPixelLocationRaw.y
    ;  48 |  56    Int pointerInfo.ptHimetricLocationRaw.x
    ;  52 |  60    Int pointerInfo.ptHimetricLocationRaw.y
    ;  56 |  64   UInt pointerInfo.dwTime
    ;  60 |  68   UInt pointerInfo.historyCount
    ;  64 |  72    Int pointerInfo.InputData
    ;  68 |  76   UInt pointerInfo.dwKeyStates
    ;  72 |  80 UInt64 pointerInfo.PerformanceCount
    ;  80 |  88    Int pointerInfo.ButtonChangeType
    ;  88 |  96   UInt touchFlags
    ;  92 | 100   UInt touchMask
    ;  96 | 104    Int rcContact.left
    ; 100 | 108    Int rcContact.top
    ; 104 | 112    Int rcContact.right
    ; 108 | 116    Int rcContact.bottom
    ; 112 | 120    Int rcContactRaw.left
    ; 116 | 124    Int rcContactRaw.top
    ; 120 | 128    Int rcContactRaw.right
    ; 124 | 132    Int rcContactRaw.bottom
    ; 128 | 136   UInt orientation
    ; 132 | 140   UInt pressure

    ; enum tagPOINTER_INPUT_TYPE {
    ;     PT_POINTER  = 1,   // Generic pointer
    ;     PT_TOUCH    = 2,   // Touch
    ;     PT_PEN      = 3,   // Pen
    ;     PT_MOUSE    = 4,   // Mouse
    ; #if(WINVER >= 0x0603)
    ;     PT_TOUCHPAD = 5,   // Touchpad
    ; #endif /* WINVER >= 0x0603 */
    ; };

    ; #define TOUCH_FLAG_NONE                 0x00000000 // Default    
    
    ; #define TOUCH_MASK_NONE                 0x00000000 // Default - none of the optional fields are valid
    ; #define TOUCH_MASK_CONTACTAREA          0x00000001 // The rcContact field is valid
    ; #define TOUCH_MASK_ORIENTATION          0x00000002 // The orientation field is valid
    ; #define TOUCH_MASK_PRESSURE             0x00000004 // The pressure field is valid

    ; #define POINTER_FLAG_NONE               0x00000000 // Default
    ; #define POINTER_FLAG_NEW                0x00000001 // New pointer
    ; #define POINTER_FLAG_INRANGE            0x00000002 // Pointer has not departed
    ; #define POINTER_FLAG_INCONTACT          0x00000004 // Pointer is in contact
    ; #define POINTER_FLAG_FIRSTBUTTON        0x00000010 // Primary action
    ; #define POINTER_FLAG_SECONDBUTTON       0x00000020 // Secondary action
    ; #define POINTER_FLAG_THIRDBUTTON        0x00000040 // Third button
    ; #define POINTER_FLAG_FOURTHBUTTON       0x00000080 // Fourth button
    ; #define POINTER_FLAG_FIFTHBUTTON        0x00000100 // Fifth button
    ; #define POINTER_FLAG_PRIMARY            0x00002000 // Pointer is primary
    ; #define POINTER_FLAG_CONFIDENCE         0x00004000 // Pointer is considered unlikely to be accidental
    ; #define POINTER_FLAG_CANCELED           0x00008000 // Pointer is departing in an abnormal manner
    ; #define POINTER_FLAG_DOWN               0x00010000 // Pointer transitioned to down state (made contact)
    ; #define POINTER_FLAG_UPDATE             0x00020000 // Pointer update
    ; #define POINTER_FLAG_UP                 0x00040000 // Pointer transitioned from down state (broke contact)
    ; #define POINTER_FLAG_WHEEL              0x00080000 // Vertical wheel
    ; #define POINTER_FLAG_HWHEEL             0x00100000 // Horizontal wheel
    ; #define POINTER_FLAG_CAPTURECHANGED     0x00200000 // Lost capture
    ; #define POINTER_FLAG_HASTRANSFORM       0x00400000 // Input has a transform associated with it



    ; WinAPI Constants
    ; =========================================================================================================================================        
    PT_TOUCH := 2
    PT_TOUCHPAD := 5

    TOUCH_FEEDBACK_DEFAULT := 0x1
    TOUCH_FEEDBACK_INDIRECT := 0x2
    TOUCH_FEEDBACK_NONE := 0x3
    
    TOUCH_FLAG_NONE := 0x00000000

    TOUCH_MASK_CONTACTAREA := 0x00000001
    TOUCH_MASK_ORIENTATION := 0x00000002
    TOUCH_MASK_PRESSURE := 0x00000004

    POINTER_FLAG_DOWN := 0x00010000
    POINTER_FLAG_INRANGE := 0x00000002
    POINTER_FLAG_INCONTACT := 0x00000004
    POINTER_FLAG_UPDATE := 0x00020000
    POINTER_FLAG_UP := 0x00040000
    
    
    ; =========================================================================================================================================

    CONTACTS := 2
    PT_MODE := PT_TOUCH

    ; Array to contain each contact (144 bytes each)
    VarSetCapacity(contact, 144*2, 0)
    
    ; Center pinch/zoom on current cursor position
    MouseGetPos, xpos, ypos1 

    xpos1 := xpos
    ypos1 := ypos
    
    xpos2 := xpos1 - 60
    xpos1 += 60
    ypos2 := ypos1

    if(!DllCall("InitializeTouchInjection", "UInt", CONTACTS, "UInt", TOUCH_FEEDBACK_DEFAULT))
        msgbox fail to InitializeTouchInjection
    
    
    ; Init first finger - 0 byte offset
    ; =========================================================================================================================================   
    NumPut(PT_MODE, contact, 0+0, "UInt")
    NumPut(0, contact, 0+4, "UInt")
    NumPut(x := xpos1, contact, 0+32, "Int")
    NumPut(y := ypos1, contact, 0+36, "Int")

    NumPut(TOUCH_FLAG_NONE, contact, 0+96, "UInt")
    NumPut(TOUCH_MASK_CONTACTAREA | TOUCH_MASK_ORIENTATION | TOUCH_MASK_PRESSURE, contact, 0+100, "UInt")

    NumPut(90, contact, 0+136, "UInt")
    NumPut(32000, contact, 0+140, "UInt")

    NumPut(x - 2, contact, 0+104, "Int")
    NumPut(y - 2, contact, 0+108, "Int")    
    NumPut(x + 2, contact, 0+112, "Int")
    NumPut(y + 2, contact, 0+116, "Int")
    
    NumPut(POINTER_FLAG_DOWN | POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, contact, 0+12, "UInt")


    ; Init second finger - 144 byte offset
    ; =========================================================================================================================================
    NumPut(PT_MODE, contact, 144+0, "UInt")
    NumPut(1, contact, 144+4, "UInt")
    NumPut(x := xpos2, contact, 144+32, "Int")
    NumPut(y := ypos2, contact, 144+36, "Int")

    NumPut(TOUCH_FLAG_NONE, contact, 144+96, "UInt")
    NumPut(TOUCH_MASK_CONTACTAREA | TOUCH_MASK_ORIENTATION | TOUCH_MASK_PRESSURE, contact, 144+100, "UInt")

    NumPut(90, contact, 144+136, "UInt")
    NumPut(32000, contact, 144+140, "UInt")

    NumPut(x - 2, contact, 144+104, "Int")
    NumPut(y - 2, contact, 144+108, "Int")
    NumPut(x + 2, contact, 144+112, "Int")
    NumPut(y + 2, contact, 144+116, "Int")
    
    NumPut(POINTER_FLAG_DOWN | POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, contact, 144+12, "UInt")


    ; Register both fingers touching
    ; =========================================================================================================================================
    DllCall("InjectTouchInput", "UInt", CONTACTS, "Ptr", &contact)
    
    
    
    ; Mimic fingers spreading to zoom in on X axis
    ; =========================================================================================================================================    
    NumPut(POINTER_FLAG_UPDATE | POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, contact, 0+12, "UInt")
    NumPut(POINTER_FLAG_UPDATE | POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, contact, 144+12, "UInt")
    
    
    ; Some delays to better see the gesture, loop to simulate fingers moving
    Sleep, 100
    Loop 50 {
    	; Trying different directions / speeds of movement.  
        x1 := xpos1 + (1*A_Index)
        x2 := xpos2 - (1*A_Index)
        ;y1 := ypos1 - (10*A_Index)
        ;y2 := ypos2 - (10*A_Index)
                
        NumPut(x1, contact, 0+32, "Int")
        NumPut(x2, contact, 144+32, "Int")
        
        ;NumPut(y1, contact, 0+36, "Int")
        ;NumPut(y2, contact, 144+36, "Int")
    
        DllCall("InjectTouchInput", "UInt", CONTACTS, "Ptr", &contact)
        Sleep, 1
    }
    
    
    ; Fingers Up
    ; =========================================================================================================================================
    NumPut(POINTER_FLAG_UP, contact, 0+12, "UInt")
    NumPut(POINTER_FLAG_UP, contact, 144+12, "UInt")
       
    DllCall("InjectTouchInput", "UInt", CONTACTS, "Ptr", &contact)    

    
    ; Return cursor to original position
    MouseMove, xpos, ypos    
}