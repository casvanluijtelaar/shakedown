---
type: Deferred
title: "Shakedown installs only by building it from this repository"
description: "tools/install.py builds the framework with cargo, HEMTT and Python, so a test author needs the whole toolchain and a clone to install it; a release is to ship as a zip on GitHub Releases, installed by winget, by a one-line script or by hand, as HEMTT does."
id: DEF-30
state: open
milestone: null
status: stable
---

# Shakedown installs only by building it from this repository

## Summary

The only way to install Shakedown is `python tools/install.py` in a clone of this repository (`docs/install.md`). A test author who only wants to test their mod needs Rust, HEMTT and Python on `PATH`, and builds every crate and the addon on their own machine. Arma developer tools install from a published release instead. Shakedown is to ship the same way: a zip on GitHub Releases, installed by winget on Windows, by a one-line script, or by downloading the zip by hand.

## Current behaviour

`tools/install.py` runs `cargo build --release` and `hemtt build` for the addon and the fixture mods, and assembles the install layout of `spec/07-tech-stack.md` into `target\shakedown\`: `shakedown.exe`, `@shakedown\` with `addons\shakedown_main.pbo` and `shakedown_x64.dll`, and `fixtures\`. The binary finds `@shakedown` next to itself. The user puts the folder on `PATH` by hand. Upgrading means pulling and running the script again. Nothing is published: the repository is private, it has no `LICENSE` file, and `.github/workflows/ci.yml` runs only the gates that need no game.

## The gap

- A test author cannot install Shakedown without the build toolchain and access to the source.
- There is no versioned artifact, so a result file's framework version cannot be traced to a download, and there is no way to install a given version or to upgrade.
- Putting the folder on `PATH` is a manual step that every install repeats.

## How comparable tools ship

Looked up on 2026-11-13.

- **HEMTT** (`BrettMayson/HEMTT`, `book/installation/index.md`) offers four routes: `winget install hemtt` on Windows, `curl -sSf https://hemtt.dev/install.sh | sh` on Linux and macOS, a manual download from GitHub Releases, and `cargo install` from source. Its release workflow (`.github/workflows/build.yaml`) builds each platform with cargo, zips the binaries (`windows-x64.zip`, `linux-x64.zip`) and attaches them to a GitHub Release with `softprops/action-gh-release`.
- **HEMTT's winget package** (`BrettMayson.HEMTT` in `microsoft/winget-pkgs`) has no installer program. Its manifest is `InstallerType: zip` with `NestedInstallerType: portable` and `hemtt.exe` as the nested file, the release zip's URL and SHA-256, and a dependency on `Microsoft.VCRedist.2015+.x64`. winget downloads the zip, checks the hash, unpacks it and puts the exe on `PATH`. What it installs is a small launcher (`hemtt/launcher`) that downloads and switches HEMTT versions itself, so the manifest needs a new version only when the launcher changes.
- **SQFvm** and **CBA_A3** attach a zip to each GitHub release and offer nothing else; mods such as CBA_A3 and ACE3 also ship on the Steam Workshop.
- None of these ships an `.exe` or `.msi` installer, and none uses Scoop or Chocolatey as its main route.

## Options

1. **Release zip, winget, a one-line script and a manual download.** A workflow on a `v*` tag builds on `windows-latest` with cargo and HEMTT, zips `shakedown.exe` and `@shakedown\` into `shakedown-windows-x64.zip`, and attaches it to a GitHub Release with its SHA-256. A winget manifest of type `zip` + `portable` names `shakedown.exe` as the nested file; winget unpacks the whole zip, so `@shakedown\` lands next to the exe as the binary expects. The release workflow opens the winget pull request for each version with `komac` or `wingetcreate`. A PowerShell script, run as `irm <url>/install.ps1 | iex`, downloads the latest zip, checks its hash, unpacks it to a folder under the user's profile and adds that folder to the user's `PATH`. On macOS and Linux, where the binary alone is the install (`NFR-PT-02`), an `install.sh` does the same for a `shakedown` binary built there with cargo.
2. **As the first, behind a launcher** like HEMTT's, so winget installs a launcher once and the launcher fetches each Shakedown version. This saves a winget pull request per version, at the cost of a second program to build and keep working.
3. **A Windows installer** (`.msi`, or Inno Setup). It adds a start-menu entry and an uninstaller, which a command-line tool has no use for, and no comparable Arma tool ships one.
4. **The Steam Workshop for `@shakedown`.** Steam updates a Workshop item on its own, so the addon could drift from the version of `shakedown.exe` it must match.

## Recommendation

The first. It is the route HEMTT takes, and the three ways in share one artifact: the release zip. Leave out the launcher until releases come often enough that a winget pull request per version is a burden. The release zip leaves out `fixtures\`: the fixture mods serve this repository's own tests, and `tools/install.py` keeps building them for that. `tools/install.py` stays as the install from source.

Before the first release:

- the repository is public, since winget, the script and a manual download all need a URL that needs no login;
- the repository has a `LICENSE`, which the winget manifest requires and which `spec/client-requirements.md` section 7 already keeps open;
- the binaries are unsigned, as HEMTT's are, so Windows SmartScreen warns on the first run of a download. Code signing waits until users show it matters.

The versions of every action and tool the workflow uses, and the winget manifest schema, are looked up when the work is done.

## Acceptance criteria

1. Pushing a `v*` tag builds a GitHub Release with `shakedown-windows-x64.zip` and its SHA-256, holding `shakedown.exe` and `@shakedown\` and nothing that needs the source to run. The tag's version matches the workspace version in `Cargo.toml`.
2. On a Windows machine with no Rust, HEMTT or Python, `winget install` with the package's identifier installs Shakedown, and `shakedown --version` in a new shell prints the release version. `winget upgrade` installs a newer release and `winget uninstall` removes it.
3. On the same kind of machine, the PowerShell one-liner installs the latest release with the same result as winget, refuses a download whose hash does not match, and run again replaces the installed version.
4. Unpacking the zip by hand into a folder on `PATH` gives the same result.
5. After each of the three, `shakedown run --setup sp` on a project runs its checks, which shows the binary found `@shakedown\` next to itself.
6. On macOS and Linux, `install.sh` installs a `shakedown` binary that serves `list`, `status`, `guide` and the MCP server's start (`NFR-PT-02`).
7. `docs/install.md`, the root `README.md` and `spec/07-tech-stack.md` describe the three routes first and the install from source last.
