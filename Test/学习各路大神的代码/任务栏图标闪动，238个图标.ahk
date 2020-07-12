;任务栏图标闪动，238个图标      
i=0
loop,238                                        
{
i+=1
Menu, Tray, Icon, Shell32.dll, %i%  ;系统默认图标
sleep,200
}
esc::Exitapp