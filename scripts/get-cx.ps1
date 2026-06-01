<#
.SYNOPSIS
    Installer script for cx (conda-express) on Windows.
.DESCRIPTION
    Downloads and installs the cx binary for Windows, verifies the checksum,
    updates the user PATH, and optionally runs cx bootstrap.
.PARAMETER Version
    Version to install (without "v" prefix). Default: "latest".
    Can also be set via the CX_VERSION environment variable.
.PARAMETER InstallDir
    Directory to install the cx binary into. Default: "$Env:USERPROFILE\.local\bin".
    Can also be set via the CX_INSTALL_DIR environment variable.
.PARAMETER NoPathUpdate
    If specified, skip adding the install directory to the user PATH.
    Can also be set via the CX_NO_PATH_UPDATE environment variable.
.PARAMETER NoBootstrap
    If specified, skip running "cx bootstrap" after installation.
    Can also be set via the CX_NO_BOOTSTRAP environment variable.
.PARAMETER SkipVerify
    If specified, skip checksum verification.
    Can also be set via the CX_SKIP_VERIFY environment variable.
.EXAMPLE
    irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex
.EXAMPLE
    & { $Version = "0.1.3"; irm https://jezdez.github.io/conda-express/get-cx.ps1 | iex }
.LINK
    https://github.com/jezdez/conda-express
#>
param (
    [string] $Version = "latest",
    [string] $InstallDir = "",
    [switch] $NoPathUpdate,
    [switch] $NoBootstrap,
    [switch] $SkipVerify
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Repo = "jezdez/conda-express"

# Environment variable overrides
if ($Env:CX_VERSION) { $Version = $Env:CX_VERSION }
if ($Env:CX_INSTALL_DIR) { $InstallDir = $Env:CX_INSTALL_DIR }
if ($Env:CX_NO_PATH_UPDATE) { $NoPathUpdate = $true }
if ($Env:CX_NO_BOOTSTRAP) { $NoBootstrap = $true }
if ($Env:CX_SKIP_VERIFY) { $SkipVerify = $true }

if (-not $InstallDir) {
    $InstallDir = Join-Path $Env:USERPROFILE ".local\bin"
}

function Get-TargetTriple {
    try {
        $a = [System.Reflection.Assembly]::LoadWithPartialName("System.Runtime.InteropServices.RuntimeInformation")
        $t = $a.GetType("System.Runtime.InteropServices.RuntimeInformation")
        $p = $t.GetProperty("OSArchitecture")
        switch ($p.GetValue($null).ToString()) {
            "X64"   { return "x86_64-pc-windows-msvc" }
            "Arm64" { throw "cx does not publish a Windows ARM64 binary yet." }
        }
    } catch {
        Write-Verbose "Falling back to Is64BitOperatingSystem"
    }

    if ([System.Environment]::Is64BitOperatingSystem) {
        return "x86_64-pc-windows-msvc"
    }

    throw "cx requires a 64-bit Windows system."
}

function Publish-EnvUpdate {
    if (-not ("Win32.NativeMethods" -as [Type])) {
        Add-Type -Namespace Win32 -Name NativeMethods -MemberDefinition @"
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
"@
    }

    $HWND_BROADCAST = [IntPtr] 0xffff
    $WM_SETTINGCHANGE = 0x1a
    $result = [UIntPtr]::Zero

    [Win32.NativeMethods]::SendMessageTimeout(
        $HWND_BROADCAST, $WM_SETTINGCHANGE,
        [UIntPtr]::Zero, "Environment",
        2, 5000, [ref] $result
    ) | Out-Null
}

function Update-UserPath {
    param ([string] $Dir)

    $RegKey = (Get-Item -Path 'HKCU:').OpenSubKey('Environment', $true)
    if (-not $RegKey) {
        throw "Could not open the current user's Environment registry key."
    }

    function Normalize-PathEntry {
        param ([string] $PathEntry)

        if (-not $PathEntry) {
            return ""
        }

        $ExpandedPath = [System.Environment]::ExpandEnvironmentVariables($PathEntry)
        try {
            $ExpandedPath = [System.IO.Path]::GetFullPath($ExpandedPath)
        } catch {
            # Keep the original expanded value if .NET cannot normalize it.
        }

        return $ExpandedPath.TrimEnd(
            [System.IO.Path]::DirectorySeparatorChar,
            [System.IO.Path]::AltDirectorySeparatorChar
        )
    }

    $CurrentPath = $RegKey.GetValue(
        'PATH', '',
        [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames
    )

    $NormalizedDir = Normalize-PathEntry $Dir
    $CurrentEntries = $CurrentPath -split ';' |
        ForEach-Object { Normalize-PathEntry $_ } |
        Where-Object { $_ }

    if ($CurrentEntries -contains $NormalizedDir) {
        Write-Host "  $Dir is already in PATH"
        return
    }

    $NewPath = if ($CurrentPath) { "$Dir;$CurrentPath" } else { $Dir }
    $Kind = if ($NewPath.Contains('%')) {
        [Microsoft.Win32.RegistryValueKind]::ExpandString
    } else {
        [Microsoft.Win32.RegistryValueKind]::String
    }
    $RegKey.SetValue('PATH', $NewPath, $Kind)
    Publish-EnvUpdate

    # Update current session too
    $Env:PATH = "$Dir;$Env:PATH"
    Write-Host "  Added $Dir to user PATH (you may need to restart your terminal)"
}

# Resolve target and URL
$Target = Get-TargetTriple
$AssetName = "cx-${Target}.exe"

if ($Version -eq "latest") {
    $DownloadUrl = "https://github.com/$Repo/releases/latest/download/$AssetName"
} else {
    $V = $Version -replace '^v', ''
    $DownloadUrl = "https://github.com/$Repo/releases/download/$V/$AssetName"
}

Write-Host ""
Write-Host "  Installing cx (conda-express) for Windows ($Target)"
Write-Host "  Downloading $DownloadUrl"

# Download binary
$TempFile = [System.IO.Path]::GetTempFileName()
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
} catch {
    Remove-Item -Path $TempFile -ErrorAction SilentlyContinue
    throw "Download failed: $DownloadUrl`n$_"
}

if ($SkipVerify) {
    Write-Warning "Skipping checksum verification because CX_SKIP_VERIFY is set"
} else {
    $ChecksumUrl = "${DownloadUrl}.sha256"
    $TempSha = [System.IO.Path]::GetTempFileName()
    try {
        Invoke-WebRequest -Uri $ChecksumUrl -OutFile $TempSha -UseBasicParsing
        $Expected = (Get-Content $TempSha -Raw).Trim().Split()[0]
        $Actual = (Get-FileHash -Path $TempFile -Algorithm SHA256).Hash.ToLower()

        if ($Expected -ne $Actual) {
            Remove-Item -Path $TempFile, $TempSha -ErrorAction SilentlyContinue
            throw "Checksum mismatch!`n  expected: $Expected`n  actual:   $Actual"
        }
        Write-Host "  Checksum OK"
    } catch {
        Remove-Item -Path $TempFile, $TempSha -ErrorAction SilentlyContinue
        throw "Checksum verification failed: $ChecksumUrl`n$_"
    } finally {
        Remove-Item -Path $TempSha -ErrorAction SilentlyContinue
    }
}

# Install
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$DestPath = Join-Path $InstallDir "cx.exe"
Move-Item -Path $TempFile -Destination $DestPath -Force

Write-Host "  Installed cx to $DestPath"

# Update PATH
if (-not $NoPathUpdate) {
    Update-UserPath $InstallDir
}

$LegacyPrefix = Join-Path $HOME ".cx"
if (Test-Path $LegacyPrefix) {
    Write-Warning "Found legacy cx prefix at $LegacyPrefix"
    Write-Warning "Current cx bootstraps into $HOME\.conda\express."
    Write-Warning "Keep $LegacyPrefix until you have moved any environments you still need."
    Write-Warning "Upgrade guide: https://jezdez.github.io/conda-express/guides/upgrade-from-early-cx/"
}

# Bootstrap
if (-not $NoBootstrap) {
    Write-Host ""
    Write-Host "  Running cx bootstrap..."
    & $DestPath bootstrap
}

Write-Host ""
Write-Host "  Done! Run 'cx --help' to get started."
Write-Host ""
