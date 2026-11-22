# Tests of tools/install.ps1 (DST-03, US-095, US-096, US-097, US-098).
#
#   powershell -NoProfile -File tools/test_install.ps1 [-Exe <shakedown.exe>]
#
# Loads the functions of install.ps1 without running it, installs zips it makes from a built
# shakedown.exe into a temporary folder, and never touches the user's PATH: the PATH step is
# replaced by one that records its argument. Exits 1 when a test fails.

param([string]$Exe)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (-not $Exe) {
    $Exe = @('target\release\shakedown.exe', 'target\debug\shakedown.exe') |
        ForEach-Object { Join-Path $root $_ } | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $Exe) { throw 'no built shakedown.exe; build it or give -Exe' }

$script = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'install.ps1') -Raw
$lines = $script.TrimEnd() -split "`r?`n"
if ($lines[-1] -ne 'Install-Shakedown') { throw 'install.ps1 does not end with Install-Shakedown' }
Invoke-Expression (($lines[0..($lines.Count - 2)]) -join "`n")

$script:PathAdded = @()
function Add-ShakedownToUserPath([string]$Dir) { $script:PathAdded += $Dir }

$failures = 0
function Check([string]$Name, [bool]$Ok, [string]$Detail = '') {
    if ($Ok) { Write-Host "ok   $Name" } else { Write-Host "FAIL $Name $Detail"; $script:failures++ }
}
function Refusal([scriptblock]$Block) {
    try { & $Block; return $null } catch { return $_.Exception.Message }
}

$tmp = Join-Path ([IO.Path]::GetTempPath()) ('shakedown-test-install-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null

# A release folder holding a zip and its .sha256, with LICENSE holding the given marker.
function New-Release([string]$Name, [string]$Marker) {
    $content = Join-Path $tmp "$Name-content"
    New-Item -ItemType Directory -Path "$content\@shakedown\addons" -Force | Out-Null
    Copy-Item -LiteralPath $Exe -Destination "$content\shakedown.exe"
    Set-Content -LiteralPath "$content\@shakedown\shakedown_x64.dll" -Value 'dll'
    Set-Content -LiteralPath "$content\@shakedown\addons\shakedown_main.pbo" -Value 'pbo'
    Set-Content -LiteralPath "$content\LICENSE" -Value $Marker
    Set-Content -LiteralPath "$content\THIRD-PARTY-LICENSES.txt" -Value 'third party'
    $release = Join-Path $tmp $Name
    New-Item -ItemType Directory -Path $release | Out-Null
    $zip = Join-Path $release 'shakedown-windows-x64.zip'
    Compress-Archive -Path "$content\*" -DestinationPath $zip
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $zip).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText("$zip.sha256", "$hash  shakedown-windows-x64.zip`n")
    $release
}

function Get-Marker([string]$Dir) { (Get-Content -LiteralPath "$Dir\LICENSE" -Raw).Trim() }

try {
    $old = New-Release 'old' 'old'
    $new = New-Release 'new' 'new'
    $corrupt = New-Release 'corrupt' 'corrupt'
    $bytes = [IO.File]::ReadAllBytes("$corrupt\shakedown-windows-x64.zip")
    [IO.File]::WriteAllBytes("$corrupt\shakedown-windows-x64.zip", $bytes[0..($bytes.Count / 2)])

    # DST-03: the PATH value names the folder once.
    Check 'DST-03 adds the folder to a PATH without it' ((Add-ToPathValue 'C:\a;C:\b' 'C:\S') -eq 'C:\a;C:\b;C:\S')
    Check 'DST-03 leaves a PATH that has the folder' ($null -eq (Add-ToPathValue 'C:\a;c:\s\;C:\b' 'C:\S'))
    Check 'DST-03 adds to an empty PATH' ((Add-ToPathValue '' 'C:\S') -eq 'C:\S')
    Check 'DST-03 the latest release by default' ((Get-ShakedownSource '' '') -eq 'https://github.com/casvanluijtelaar/shakedown/releases/latest/download')
    Check 'DST-03 SHAKEDOWN_VERSION names the release' ((Get-ShakedownSource '0.2.0-rc.1' '') -eq 'https://github.com/casvanluijtelaar/shakedown/releases/download/v0.2.0-rc.1')

    # US-097: a corrupt zip on a machine with none installed.
    $dir = Join-Path $tmp 'install'
    $refusal = Refusal { Install-Shakedown -Download $corrupt -InstallDir $dir }
    $want = ((Get-Content "$corrupt\shakedown-windows-x64.zip.sha256" -Raw) -split '\s+')[0]
    $got = (Get-FileHash -Algorithm SHA256 "$corrupt\shakedown-windows-x64.zip").Hash.ToLowerInvariant()
    Check 'US-097 a corrupt zip is refused naming both hashes' ($refusal -and $refusal.Contains($want) -and $refusal.Contains($got) -and $refusal.Contains('nothing was installed')) $refusal
    Check 'US-097 with none installed, no install folder and no PATH change' ((-not (Test-Path $dir)) -and $script:PathAdded.Count -eq 0)

    # US-095: install from a release folder.
    $refusal = Refusal { Install-Shakedown -Download $old -InstallDir $dir }
    Check 'US-095 installs' ($null -eq $refusal) $refusal
    Check 'US-095 the install holds the release files' ((Test-Path "$dir\shakedown.exe") -and (Test-Path "$dir\@shakedown\addons\shakedown_main.pbo") -and (Test-Path "$dir\@shakedown\shakedown_x64.dll") -and (Get-Marker $dir) -eq 'old')
    Check 'US-095 adds the folder to PATH' ($script:PathAdded.Count -eq 1 -and $script:PathAdded[0] -eq [IO.Path]::GetFullPath($dir))
    $version = & "$dir\shakedown.exe" version
    Check 'US-095 the installed shakedown runs' ($version -match '^shakedown \d') $version

    # US-097: a corrupt zip leaves the installed version.
    $refusal = Refusal { Install-Shakedown -Download $corrupt -InstallDir $dir }
    Check 'US-097 a corrupt zip over an install is refused' ($refusal -and $refusal.Contains('nothing was installed')) $refusal
    Check 'US-097 the installed version is unchanged and runs' ((Get-Marker $dir) -eq 'old' -and ((& "$dir\shakedown.exe" version) -eq $version))

    # US-098: a running shakedown.exe of the install folder stops the upgrade.
    $info = New-Object Diagnostics.ProcessStartInfo "$dir\shakedown.exe", 'mcp'
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.CreateNoWindow = $true
    $mcp = [Diagnostics.Process]::Start($info)
    try {
        $seen = $false
        for ($i = 0; $i -lt 50 -and -not $seen; $i++) {
            $seen = @(Get-RunningShakedown ([IO.Path]::GetFullPath($dir))).Count -gt 0
            if (-not $seen) { Start-Sleep -Milliseconds 100 }
        }
        Check 'US-098 the running shakedown mcp is found' $seen
        $refusal = Refusal { Install-Shakedown -Download $new -InstallDir $dir }
        Check 'US-098 the upgrade is refused naming the process id and command' ($refusal -and $refusal.Contains("$($mcp.Id)") -and $refusal.Contains('mcp') -and $refusal.Contains('nothing was changed')) $refusal
        Check 'US-098 the install is unchanged' ((Get-Marker $dir) -eq 'old')
    } finally {
        if (-not $mcp.HasExited) { $mcp.Kill() }
        [void]$mcp.WaitForExit(10000)
    }

    # US-096 and US-098: once stopped, running the script again upgrades.
    $refusal = Refusal { Install-Shakedown -Download $new -InstallDir $dir }
    Check 'US-096 running again upgrades' ($null -eq $refusal -and (Get-Marker $dir) -eq 'new') $refusal
    Check 'US-096 the folder is added to PATH by the PATH step, which adds it once' ($script:PathAdded.Count -eq 2 -and $script:PathAdded[1] -eq $script:PathAdded[0])
} finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}

if ($failures -gt 0) { Write-Host "$failures failed"; exit 1 }
Write-Host 'all passed'
