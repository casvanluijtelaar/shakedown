---
type: Deferred
title: "A winget update from release.yml may drop ArchiveBinariesDependOnPath"
description: "The first winget manifest got ArchiveBinariesDependOnPath: true by hand, since komac new does not ask for it, and nothing has shown that the komac update release.yml runs for each later version carries it over from the previous manifest; without it winget links only shakedown.exe onto PATH and the binary does not find @shakedown."
id: DEF-36
state: open
milestone: null
status: draft
---

# A winget update from release.yml may drop ArchiveBinariesDependOnPath

## Summary

`DST-02` asks for a winget manifest with `ArchiveBinariesDependOnPath: true`, so winget puts the package's folder on `PATH` and `shakedown.exe` finds `@shakedown\` next to itself. For `0.2.0`, `komac new` (2.17.2) wrote the manifests without that key, because it does not ask for it, and the key was added by hand before `winget validate` and `komac submit` (microsoft/winget-pkgs#487410). Every later version is submitted by `release.yml` with `komac update casvanluijtelaar.Shakedown --version <v> --urls <zip> --submit`, which builds the new manifest from the one in `microsoft/winget-pkgs`. Whether it keeps `ArchiveBinariesDependOnPath` is not proved.

## Current behaviour

- `release.yml`, on a tag with no pre-release part whose package is already in `microsoft/winget-pkgs`, downloads Komac 2.17.2, checks its SHA-256 and runs `komac update ... --submit` with `WINGET_TOKEN`. It does not read the manifest it submits.
- The first manifest also had leading spaces in three locale fields, which Komac kept and which were trimmed by hand; whether `komac update` copies those fields from the merged manifest is not seen either.
- No version after `0.2.0` exists, so `komac update` has never run for this package.

## The gap

If `komac update` drops the key, the next version's manifest makes winget link `shakedown.exe` alone onto `PATH`. `shakedown` would run but not find `@shakedown\`, and every session start of a winget install would fail until a fixed manifest is merged. winget's validation of the pull request would not catch it.

## Options

1. **Look before the next release.** Locally, run `komac update` for the next version without `--submit`, and read the installer manifest it writes.
2. **Check in release.yml.** Have Komac write the manifests to a folder, fail the job when the installer manifest lacks `ArchiveBinariesDependOnPath: true`, and submit only after the check.
3. **Leave it** and read the pull request `release.yml` opens before the winget maintainers merge it.

## Recommendation

Option 1 at the next release, before its tag is pushed. If the key survives, nothing more is needed; if it does not, option 2.

## Acceptance criteria

- The manifest `komac update` makes for the version after `0.2.0` holds `ArchiveBinariesDependOnPath: true`, seen before it is submitted.
- After that version merges, `winget upgrade casvanluijtelaar.Shakedown` gives a `shakedown` whose `session start` finds `@shakedown\`.
