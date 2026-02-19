$code = @"
using System;
using System.Runtime.InteropServices;
public class KSQuery {
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern IntPtr CreateFileW(
        string lpFileName, uint dwDesiredAccess, uint dwShareMode,
        IntPtr lpSecurityAttributes, uint dwCreationDisposition,
        uint dwFlagsAndAttributes, IntPtr hTemplateFile);
    [DllImport("kernel32.dll")]
    public static extern bool CloseHandle(IntPtr hObject);
    [DllImport("kernel32.dll", SetLastError=true)]
    public static extern bool DeviceIoControl(
        IntPtr hDevice, uint IoControlCode,
        IntPtr InBuffer, uint InBufferSize,
        IntPtr OutBuffer, uint OutBufferSize,
        out uint BytesReturned, IntPtr Overlapped);
}
"@
Add-Type -TypeDefinition $code

$sb = [System.Text.StringBuilder]::new()

# IOCTL_KS_PROPERTY = 0x2F000003 (METHOD_NEITHER based) 
# Actually for KS: CTL_CODE(FILE_DEVICE_KS, 0, METHOD_NEITHER, FILE_ANY_ACCESS)
# FILE_DEVICE_KS = 0x2F
# IOCTL_KS_PROPERTY = (0x2F << 16) | (0 << 14) | (0 << 2) | 3 = 0x2F0003
$IOCTL_KS_PROPERTY = 0x002F0003

$paths = @(
    @('\\?\ROOT#MEDIA#0000#{6994ad04-93ef-11d0-a3cc-00a0c9223196}\WaveRender', 'WaveRender'),
    @('\\?\ROOT#MEDIA#0000#{6994ad04-93ef-11d0-a3cc-00a0c9223196}\TopologyRender', 'TopoRender')
)

foreach ($entry in $paths) {
    $path = $entry[0]
    $name = $entry[1]
    
    $h = [KSQuery]::CreateFileW($path, 0xC0000000, 7, [IntPtr]::Zero, 3, 0x40000000, [IntPtr]::Zero)
    $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
    
    if ($h -ne [IntPtr]::new(-1)) {
        [void]$sb.AppendLine("$name : OPENED")
        
        # Try query KSPROPSETID_Pin / KSPROPERTY_PIN_CTYPES
        # KSPROPSETID_Pin = {8C134960-51AD-11CF-878A-94F801C10000}
        # KSPROPERTY_PIN_CTYPES = 0
        # KSP_PIN structure: KSPROPERTY (24 bytes) + PinId (4 bytes) + Reserved (4 bytes) = 32 bytes
        # KSPROPERTY: Set (16 bytes GUID) + Id (4 bytes) + Flags (4 bytes) = 24 bytes
        $propSize = 32  # KSP_PIN
        $outSize = 4    # ULONG result
        
        $propBuf = [System.Runtime.InteropServices.Marshal]::AllocHGlobal($propSize)
        $outBuf = [System.Runtime.InteropServices.Marshal]::AllocHGlobal($outSize)
        
        # Zero out
        for ($i = 0; $i -lt $propSize; $i++) { [System.Runtime.InteropServices.Marshal]::WriteByte($propBuf, $i, 0) }
        
        # KSPROPSETID_Pin GUID bytes: 8C134960-51AD-11CF-878A-94F801C10000
        $guidBytes = [byte[]]@(0x60, 0x49, 0x13, 0x8C, 0xAD, 0x51, 0xCF, 0x11, 0x87, 0x8A, 0x94, 0xF8, 0x01, 0xC1, 0x00, 0x00)
        [System.Runtime.InteropServices.Marshal]::Copy($guidBytes, 0, $propBuf, 16)
        
        # Id = 0 (KSPROPERTY_PIN_CTYPES)
        [System.Runtime.InteropServices.Marshal]::WriteInt32($propBuf, 16, 0)
        
        # Flags = KSPROPERTY_TYPE_GET = 1
        [System.Runtime.InteropServices.Marshal]::WriteInt32($propBuf, 20, 1)
        
        $bytesReturned = [uint32]0
        $ok = [KSQuery]::DeviceIoControl($h, $IOCTL_KS_PROPERTY, $propBuf, $propSize, $outBuf, $outSize, [ref]$bytesReturned, [IntPtr]::Zero)
        $err2 = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
        
        if ($ok) {
            $pinCount = [System.Runtime.InteropServices.Marshal]::ReadInt32($outBuf)
            [void]$sb.AppendLine("  PIN_CTYPES = $pinCount")
        }
        else {
            [void]$sb.AppendLine("  PIN_CTYPES query FAILED (err=$err2)")
        }
        
        [System.Runtime.InteropServices.Marshal]::FreeHGlobal($propBuf)
        [System.Runtime.InteropServices.Marshal]::FreeHGlobal($outBuf)
        [KSQuery]::CloseHandle($h) | Out-Null
    }
    else {
        [void]$sb.AppendLine("$name : FAILED to open (err=$err)")
    }
}

$sb.ToString() | Set-Content "C:\LeylineInstall\ksquery.txt"
Write-Host $sb.ToString()
