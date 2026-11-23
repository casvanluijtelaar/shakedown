---
type: Research
title: The licence and the first public release
description: "Which licence Shakedown can take given the licences compiled into its binaries - GPL-2.0-or-later holds because rmcp, Apache-2.0 only, is compatible with GPLv3 - what winget needs to list a zip and update it from a release workflow, and what the public repository holds that a release must keep out."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-21T12:00:00+01:00 }
sources:
  - { id: owner, resource: "the owner's answers of 2026-11-21: licence GPL-2.0-or-later; a release milestone M19 as 0.2.0; the zip, the one-line script and winget in it, with the winget pull request opened by the release workflow; Windows only", read: 2026-11-21 }
  - { id: meta, resource: "cargo metadata --locked on main at 739a237: the declared licence of every package in Cargo.lock; cargo tree -p shakedown and -p shakedown-ext -e normal", read: 2026-11-21 }
  - { id: fsf, resource: "https://www.gnu.org/licenses/license-list.html, the entry Apache License, Version 2.0 (#apache2)", read: 2026-11-21 }
  - { id: armars, resource: "https://raw.githubusercontent.com/BrettMayson/arma-rs/main/LICENSE and https://crates.io/api/v1/crates/arma-rs (1.13.0, license MIT)", read: 2026-11-21 }
  - { id: releaser, resource: "https://github.com/vedantmgoyal9/winget-releaser README at tag v2 (2025-01-27): prerequisites and the token input", read: 2026-11-21 }
  - { id: komac, resource: "https://github.com/russellbanks/Komac README at v2.17.2 (2026-11-03): komac new, komac update, GitHub token setup", read: 2026-11-21 }
  - { id: def30, resource: "engineering/plan/deferred/30-release-and-install.md: how HEMTT, SQFvm and CBA_A3 ship, looked up on 2026-11-13", read: 2026-11-21 }
  - { id: r20, resource: "engineering/research/20-licenses-and-rules.md: the licences of CBA, ACE3, arma-rs and the T2 tools, and what the EULA allows", read: 2026-11-21 }
  - { id: scan, resource: "git grep and git log -p --all on main at 739a237 for Steam IDs (7656119 followed by 10 digits), user paths, tokens and internal names; gh repo view for the repository's visibility and licence", read: 2026-11-21 }
---

# The licence and the first public release

This topic grounds `CR-70`, `DST-01` to `DST-04` and `NFR-LG-03` and `NFR-LG-04`: the licence Shakedown is released under, what a release on GitHub and winget needs, and what the public repository holds. The owner chose the licence, the routes and the platform[^owner]; this topic checks that each choice holds and finds what it requires.

## The licence

The owner chose GPL-2.0-or-later[^owner], the licence family of HEMTT, CBA_A3 and ACE3 and of arma-grappling-hook[^r20].

What goes into the release's binaries[^meta]:

- **Every dependency is under a permissive licence.** Of the packages in `Cargo.lock`, 119 are `MIT OR Apache-2.0`, 38 are `MIT`, and the rest are other combinations of MIT, Apache-2.0, BSD-2-Clause, Zlib, Unlicense, MIT-0 and Unicode-3.0. None is copyleft except as one choice of an `OR`.
- **Two packages are Apache-2.0 only**: `rmcp` 3.9.0, compiled into `shakedown.exe`, and `unicode-general-category` 1.1.0, in neither shipped binary's normal dependencies.
- **Apache-2.0 is compatible with GPLv3 and not with GPLv2**: "This is a free software license, compatible with version 3 of the GNU GPL. Please note that this license is not compatible with GPL version 2"[^fsf]. GPL-2.0-only would therefore not let `shakedown.exe` be distributed with `rmcp` in it. GPL-2.0-or-later lets it be distributed under GPLv3's terms, which is what the "or later" is for here.
- **`arma-rs` is in the extension only.** crates.io declares 1.13.0 MIT, and its repository's `LICENSE` is the GPL version 2 text[^armars]. The extension holds no Apache-2.0-only package[^meta], so it can be distributed under GPLv2 or GPLv3 whichever reading of `arma-rs` holds. The question research 20 left to the owner no longer blocks anything[^r20].
- **The licence does not lift the game's terms.** GPL governs the framework's own code. What a user may do with Arma 3 is still the EULA's: the PRD's constraint of non-commercial mod testing binds the use of the game, not the use of the code[^r20].

What GPL-2.0-or-later and the permissive licences ask of a release:

- the `LICENSE` file with the GPL version 2 text and a statement that later versions may be chosen, in the repository and in the release zip;
- the licence text and copyright notice of every package compiled into a shipped binary, since MIT, BSD and Apache-2.0 each require their notice to go with copies in binary form; Apache-2.0 adds any `NOTICE` file the package carries;
- the corresponding source, which the tag on the public repository gives for every release.

## Shipping on GitHub and winget

How HEMTT, SQFvm and CBA_A3 ship was looked up for `DEF-30`: a zip on GitHub Releases, winget with a manifest of type `zip` and `portable` for HEMTT, and no installer program[^def30].

- **winget lists a package only after one version is in `microsoft/winget-pkgs`.** The release action names it a prerequisite: "At least one version of your package should already be present in the Windows Package Manager Community Repository"[^releaser]. The first version is submitted with `komac new`[^komac], from the owner's account, and is merged when the repository's maintainers accept it, a review the project does not control.
- **Each later version can be submitted by the release workflow**, with Komac (`komac update <identifier> --version <v> --urls <zip url>`) or with `winget-releaser`, which wraps it[^komac][^releaser].
- **Both need a classic personal access token with the `public_repo` scope**, and a fork of `microsoft/winget-pkgs` under the owner's account; fine-grained tokens are not supported. `winget-releaser` asks for the `workflow` scope too, or the fork is synced by hand[^releaser][^komac]. The token is kept as a repository secret, never in the workflow file[^releaser]. `public_repo` lets the token write to every public repository of the account, so it is the widest credential the project holds, and it is used only by the release job on a tag.

## What the public repository holds

The repository on GitHub is public and has no licence[^scan]. Searching it and its whole history[^scan]:

- **No Steam ID.** The one number in the form of a Steam ID is the made-up `76561190000000042` of the redaction test in `crates/shakedown/src/logs.rs`. The Steam ID printed in the M16 build reached no file or commit, as the coordinator playbook records.
- **No token, password or user path.**
- **The owner's name and work e-mail** are in the commit metadata and in the coordinator playbook's note on the git identity, and the internal address of the agentic engineering guide is in `CLAUDE.md`. All of them are already public; whether to keep them is the owner's call.
- **The fixture mods** are this repository's own test targets. They belong in the source and stay out of the release zip, as `DEF-30` recommends[^def30].

## What this means for the spec

- `NFR-LG-03` names GPL-2.0-or-later instead of a licence still to come, and `NFR-LG-04` makes the release carry `LICENSE` and every compiled package's notice. Touches the PRD's section 7.
- `CR-70` and `DST-01` to `DST-04` specify a release from a tag, winget, the one-line script and the zip by hand, Windows only[^owner].
- The first winget version is a step of the owner's, and the milestone that builds the release counts winget as done when that submission is open and a local install from its manifest works, since the merge is not the project's to make.

[^owner]: The owner's answers of 2026-11-21.
[^meta]: `cargo metadata --locked` and `cargo tree` on main at 739a237.
[^fsf]: GNU, "Various Licenses and Comments about Them", Apache License, Version 2.0.
[^armars]: `arma-rs` LICENSE on its main branch, and its crates.io record.
[^releaser]: `vedantmgoyal9/winget-releaser` README.
[^komac]: `russellbanks/Komac` README.
[^def30]: `engineering/plan/deferred/30-release-and-install.md`.
[^r20]: `engineering/research/20-licenses-and-rules.md`.
[^scan]: The search of the repository and its history on 2026-11-21.
