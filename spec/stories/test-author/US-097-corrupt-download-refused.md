---
type: User Story
title: "A corrupt or cut-off download is refused"
description: "When the zip the one-line script downloads does not match the release's SHA-256, the script refuses it, says so, and leaves the installed version as it was."
status: draft
id: US-097
role: test-author
kind: system fail
requirements: [DST-03]
---

# US-097 A corrupt or cut-off download is refused

As a test author, I want a broken download stopped before it touches my install, so that a bad network never leaves me with half a framework.

## Acceptance criteria

1. When the downloaded zip's SHA-256 differs from the release's `.sha256`, the script exits with an error that names both hashes and says nothing was installed.
2. After that refusal, the version installed before still runs, and `shakedown version` prints it; on a machine with none installed, the install folder holds no file of the release and `PATH` is unchanged.

## Next to

- [US-095](US-095-install-a-release.md) (feature): install a released Shakedown.
