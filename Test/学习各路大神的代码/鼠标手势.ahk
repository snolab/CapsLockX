LButton::     
  mouseGetPos xpos1,ypos1
  SetTimer,gTrack,1
  Return
LButton up::
  SetTimer,gTrack,off
  msgbox,,,%gTrack%, 0.3
  gTrack=
  Return
gTrack:
    mouseGetPos xpos2,ypos2
    track:=(abs(ypos1-ypos2)>=abs(xpos1-xpos2)) ? (ypos1>ypos2 ? "u" : "d") : (xpos1>xpos2 ? "l" : "r")
    if (track<>SubStr(gTrack, 0, 1)) and (abs(ypos1-ypos2) > 4 or abs(xpos1-xpos2)>4)
        gTrack.=track 
    xpos1:=xpos2,ypos1:=ypos2
    Return

Esc::exitapp