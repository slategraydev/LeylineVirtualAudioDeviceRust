# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

param (
    [string]$InxPath = "crates/leyline-kernel/leyline.inx"
)

$Date = Get-Date -Format "MM/dd/yyyy"
# Generate a version number based on today's session (e.g., 1.0.13.xxx)
$SessionNum = 13
$Minutes = (Get-Date).Hour * 60 + (Get-Date).Minute
$Version = "1.0.$SessionNum.$Minutes"

$Content = Get-Content $InxPath
$NewContent = $Content -replace "DriverVer\s*=.*", "DriverVer   = $Date,$Version"
$NewContent | Set-Content $InxPath

Write-Host "[*] Updated $InxPath to $Date,$Version"
