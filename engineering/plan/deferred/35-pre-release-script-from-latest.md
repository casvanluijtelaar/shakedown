---
type: Deferred
title: "The one-line install fetches its script from the latest release"
description: "The documented irm line takes install.ps1 from releases/latest, which GitHub never points at a pre-release, so it cannot install anything while only pre-releases exist, and with SHAKEDOWN_VERSION set it still runs the latest release's script; installing a pre-release reliably needs the tag's own script URL, which docs/install.md does not give."
id: DEF-35
state: open
milestone: null
status: draft
---

# The one-line install fetches its script from the latest release

## Summary

`docs/install.md`, the root `README.md` and `DST-03` give one line for the script route: `irm https://github.com/casvanluijtelaar/shakedown/releases/latest/download/install.ps1 | iex`. GitHub's `releases/latest` names the newest release that is not a pre-release. While only pre-releases exist the URL answers 404, and when a pre-release is newer than the latest release the line runs the older release's script. `SHAKEDOWN_VERSION` picks the zip, not the script. M19's install of `0.2.0-rc.1` worked because the owner took the script from the tag's own URL, `releases/download/v0.2.0-rc.1/install.ps1`.

## Current behaviour

- `install.ps1` reads `SHAKEDOWN_VERSION`; set, it downloads the zip and its `.sha256` from `releases/download/v<version>`, and unset, from `releases/latest/download`.
- The script itself is fetched by the `irm` URL, before it reads any variable. With the documented line that is always the latest release's script.
- `docs/install.md` gives `$env:SHAKEDOWN_VERSION = '0.2.0-rc.1'` as the way to install a pre-release beside the `latest` line, and says nothing about where the script comes from.
- Since `v0.2.0` is published as a release, the `latest` line works for `0.2.0`, and with `SHAKEDOWN_VERSION` set it installs a pre-release through `0.2.0`'s script as long as that script fits the pre-release's zip.

## The gap

- While no release exists, as before `v0.2.0`, the documented line fails with a 404 that names no cause, also with `SHAKEDOWN_VERSION` set.
- A pre-release whose zip or install steps differ from the latest release's is installed by a script that does not know them.

## Options

1. **Document the tag's URL.** `docs/install.md` and `DST-03` give `irm https://github.com/casvanluijtelaar/shakedown/releases/download/v<version>/install.ps1 | iex` with `SHAKEDOWN_VERSION` set to the same version, for a pre-release or any given version. No code.
2. **Stamp the version into each release's script.** `release.yml` writes the tag's version into the `install.ps1` it attaches, as the default when `SHAKEDOWN_VERSION` is unset, so a tag's own script installs its own release with no variable. The `latest` line is unchanged.
3. **Leave it.** A release exists, and pre-releases are rare.

## Recommendation

Option 1 at the next pre-release, or sooner if a user asks: a sentence and a line in `docs/install.md` and `DST-03`. Option 2 waits until a pre-release needs a script the latest release's does not have.

## Acceptance criteria

- `docs/install.md` and `DST-03` say that the `latest` line never installs a pre-release by itself, and give the tag's own script URL with `SHAKEDOWN_VERSION` for a pre-release or a given version.
- That line, run in a shell with no Rust, HEMTT or Python on `PATH`, installs a published pre-release, and `shakedown version` prints its version.
