Return
::[d::
    FormatTime, DateString, , [$yyyy-MM-dd] 
    SendInput %DateString%
    Return
::[t::
    FormatTime, TimeString, , [$yyyy-MM-dd HH:mm] 
    SendInput %TimeString%
    Return
::[s::
    FormatTime, TimeString, , [$HH:mm] 
    SendInput %TimeString%
    Return
