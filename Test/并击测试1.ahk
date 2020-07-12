#SingleInstance, Force
SendMode Event
SetWorkingDir, %A_ScriptDir%

Return


; 1234567890
; qwertyuiop
; asdfghjkl;
; zxcvbnm,./

5 & 4:: SendEvent 6
4 & 5:: SendEvent 6
5 & 3:: SendEvent 7
3 & 5:: SendEvent 7
4 & 3:: SendEvent 8
3 & 4:: SendEvent 8
4 & 2:: SendEvent 9
2 & 4:: SendEvent 9
3 & 2:: SendEvent 0
2 & 3:: SendEvent 0
$*1:: SendEvent {blind}1
$*2:: SendEvent {blind}2
$*3:: SendEvent {blind}3
$*4:: SendEvent {blind}4
$*5:: SendEvent {blind}5

; 1 w e r t y u 8 9 0 
; q w e r t y u i o p 
; a s d f g h j k l ; 
; z x c v b n m , . / 

t & r::
r & t:: SendEvent y
t & e::
e & t:: SendEvent u
r & e::
e & r:: SendEvent i
r & w::
w & r:: SendEvent o
e & w::
w & e:: SendEvent p
$*q:: SendEvent {blind}q
$*w:: SendEvent {blind}w
$*e:: SendEvent {blind}e
$*r:: SendEvent {blind}r
$*t:: SendEvent {blind}t


g & f::
f & g:: SendEvent h
g & d::
d & g:: SendEvent j
f & d::
d & f:: SendEvent k
f & s::
s & f:: SendEvent l
d & s::
s & d:: SendEvent `;
$*a:: SendEvent {blind}a
$*s:: SendEvent {blind}s
$*d:: SendEvent {blind}d
$*f:: SendEvent {blind}f
$*g:: SendEvent {blind}g

; 1 2 3 4 5 6 7 8 9 0
; z x c v b y u i o p 
; z x c v b h j k l ; 
; z x c v b n m , . / 

b & v::
v & b:: SendEvent n
b & c::
c & b:: SendEvent m
v & c::
c & v:: SendEvent `,
v & x::
x & v:: SendEvent `.
c & x::
x & c:: SendEvent /
$*z:: SendEvent {blind}z
$*x:: SendEvent {blind}x
$*c:: SendEvent {blind}c
$*v:: SendEvent {blind}v
$*b:: SendEvent {blind}b


F12:: Reload
^F12:: ExitApp