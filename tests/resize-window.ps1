param([int]$AppProcess, [ValidateSet('compact','maximize','restore')][string]$Mode)
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class PopitWindowTest {
 public delegate bool WindowCallback(IntPtr h, IntPtr arg);
 [DllImport("user32.dll")] public static extern bool EnumWindows(WindowCallback callback,IntPtr arg);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,System.Text.StringBuilder title,int count);
 public static IntPtr FindMain(uint wanted) {
  IntPtr found=IntPtr.Zero;
  EnumWindows(delegate(IntPtr h,IntPtr arg){uint pid; GetWindowThreadProcessId(h,out pid);var title=new System.Text.StringBuilder(500); GetWindowText(h,title,500);if(pid==wanted && (title.ToString().StartsWith("LittleBigPlanet") || title.ToString().StartsWith("Patchwork Pod"))){found=h;return false;}return true;},IntPtr.Zero);
  return found;
 }
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls,string title);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int command);
 [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h,int x,int y,int w,int height,bool repaint);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr h);
}
'@
[PopitWindowTest]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$testProcess=Get-Process -Id $AppProcess
if($testProcess.MainWindowTitle -like '*-siw'){[PopitWindowTest]::ShowWindow($testProcess.MainWindowHandle,0) | Out-Null}
$window=[PopitWindowTest]::FindMain($AppProcess)
[uint32]$owner=0
[PopitWindowTest]::GetWindowThreadProcessId($window,[ref]$owner) | Out-Null
if($owner -ne $AppProcess){throw 'Window does not belong to test process'}
if($window -eq 0){throw 'Test application window unavailable'}
if($Mode -eq 'maximize'){
 [PopitWindowTest]::ShowWindow($window,3) | Out-Null
}else{
 [PopitWindowTest]::ShowWindow($window,9) | Out-Null
 $scale=[PopitWindowTest]::GetDpiForWindow($window)/96
 $width=if($Mode -eq 'compact'){1020}else{1300}
 [PopitWindowTest]::MoveWindow($window,40,40,[int]($width*$scale),[int](800*$scale),$true) | Out-Null
}
[PopitWindowTest]::IsZoomed($window)
