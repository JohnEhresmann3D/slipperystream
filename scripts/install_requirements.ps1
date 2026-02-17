param(
    [switch]$CheckOnly,
    [switch]$InstallMSVCBuildTools
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "[setup] $Message"
}

function Has-Command {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Ensure-Winget {
    if (-not (Has-Command "winget")) {
        throw "winget is required for automated install. Install App Installer from Microsoft Store, or install dependencies manually."
    }
}

function Install-WithWinget {
    param(
        [string]$Id,
        [string]$Name,
        [string]$ExtraArgs = ""
    )

    Write-Step "Installing $Name ($Id)..."
    $args = @(
        "install",
        "--id", $Id,
        "--exact",
        "--accept-package-agreements",
        "--accept-source-agreements"
    )

    if ($ExtraArgs -ne "") {
        $args += @("--override", $ExtraArgs)
    }

    & winget @args
}

function Check-State {
    $state = [ordered]@{
        Git = Has-Command "git"
        Rustc = Has-Command "rustc"
        Cargo = Has-Command "cargo"
        Rustup = Has-Command "rustup"
        Winget = Has-Command "winget"
        Cl = Has-Command "cl"
    }
    return $state
}

function Print-State {
    param($State)
    Write-Host ""
    Write-Host "Requirement status:"
    foreach ($kv in $State.GetEnumerator()) {
        $status = if ($kv.Value) { "OK" } else { "MISSING" }
        Write-Host ("- {0}: {1}" -f $kv.Key, $status)
    }
    Write-Host ""
}

$initial = Check-State
Print-State -State $initial

if ($CheckOnly) {
    Write-Step "Check-only mode complete."
    exit 0
}

Ensure-Winget

if (-not $initial.Git) {
    Install-WithWinget -Id "Git.Git" -Name "Git"
} else {
    Write-Step "Git already installed."
}

if (-not $initial.Rustup) {
    Install-WithWinget -Id "Rustlang.Rustup" -Name "Rustup"
} else {
    Write-Step "Rustup already installed."
}

if ($InstallMSVCBuildTools) {
    if (-not $initial.Cl) {
        Install-WithWinget `
            -Id "Microsoft.VisualStudio.2022.BuildTools" `
            -Name "Visual Studio Build Tools 2022" `
            -ExtraArgs "--quiet --wait --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
    } else {
        Write-Step "MSVC compiler (cl.exe) already available."
    }
} else {
    Write-Step "Skipping Visual Studio Build Tools installation (use -InstallMSVCBuildTools to include it)."
}

if (Has-Command "rustup") {
    Write-Step "Ensuring stable toolchain is installed..."
    & rustup toolchain install stable
    & rustup default stable
}

$final = Check-State
Print-State -State $final

if (-not $final.Git -or -not $final.Rustc -or -not $final.Cargo -or -not $final.Rustup) {
    throw "One or more required tools are still missing."
}

if (-not $final.Cl) {
    Write-Host "Note: cl.exe not detected. If builds fail on MSVC toolchain, re-run with -InstallMSVCBuildTools."
}

Write-Step "Done. Open a new terminal and run: cargo build"
