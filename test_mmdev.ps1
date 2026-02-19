# Test MMDevice enumeration (this is what applications use to find audio devices)
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

[Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IMMDeviceEnumerator {
    int EnumAudioEndpoints(int dataFlow, uint dwStateMask, out IMMDeviceCollection ppDevices);
    int GetDefaultAudioEndpoint(int dataFlow, int role, out IntPtr ppEndpoint);
    int GetDevice(string pwstrId, out IntPtr ppDevice);
    int RegisterEndpointNotificationCallback(IntPtr pClient);
    int UnregisterEndpointNotificationCallback(IntPtr pClient);
}

[Guid("0BD7A1BE-7A1A-44DB-8397-CC5392387B5E")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IMMDeviceCollection {
    int GetCount(out uint pcDevices);
    int Item(uint nDevice, out IntPtr ppDevice);
}

public static class AudioEnumerator {
    [DllImport("ole32.dll")]
    public static extern int CoCreateInstance(
        [In] ref Guid clsid, IntPtr pUnkOuter, uint dwClsContext,
        [In] ref Guid riid, out IMMDeviceEnumerator ppv);
}
"@ -IgnoreWarnings

$sb = [System.Text.StringBuilder]::new()

$CLSID_MMDeviceEnumerator = [Guid]"BCDE0395-E52F-467C-8E3D-C4579291692E"
$IID_IMMDeviceEnumerator = [Guid]"A95664D2-9614-4F35-A746-DE8DB63617E6"

$enumerator = $null
$hr = [AudioEnumerator]::CoCreateInstance(
    [ref]$CLSID_MMDeviceEnumerator, [IntPtr]::Zero, 1,
    [ref]$IID_IMMDeviceEnumerator, [ref]$enumerator)

if ($hr -eq 0 -and $enumerator -ne $null) {
    [void]$sb.AppendLine("MMDeviceEnumerator created OK")
    
    # eRender=0, eCapture=1, eAll=2
    # DEVICE_STATE_ACTIVE=1, DEVICE_STATE_DISABLED=2, DEVICE_STATE_NOTPRESENT=4, DEVICE_STATE_UNPLUGGED=8, ALL=15
    foreach ($flow in @(0, 1)) {
        $flowName = if ($flow -eq 0) { "Render" } else { "Capture" }
        $coll = $null
        $hr2 = $enumerator.EnumAudioEndpoints($flow, 15, [ref]$coll)
        if ($hr2 -eq 0 -and $coll -ne $null) {
            $count = 0
            $coll.GetCount([ref]$count)
            [void]$sb.AppendLine("$flowName endpoints (all states): $count")
        }
        else {
            [void]$sb.AppendLine("$flowName EnumAudioEndpoints failed: 0x$($hr2.ToString('X8'))")
        }
    }
}
else {
    [void]$sb.AppendLine("Failed to create MMDeviceEnumerator (hr=0x$($hr.ToString('X8')))")
}

$sb.ToString() | Set-Content "C:\LeylineInstall\mmdev.txt"
Write-Host $sb.ToString()
