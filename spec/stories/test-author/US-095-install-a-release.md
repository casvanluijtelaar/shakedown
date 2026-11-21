---
type: User Story
title: "Install a released Shakedown without building it"
description: "A test author installs a released Shakedown on Windows by winget, by the one-line script or by unpacking its zip, with no Rust, HEMTT, Python or clone, and runs a session with it."
status: draft
id: US-095
role: test-author
kind: feature
requirements: [DST-01, DST-02, DST-03, DST-04, NFR-LG-04]
---

# US-095 Install a released Shakedown without building it

As a test author, I want to install Shakedown the way I install HEMTT, so that I can test my mod without building the framework or reading its source.

## Acceptance criteria

1. The release of a version on GitHub holds `shakedown-windows-x64.zip`, its `.sha256` and `install.ps1`, and the zip holds `shakedown.exe`, `@shakedown\`, `LICENSE` and `THIRD-PARTY-LICENSES.txt` and no fixture.
2. On a Windows machine with no Rust, HEMTT or Python on `PATH`, each of the three routes gives an install on `PATH` whose `shakedown version` in a new shell prints the release's version: `winget install` of the package (from the merged package, or from its manifest while the first submission waits), the one-line script, and the zip unpacked by hand into a folder put on `PATH`.
3. After each route, `shakedown run --setup sp` in a project with one passing check gives `pass`, so the binary found `@shakedown\` next to itself.

## Next to

- [US-097](US-097-corrupt-download-refused.md) (system fail): a corrupt or cut-off download is refused.
