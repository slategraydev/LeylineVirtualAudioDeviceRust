$code = @"
using System;
using System.Runtime.InteropServices;
public class KSTest {
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern IntPtr CreateFileW(
        string lpFileName, uint dwDesiredAccess, uint dwShareMode,
        IntPtr lpSecurityAttributes, uint dwCreationDisposition,
        uint dwFlagsAndAttributes, IntPtr hTemplateFile);
    [DllImport("kernel32.dll")]
    public static extern bool CloseHandle(IntPtr hObject);
}
"@
Add-Type -TypeDefinition $code

$sb = [System.Text.StringBuilder]::new()
$paths = @(
    '\\?\ROOT#MEDIA#0000#{6994ad04-93ef-11d0-a3cc-00a0c9223196}\WaveRender',
    '\\?\ROOT#MEDIA#0000#{6994ad04-93ef-11d0-a3cc-00a0c9223196}\WaveCapture',
    '\\?\ROOT#MEDIA#0000#{6994ad04-93ef-11d0-a3cc-00a0c9223196}\TopologyRender',
    '\\?\ROOT#MEDIA#0000#{6994ad04-93ef-11d0-a3cc-00a0c9223196}\TopologyCapture',
    '\\?\ROOT#MEDIA#0000#{dda54a40-1e4c-11d1-a050-405705c10000}\TopologyRender',
    '\\?\ROOT#MEDIA#0000#{dda54a40-1e4c-11d1-a050-405705c10000}\TopologyCapture',
    '\\?\ROOT#MEDIA#0000#{65e8773e-8f56-11d0-a3b9-00a0c9223196}\WaveRender',
    '\\?\ROOT#MEDIA#0000#{65e8773e-8f56-11d0-a3b9-00a0c9223196}\WaveCapture'
)

foreach ($p in $paths) {
    # GENERIC_READ = 0x80000000, FILE_SHARE_READ|WRITE = 3, OPEN_EXISTING = 3
    # FILE_FLAG_OVERLAPPED = 0x40000000
    $h = [KSTest]::CreateFileW($p, 0x80000000, 3, [IntPtr]::Zero, 3, 0x40000000, [IntPtr]::Zero)
    $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
    if ($h -ne [IntPtr]::new(-1)) {
        [void]$sb.AppendLine("OK   : $p (handle=$h)")
        [KSTest]::CloseHandle($h) | Out-Null
    }
    else {
        $errMsg = switch ($err) {
            2 { "FILE_NOT_FOUND" }
            3 { "PATH_NOT_FOUND" }
            5 { "ACCESS_DENIED" }
            87 { "INVALID_PARAMETER" }
            1167 { "DEVICE_NOT_CONNECTED" }
            default { "code=$err" }
        }
        [void]$sb.AppendLine("FAIL : $p ($errMsg)")
    }
}

$sb.ToString() | Set-Content "C:\LeylineInstall\createfile.txt"
Write-Host $sb.ToString()
