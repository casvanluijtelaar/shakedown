# Installs a released Shakedown for the current user, with no administrator rights (DST-03).
#
#   irm https://github.com/casvanluijtelaar/shakedown/releases/latest/download/install.ps1 | iex
#
# It downloads shakedown-windows-x64.zip and its .sha256, refuses a zip whose SHA-256 differs,
# unpacks it into %LOCALAPPDATA%\Programs\Shakedown and adds that folder to the user's PATH once.
# Run again, it replaces the installed version. While a shakedown.exe of that folder runs, it
# refuses before it changes any file. Nothing changes until the download is checked.
#
#   $env:SHAKEDOWN_VERSION      a release to install instead of the latest, such as 0.2.0-rc.1
#   $env:SHAKEDOWN_DOWNLOAD     a URL or a local folder holding the zip and its .sha256
#   $env:SHAKEDOWN_INSTALL_DIR  the folder to install into
#
# Shakedown is licensed GPL-2.0-or-later; see LICENSE in the installed folder.

$ShakedownZip = 'shakedown-windows-x64.zip'
# The files of a release in the install folder, removed before the new ones are copied in.
$ShakedownFiles = @('shakedown.exe', '@shakedown', 'LICENSE', 'THIRD-PARTY-LICENSES.txt')

function Get-ShakedownSource([string]$Version, [string]$Download) {
    if ($Download) { return $Download.TrimEnd('/', '\') }
    if ($Version) { return "https://github.com/casvanluijtelaar/shakedown/releases/download/v$Version" }
    'https://github.com/casvanluijtelaar/shakedown/releases/latest/download'
}

function Get-ShakedownFile([string]$Source, [string]$Name, [string]$Destination) {
    if ($Source -notmatch '^[a-z]+://' -and (Test-Path -LiteralPath $Source -PathType Container)) {
        Copy-Item -LiteralPath (Join-Path $Source $Name) -Destination $Destination
    } else {
        Invoke-WebRequest -UseBasicParsing -Uri "$Source/$Name" -OutFile $Destination
    }
}

# Every shakedown.exe process started from the install folder: a runner, an AI agent's
# shakedown mcp, or any other command of that install.
function Get-RunningShakedown([string]$Dir) {
    $exe = Join-Path $Dir 'shakedown.exe'
    @(Get-CimInstance Win32_Process -Filter "Name = 'shakedown.exe'" |
        Where-Object { $_.ExecutablePath -and ([IO.Path]::GetFullPath($_.ExecutablePath) -ieq $exe) })
}

function Assert-ShakedownNotRunning([string]$Dir) {
    # A single CimInstance has no Count, so the result is wrapped in an array here.
    $running = @(Get-RunningShakedown $Dir)
    if ($running.Count -gt 0) {
        $list = ($running | ForEach-Object { "  $($_.ProcessId)  $($_.CommandLine)" }) -join "`n"
        throw ("Shakedown is running from $Dir, so nothing was changed. Stop these processes first, " +
            "then run the install again:`n$list`n" +
            "Stop a session with 'shakedown session stop'; its runner then exits by itself within 60 s. " +
            "End the AI agent sessions that run 'shakedown mcp'.")
    }
}

# The PATH value with Dir added at the end and the rest left as it is, or $null when Dir is in it.
function Add-ToPathValue([string]$Current, [string]$Dir) {
    foreach ($part in ($Current -split ';')) {
        if ($part -and $part.TrimEnd('\') -ieq $Dir.TrimEnd('\')) { return $null }
    }
    if (-not $Current.TrimEnd(';')) { return $Dir }
    $Current.TrimEnd(';') + ';' + $Dir
}

# Adds Dir to the user's PATH in the registry, keeping its other entries unexpanded, and tells
# running programs so that a new terminal sees it. Also adds it to this shell's PATH.
function Add-ShakedownToUserPath([string]$Dir) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    try {
        $current = $key.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $new = Add-ToPathValue $current $Dir
        if ($null -ne $new) {
            $kind = [Microsoft.Win32.RegistryValueKind]::ExpandString
            if ($null -ne $key.GetValue('Path')) { $kind = $key.GetValueKind('Path') }
            $key.SetValue('Path', $new, $kind)
            if (-not ('Shakedown.SettingChange' -as [type])) {
                Add-Type -Namespace Shakedown -Name SettingChange -MemberDefinition @'
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
public static extern IntPtr SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
'@
            }
            $result = [UIntPtr]::Zero
            # HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG, at most 5 s.
            [void][Shakedown.SettingChange]::SendMessageTimeout([IntPtr]0xffff, 0x1A, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result)
            Write-Host "Added $Dir to your PATH."
        }
    } finally {
        $key.Close()
    }
    $session = Add-ToPathValue $env:Path $Dir
    if ($null -ne $session) { $env:Path = $session }
}

function Install-Shakedown {
    param(
        [string]$Version = $env:SHAKEDOWN_VERSION,
        [string]$Download = $env:SHAKEDOWN_DOWNLOAD,
        [string]$InstallDir = $env:SHAKEDOWN_INSTALL_DIR
    )
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'
    if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\Shakedown' }
    $InstallDir = [IO.Path]::GetFullPath($InstallDir)
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    Assert-ShakedownNotRunning $InstallDir
    $source = Get-ShakedownSource $Version $Download
    $work = Join-Path ([IO.Path]::GetTempPath()) ('shakedown-install-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $work | Out-Null
    try {
        $zip = Join-Path $work $ShakedownZip
        Write-Host "Downloading $ShakedownZip from $source"
        try {
            Get-ShakedownFile $source $ShakedownZip $zip
            Get-ShakedownFile $source "$ShakedownZip.sha256" "$zip.sha256"
        } catch {
            throw "The download from $source failed, so nothing was installed: $($_.Exception.Message)"
        }
        $want = ((Get-Content -LiteralPath "$zip.sha256" -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
        $got = (Get-FileHash -Algorithm SHA256 -LiteralPath $zip).Hash.ToLowerInvariant()
        if ($got -ne $want) {
            throw ("The downloaded $ShakedownZip has SHA-256 $got, but the release's $ShakedownZip.sha256 " +
                "says $want. The download is corrupt or cut off, so nothing was installed.")
        }
        $staged = Join-Path $work 'staged'
        Expand-Archive -LiteralPath $zip -DestinationPath $staged
        if (-not (Test-Path -LiteralPath (Join-Path $staged 'shakedown.exe'))) {
            throw "$ShakedownZip holds no shakedown.exe, so nothing was installed."
        }

        Assert-ShakedownNotRunning $InstallDir
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        foreach ($name in $ShakedownFiles) {
            $old = Join-Path $InstallDir $name
            if (Test-Path -LiteralPath $old) { Remove-Item -LiteralPath $old -Recurse -Force }
        }
        Get-ChildItem -LiteralPath $staged | Copy-Item -Destination $InstallDir -Recurse -Force
        Add-ShakedownToUserPath $InstallDir
        $installed = & (Join-Path $InstallDir 'shakedown.exe') version
        Write-Host "Installed $installed in $InstallDir. Open a new terminal to use 'shakedown'."
    } finally {
        Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Install-Shakedown
