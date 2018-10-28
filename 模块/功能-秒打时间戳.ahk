Return
::[d::
    FormatTime, DateString, , [yyyyMMdd]
    SendInput %DateString%
    Return
::[t::
    FormatTime, TimeString, , [yyyyMMdd.HHmmss]
    SendInput %TimeString%
    Return
::[s::
    FormatTime, TimeString, , [HHmm]
    SendInput %TimeString%
    Return
::[v::
    FormatTime, TimeString, , vyyyy.MM.dd
    SendInput %TimeString%
    Return
