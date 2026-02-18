# Verify-AEB-Status.ps1
# Comprehensive verification script for Leyline Audio Endpoint Builder engagement diagnosis
# Run this on the VM to diagnose why AEB is not creating endpoints

param(
    [string]$OutputPath = "C:\LeylineInstall\AEB_Status_Report.txt"
)

$ErrorActionPreference = "Continue"

# Initialize report
$report = @()
$report += "=" * 80
$report += "Leyline Audio Driver - AEB Status Verification Report"
$report += "Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
$report += "=" * 80
$report += ""

# ============================================================================
# Section 1: Device Manager Analysis
# ============================================================================
$report += "# SECTION 1: DEVICE MANAGER ANALYSIS"
$report += "-" * 80

# Find Leyline device
$leylineDevice = Get-PnpDevice | Where-Object { $_.InstanceId -match "LeylineAudio" -or $_.FriendlyName -match "Leyline" }

if ($leylineDevice) {
    $report += "OK DEVICE FOUND"
    $report += "  Instance ID: $($leylineDevice.InstanceId)"
    $report += "  Friendly Name: $($leylineDevice.FriendlyName)"
    $report += "  Class: $($leylineDevice.Class)"
    $report += "  Status: $($leylineDevice.Status)"
    $report += "  Problem Code: $($leylineDevice.ProblemCode)"
    $report += "  Present: $($leylineDevice.Present)"

    # Get additional device details
    $deviceDetails = Get-PnpDeviceProperty $leylineDevice.InstanceId
    $report += "  Device Properties:"
    foreach ($prop in $deviceDetails | Select-Object -First 10) {
        $report += "    $($prop.KeyName): $($prop.Data)"
    }
}
else {
    $report += "FAIL DEVICE NOT FOUND"
    $report += "  The Leyline Audio device is not visible in Device Manager."
    $report += "  This suggests the driver is not loading or the INF registration failed."
}

$report += ""

# Check all audio devices
$report += "All Audio Devices in Device Manager:"
$audioDevices = Get-PnpDevice | Where-Object { $_.Class -eq "AudioEndpoint" -or $_.Class -eq "Media" }
if ($audioDevices) {
    foreach ($dev in $audioDevices) {
        $report += "  - $($dev.FriendlyName) | Status: $($dev.Status) | Class: $($dev.Class)"
    }
}
else {
    $report += "  WARN No audio devices found. This is unusual if the driver loaded."
}

$report += ""

# ============================================================================
# Section 2: Registry Analysis
# ============================================================================
$report += "# SECTION 2: REGISTRY ANALYSIS"
$report += "-" * 80

# Check driver registration
HKLM:\SYSTEM\CurrentControlSet\Services\Leyline | ForEach-Object {
    if (Test-Path $_) {
        $report += "OK Driver Service Registry Key Exists"
        $report += "  Path: HKLM:\SYSTEM\CurrentControlSet\Services\Leyline"

        $service = Get-ItemProperty $_
        $report += "  Type: $($service.Type)"
        $report += "  Start: $($service.Start)"
        $report += "  ErrorControl: $($service.ErrorControl)"
        $report += "  DisplayName: $($service.DisplayName)"
        $report += "  ImagePath: $($service.ImagePath)"
    }
    else {
        $report += "FAIL Driver Service Registry Key NOT FOUND"
    }
}

$report += ""

# Check device interface registration
$report += "Checking Device Interface Registration (HKLM\SYSTEM\CurrentControlSet\Enum\ROOT\Media):"
$enumPath = "HKLM:\SYSTEM\CurrentControlSet\Enum\ROOT\Media"
if (Test-Path $enumPath) {
    $mediaDevices = Get-ChildItem $enumPath | Where-Object { $_.Name -match "Leyline" }
    if ($mediaDevices) {
        $report += "OK Device Enum Key Exists"
        foreach ($dev in $mediaDevices) {
            $report += "  Device: $($dev.Name)"

            # Check device parameters
            $deviceParamsPath = "$($dev.PSPath)\Device Parameters"
            if (Test-Path $deviceParamsPath) {
                $report += "    Device Parameters:"
                Get-Item $deviceParamsPath | Get-ChildItem | ForEach-Object {
                    $propName = $_.PSChildName
                    $propValue = (Get-ItemProperty $_.PSPath).$propName
                    $report += "      $propName = $propValue"
                }
            }
        }
    }
    else {
        $report += "FAIL Device Enum Key NOT FOUND under ROOT\Media"
        $report += "  This indicates the device was not enumerated properly."
    }
}
else {
    $report += "FAIL ROOT\Media Registry Path Does Not Exist"
}

$report += ""

# Check interface registration
$report += "Checking Audio Interface Registration (HKLM\SYSTEM\CurrentControlSet\Control\Class):"
$classPath = "HKLM:\SYSTEM\CurrentControlSet\Control\Class"
if (Test-Path $classPath) {
    $report += "  Interface Classes Found:"
    Get-ChildItem $classPath -ErrorAction SilentlyContinue | Where-Object {
        $prop = Get-ItemProperty $_.PSPath
        $prop.'(default)' -match "Audio" -or $prop.'(default)' -match "Media"
    } | ForEach-Object {
        $prop = Get-ItemProperty $_.PSPath
        $report += "    - $($prop.'(default)')"
    }
}

$report += ""

# ============================================================================
# Section 3: Audio Endpoints Analysis
# ============================================================================
$report += "# SECTION 3: AUDIO ENDPOINTS ANALYSIS"
$report += "-" * 80

# Check using PowerShell audio commands (if available)
try {
    $endpoints = Get-AudioDevice -List
    if ($endpoints) {
        $leylineEndpoints = $endpoints | Where-Object { $_.Name -match "Leyline" }

        if ($leylineEndpoints) {
            $report += "OK Audio Endpoints Found:"
            foreach ($ep in $leylineEndpoints) {
                $report += "  - Endpoint: $($ep.Name)"
                $report += "    ID: $($ep.ID)"
                $report += "    Type: $($ep.Type)"
                $report += "    State: $($ep.State)"
                $report += "    Default: $($ep.Default)"
            }
        }
        else {
            $report += "FAIL No Leyline Audio Endpoints Found"
            $report += "  Available Audio Endpoints:"
            foreach ($ep in $endpoints | Select-Object -First 5) {
                $report += "    - $($ep.Name)"
            }
        }
    }
    else {
        $report += "WARN Get-AudioDevice cmdlet not available (Windows 10/11 AudioCmdlets required)"
    }
}
catch {
    $report += "WARN Cannot query audio endpoints using Get-AudioDevice"
    $report += "  Error: $($_.Exception.Message)"
}

$report += ""

# Check Sound Control Panel registry entries
$report += "Checking Sound Control Panel Registry (HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio):"
$audioDevicesPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio"
if (Test-Path $audioDevicesPath) {
    $report += "OK Audio Devices Registry Path Exists"

    $renderPath = "$audioDevicesPath\Render"
    $capturePath = "$audioDevicesPath\Capture"

    $report += "  Render Devices:"
    if (Test-Path $renderPath) {
        $renderDevices = Get-ChildItem $renderPath -ErrorAction SilentlyContinue | Select-Object -First 10
        foreach ($dev in $renderDevices) {
            $guid = $dev.PSChildName
            $propPath = "$($dev.PSPath)"
            $friendlyName = (Get-ItemProperty $propPath).FriendlyName
            $deviceState = (Get-ItemProperty $propPath).DeviceState
            $report += "    - $guid | Name: $friendlyName | State: $deviceState"
        }
    }
    else {
        $report += "    WARN No render devices in registry"
    }

    $report += "  Capture Devices:"
    if (Test-Path $capturePath) {
        $captureDevices = Get-ChildItem $capturePath -ErrorAction SilentlyContinue | Select-Object -First 10
        foreach ($dev in $captureDevices) {
            $guid = $dev.PSChildName
            $propPath = "$($dev.PSPath)"
            $friendlyName = (Get-ItemProperty $propPath).FriendlyName
            $deviceState = (Get-ItemProperty $propPath).DeviceState
            $report += "    - $guid | Name: $friendlyName | State: $deviceState"
        }
    }
    else {
        $report += "    WARN No capture devices in registry"
    }
}
else {
    $report += "FAIL Audio Devices Registry Path Does Not Exist"
    $report += "  This suggests Windows Audio Subsystem has not initialized or is disabled."
}

$report += ""

# ============================================================================
# Section 4: Audio Service Status
# ============================================================================
$report += "# SECTION 4: AUDIO SERVICE STATUS"
$report += "-" * 80

# Check Windows Audio service
$audioService = Get-Service -Name "Audiosrv" -ErrorAction SilentlyContinue
if ($audioService) {
    $report += "Windows Audio Service (Audiosrv):"
    $report += "  Status: $($audioService.Status)"
    $report += "  StartType: $($audioService.StartType)"
    $report += "  State: $($audioService.Status)"
}
else {
    $report += "FAIL Windows Audio Service (Audiosrv) NOT FOUND"
}

$report += ""

# Check Audio Endpoint Builder service
$aebService = Get-Service -Name "AudioEndpointBuilder" -ErrorAction SilentlyContinue
if ($aebService) {
    $report += "Audio Endpoint Builder Service:"
    $report += "  Status: $($aebService.Status)"
    $report += "  StartType: $($aebService.StartType)"

    if ($aebService.Status -ne "Running") {
        $report += "  WARN WARNING: AEB Service is not running!"
        $report += "  This explains why endpoints are not being created."
    }
}
else {
    $report += "FAIL Audio Endpoint Builder Service NOT FOUND"
}

$report += ""

# Check other audio services
$dependencies = Get-Service -Name "Audiosrv" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty DependentServices
if ($dependencies) {
    $report += "Audio Service Dependencies:"
    foreach ($dep in $dependencies) {
        $status = Get-Service $dep.Name
        $report += "  - $($dep.Name): $($status.Status)"
    }
}

$report += ""

# ============================================================================
# Section 5: Kernel Streaming Analysis
# ============================================================================
$report += "# SECTION 5: KERNEL STREAMING ANALYSIS"
$report += "-" * 80

# Check KS filters
try {
    $ksFilters = Get-WmiObject Win32_SystemDriver | Where-Object { $_.Name -match "ks" -or $_.Name -match "portcls" }
    if ($ksFilters) {
        $report += "KS/PortCls System Drivers:"
        foreach ($filter in $ksFilters) {
            $report += "  - $($filter.Name) | State: $($filter.State) | Started: $($filter.Started)"
        }
    }
    else {
        $report += "WARN No KS/PortCls drivers found"
    }
}
catch {
    $report += "WARN Cannot query KS drivers"
}

$report += ""

# Check for Leyline kernel driver load status
$report += "Checking Leyline Kernel Driver Load Status:"
try {
    $loadedDrivers = Get-Command -ListAvailable -Module | Where-Object { $_.Name -like "*leyline*" }
    if ($loadedDrivers) {
        $report += "  OK Leyline module found in loaded modules"
    }
    else {
        $report += "  WARN Leyline module not found in loaded modules check"
    }
}
catch {
    $report += "  WARN Cannot check loaded modules"
}

$report += ""

# ============================================================================
# Section 6: Summary and Recommendations
# ============================================================================
$report += "# SECTION 6: SUMMARY AND DIAGNOSIS"
$report += "=" * 80

# Collect status indicators
$deviceFound = ($leylineDevice -ne $null)
$aebRunning = ($aebService -and $aebService.Status -eq "Running")
$audioServiceRunning = ($audioService -and $audioService.Status -eq "Running")

$report += "Diagnosis Summary:"
$report += "  Device Found in Device Manager: $(if ($deviceFound) { 'OK' } else { 'FAIL' })"
$report += "  Audio Service Running: $(if ($audioServiceRunning) { 'OK' } else { 'FAIL' })"
$report += "  AEB Service Running: $(if ($aebRunning) { 'OK' } else { 'FAIL' })"

$report += ""

$report += "Root Cause Analysis:"

if (-not $aebRunning) {
    $report += "  PRIMARY ISSUE: Audio Endpoint Builder Service is NOT RUNNING"
    $report += "     This is why no endpoints are being created."
    $report += "     Action: Start the AudioEndpointBuilder service."
    $report += "     Command: Start-Service AudioEndpointBuilder"
}
elseif (-not $deviceFound) {
    $report += "  PRIMARY ISSUE: Leyline Device is NOT FOUND in Device Manager"
    $report += "     The driver may not be loading or INF installation failed."
    $report += "     Action: Verify driver installation and check Device Manager."
}
elseif ($deviceFound -and $aebRunning) {
    $report += "  SECONDARY ISSUE: Device is Registered but AEB is Not Engaging"
    $report += "     The device loads but AEB is not querying properties or creating endpoints."
    $report += "     This indicates an architectural issue with the driver."
    $report += "     Possible causes:"
    $report += "       - Incorrect device class registration"
    $report += "       - Missing or incorrect interface registration"
    $report += "       - Filter stack issues"
    $report += "       - Jack description not being queried"
    $report += "     Action: Add comprehensive logging and check kernel debugger."
}
else {
    $report += "  WARN UNKNOWN ISSUE: Unable to determine root cause from available data"
}

$report += ""

$report += "Recommended Next Steps:"
$report += "  1. Verify all checks above by manually inspecting Device Manager"
$report += "  2. Check Event Viewer (Applications and Services Logs > Microsoft > Windows > Audio)"
$report += "  3. Review the kernel debugger log for any AEB activity or property queries"
$report += "  4. If AEB is not running, start it and restart the audio service"
$report += "  5. If device not found, reinstall the driver using devcon.exe"
$report += "  6. Add comprehensive logging to track all interface queries and property requests"

$report += ""
$report += "=" * 80
$report += "END OF REPORT"
$report += "=" * 80

# Write report to file
$report | Out-File -FilePath $OutputPath -Encoding UTF8 -Force

# Also display to console
$report | ForEach-Object { Write-Host $_ }

Write-Host "`nReport saved to: $OutputPath" -ForegroundColor Cyan
