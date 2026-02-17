# Requirements

This document defines the development/runtime requirements for SlipperyStreamEngine v0.1.

## Platform Support (Current)

- Primary: Windows 10/11 (64-bit)
- Target runtime: Desktop Windows

## Required Tools

1. Git
2. Rust toolchain (stable) via `rustup`
3. Cargo (installed with Rust toolchain)
4. MSVC C++ Build Tools (required for Windows Rust `msvc` toolchain builds)
5. GPU drivers with DirectX 12 support (engine currently uses `wgpu` with `dx12` feature)

## Verify Requirements

Run:

```powershell
.\scripts\install_requirements.ps1 -CheckOnly
```

This performs checks and prints what is missing.

## Install Requirements (Windows)

Basic install (Git + Rust toolchain):

```powershell
.\scripts\install_requirements.ps1
```

Full install including Visual Studio Build Tools:

```powershell
.\scripts\install_requirements.ps1 -InstallMSVCBuildTools
```

## Post-Install Validation

Run:

```powershell
rustc --version
cargo --version
cargo build
cargo run -p sme_game
```

## Notes

- If script execution is blocked in PowerShell, run:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

- If `winget` is unavailable, install requirements manually using official installers:
  - Git: https://git-scm.com/download/win
  - Rustup: https://rustup.rs/
  - Visual Studio Build Tools 2022: https://visualstudio.microsoft.com/downloads/
