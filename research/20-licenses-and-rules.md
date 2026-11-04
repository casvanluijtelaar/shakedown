---
type: Research
title: Licenses, the EULA and Bohemia's rules
description: What the Arma 3 EULA, Bohemia's tools license and content rules, Steam's terms, the BI wiki's terms and the licenses of CBA, ACE3 and the T2 tools allow a test framework to do, and what personal data its logs hold.
status: stable
generated: { by: "claude-code/research", at: 2026-11-04T21:58:00+01:00 }
sources:
  - { id: eula, resource: "https://www.bohemia.net/en/licenses/arma3-end-user-license (Arma 3 End User License Agreement, updated 20/08/2026)", read: 2026-11-04 }
  - { id: toolseula, resource: "https://www.bohemia.net/en/licenses/bohemia-interactives-tools-end-user-license (Bohemia Interactive Tools End User License, updated 20/08/2026)", read: 2026-11-04 }
  - { id: gcur, resource: "https://www.bohemia.net/en/community/game-content-usage-rules (Game Content Usage Rules, updated 21/08/2026)", read: 2026-11-04 }
  - { id: apl, resource: "https://www.bohemia.net/en/licenses/arma-public-license (APL), https://www.bohemia.net/en/licenses/arma-public-license-share-alike (APL-SA), https://www.bohemia.net/en/licenses/arma-public-license-nd (APL-ND)", read: 2026-11-04 }
  - { id: licfaq, resource: "https://www.bohemia.net/en/licenses/faq (Licenses FAQ)", read: 2026-11-04 }
  - { id: ssa, resource: "https://store.steampowered.com/subscriber_agreement/ (Steam Subscriber Agreement, last updated September 10, 2026), sections 2 and 4", read: 2026-11-04 }
  - { id: wikiterms, resource: "https://community.bistudio.com/wiki/Meta:General_disclaimer (target of Bohemia_Interactive_Community:Copyrights, revision of 2023-05-25)", read: 2026-11-04 }
  - { id: wikiext, resource: "https://community.bistudio.com/wiki/Extensions (revision 379582, 2026-11-03): history, FAQ, BattlEye whitelisting, Context arguments", read: 2026-11-04 }
  - { id: uid, resource: "https://community.bistudio.com/wiki/getPlayerUID (revision 353295)", read: 2026-11-04 }
  - { id: startup, resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters (revision 378577): -name, -profiles, -noLogs", read: 2026-11-04 }
  - { id: gdpr, resource: "https://gdpr-info.eu/art-4-gdpr/ and https://gdpr-info.eu/art-2-gdpr/ (GDPR Article 4(1) and Article 2(2)(c))", read: 2026-11-04 }
  - { id: cba, resource: "https://github.com/CBATeam/CBA_A3 at 8b15a04bd52ff7acc568a93cf6e90343e43a8a56 (LICENSE.md, README.md License section, addons/*/license.txt)", read: 2026-11-04 }
  - { id: ace, resource: "https://github.com/acemod/ACE3 at 45c25247f7599687e85c5981f150efe37227393b (LICENSE, addons/apl/LICENSE, per-folder LICENSE files)", read: 2026-11-04 }
  - { id: tools, resource: "research/11-arma-tools.md (T2): repositories, commits and LICENSE files of the Arma tools", read: 2026-11-04 }
  - { id: neefay, resource: "https://github.com/Neefay/arma3-llm-harness at 21aac67e8184901e4016ed4effa06c99afd2e067 (docs/ARCHITECTURE.md, host/src/arma_harness/serverroot.py)", read: 2026-11-04 }
  - { id: headswe, resource: "https://github.com/headswe/arma3-mcp at d778d75e87bf7590793297e1afb63558c8b88f14 (README.md, list_players tool)", read: 2026-11-04 }
  - { id: armars, resource: "https://github.com/BrettMayson/arma-rs at 8e7b243ccf6fcf98fbe83a9dd86982336ac660d1 (LICENSE, Cargo.toml) and https://crates.io/api/v1/crates/arma-rs", read: 2026-11-04 }
  - { id: ade, resource: "https://github.com/dedmen/ArmaDebugEngine at e01f80e42665082bf209046dd97f87787d5c0547 (LICENSE)", read: 2026-11-04 }
---

# Licenses, the EULA and Bohemia's rules

This topic reads the texts that bind the framework: the Arma 3 EULA, Bohemia's tools license, content rules and public licenses, Steam's Subscriber Agreement, the BI wiki's terms, and the licenses of CBA, ACE3 and the tools of T2. It says what each means for three things:
- (a) a tool that drives the game and loads an extension DLL;
- (b) several copies of the game on one machine;
- (c) reusing code in a project that may be open-sourced later.

This is not legal advice. Where a text can be read more than one way, the question is marked **owner's judgment**.

## The texts

### The Arma 3 EULA

The Arma 3 EULA was updated 20/08/2026[^eula]. It says:

- **Grant.** "a restricted, non-exclusive license and right to install and use one (1) copy of the Program for your personal use. You may not network the Program or otherwise install it or use it on more than one computer at a time, except if expressly authorized otherwise in the applicable documentation"[^eula].
- **Obligations.** You may not "copy, duplicate, reproduce, translate, reverse-engineer, modify, disassemble, decompile, derive source code, create derivative works based on the Program ... or otherwise modify the Program without the prior written consent of the Licensor"[^eula].
- **Purpose.** You may not "Use the Program for anything other than entertainment and recreation purposes, including but not limited to using the Program for training, educational, occupational or vocational purposes", nor exploit it "for any commercial purpose"[^eula].

### Bohemia's Tools EULA

The Tools EULA covers Bohemia's own tools, such as the Arma 3 Tools, and was updated 20/08/2026[^toolseula]. It allows use only for "designing, developing, testing, and producing non-commercial game content for computer games developed by the Licensor"[^toolseula]. It forbids commercial exploitation of content made with the tools, and has the same one-copy and reverse-engineering clauses as the game EULA[^toolseula].

### Bohemia's Game Content Usage Rules

The Game Content Usage Rules were updated 21/08/2026[^gcur]. They are a plain-language guide that does not replace the EULA[^gcur], and say:

- "You may develop your own game content, tools, plug-ins and other utilities or services for both noncommercial and commercial use as long as it is your original creation, without using the content, tool or any other intellectual property of Bohemia Interactive"[^gcur].
- On dedicated servers: "Can I host an Arma dedicated server? Yes ... You are not allowed to hack or modify the dedicated server"[^gcur].
- On reverse engineering: "You may not reverse engineer or hack our games to access the content you want to make items from or to change the game's functionality"[^gcur].
- On names and logos: a mod's own name may go on merchandise, but not the Arma 3 logo or other Bohemia trademarks[^gcur].

### Arma Public Licenses (APL, APL-SA, APL-ND)

These grant rights "for NonCommercial and ArmaOnly purposes only". ArmaOnly means "primarily intended for or directed towards the use in any of existing and future Arma games"[^apl]. APL-SA adds that an adaptation may be shared "only under the same license"[^apl]. Bohemia's FAQ says that a mod made entirely by its author may use any license[^licfaq].

### The Steam Subscriber Agreement

The Steam Subscriber Agreement was last updated September 10, 2026[^ssa].

- **Section 4B.** "You agree that you will not tamper with the execution of Steam or Content and Services unless otherwise authorized by Valve", and you will not "disable, circumvent, or otherwise interfere with" anti-cheat software[^ssa].
- **Section 4C.** "You may not use any form of scripts, bots, macros, or other non-human-controlled systems ('Automation') to interact with Content and Services on Steam in any manner, including but not limited to" automating account creation, "Faking gameplay statistics (e.g., inflated wins or losses, XP, playtime)", "Earning rewards or progress without genuine user input", and automated participation in adjudication systems[^ssa].
- **Section 2.** It bars reverse engineering and modifying software accessed through Steam, except as the agreement, including a game's own terms, allows[^ssa]. A game's EULA counts as "Subscription Terms" under the agreement[^ssa].

### The BI Community Wiki's terms

The wiki's copyright page redirects to its general disclaimer[^wikiterms]. Use of its content "may only be for your personal, non-commercial entertainment use only". Content from it may be integrated into derived works "as long as your content and any derived or included works remain licensed non-commercial"[^wikiterms].

The wiki's Extensions page is Bohemia's documentation of the extension interface[^wikiext]:
- the functions `RVExtension`, `RVExtensionArgs`, `RVExtensionVersion`, `RVExtensionRegisterCallback` and `RVExtensionContext`;
- the history of each since Arma 2 1.61;
- where the game looks for extension files;
- that since 1.46 "client extensions need to be whitelisted in order to get loaded" under BattlEye.

### CBA_A3

The repository's `LICENSE.md` is GPL-2.0[^cba]. Each addon folder has its own `license.txt`: GPL-2.0 for all of them except `keybinding` and `settings`, which are MIT (copyright 2014 CBA Project, Ryan Schultz)[^cba].

The README says: "Any addon which calls CBA-defined functions need not be licensed under the GPLv2 or released under a free software license. Only if it is directly including CBA code in the addon's binarized PBO or redistributing a modified version of CBA itself would it be considered derivative"[^cba].

### ACE3

ACE3's `LICENSE` is GPL-2.0 "or (at your option) any later version", plus three things[^ace]:
- an extra grant to redistribute unmodified `.pbo` combinations without sharing source;
- a rule that a derivative may not use a name that suggests an official release;
- a note that folders with their own LICENSE follow it.

`addons/apl` is under the Arma Public License. Some sound and data folders carry their own `LICENSE.md`[^ace].

## (a) A tool that drives the game and loads an extension DLL

- **The extension interface is Bohemia's own documented feature.** The wiki documents `callExtension`, the entry points, callbacks (1.96), context (2.12), feature flags (2.18) and where the game loads extension files from[^wikiext]. Loading an extension through it does not modify, decompile or reverse-engineer the program. The Content Usage Rules allow "your own ... tools, plug-ins and other utilities" that are your original creation[^gcur]. A framework that talks to the game only through documented commands, `callExtension`, startup parameters and server configuration stays on that side of the line.
- **Patching the engine is on the other side.** These run into the EULA's "reverse-engineer, modify, disassemble" clause[^eula], the rules' "change the game's functionality" and "hack or modify the dedicated server"[^gcur], and Steam's "tamper with the execution"[^ssa]:
  - pattern scanning and hooking engine internals, as ArmaDebugEngine and Intercept's loader do[^tools];
  - hooking the Direct3D Present call to grab frames, as Neefay's extension does[^neefay].

  Whether a read-only Present hook "modifies the Program" is **owner's judgment**. The framework does not need one for the MVP.
- **BattlEye off is required, not a workaround.** A non-whitelisted extension is blocked under BattlEye[^wikiext]. Running local test sessions with BattlEye off is a server and launcher setting. The framework must never try to load its DLL under BattlEye, and never connect to a BattlEye-protected server. Doing either would come close to Steam's ban on circumventing anti-cheat[^ssa].
- **Purpose: entertainment, or developing mods?** The game EULA allows "entertainment and recreation purposes" and excludes "occupational or vocational purposes"[^eula]. Bohemia's own tools are licensed for "designing, developing, testing, and producing non-commercial game content"[^toolseula], which shows that Bohemia treats mod testing as an accepted use of its products. Testing a free mod for the community fits that.
  - Using the framework for paid work, such as a commissioned mod, or under a company's name, would rest on the "commercial purpose" and "occupational" exclusions[^eula]. That is **owner's judgment**, and a question to put to Bohemia before any commercial use.
- **Steam's automation clause is broad.** Section 4C bars "any form of scripts, bots ... to interact with Content and Services on Steam in any manner", and its examples are about faking statistics, playtime and rewards[^ssa]. A test runner is a script that drives a Steam game. Two readings are possible:
  - Mod scripting and dedicated servers are features the game's own terms and documentation provide, which section 2 lets a game's terms authorize[^ssa][^wikiext][^gcur].
  - Taken literally, the clause covers automation of any kind.

  The **owner's judgment** decides which reading holds. Either way, the framework should keep test runs local, LAN-only and off the public server list. It should not chase achievements or stats, and should run with the fewest client hours a check needs. These measures reduce what the examples in 4C name, but do not settle the clause.

What this forces:
- The framework uses only documented interfaces: SQF, `callExtension` with callbacks and context, startup parameters, server config. It does not pattern-scan, hook or patch the engine. Touches CR-10, CR-15, CR-25, CR-27.
- Refuse to start with BattlEye on, and refuse a non-loopback server. This makes CR-61 a guard of the license position, not only of safety. Touches CR-61.
- The spec records the purpose position: non-commercial mod testing, local runs only. Commercial use waits for the owner to clear it with Bohemia. Touches the PRD's section 7 constraint.

## (b) Several copies on one machine

- **One copy, one computer.** The EULA grants use of "one (1) copy" and forbids use "on more than one computer at a time"[^eula]. Several game processes from one installation, on one computer, under one Steam account, are not plainly forbidden. The text speaks of computers and networking, not processes, but neither does it expressly allow them. This is **owner's judgment**. Section 2 of the SSA marks a game's own terms as the place where such permission sits[^ssa].
- **Dedicated servers are explicitly allowed.** "Can I host an Arma dedicated server? Yes"[^gcur]. A local dedicated server with headless clients is the documented setup. Headless clients are server binaries started with `-client`, as T2's tools do[^neefay].
- **Copying the executable.** Neefay copies `arma3server_x64.exe` and the Steam DLLs into a separate root, so its server reads its own `MPMissions`[^neefay]. The EULA forbids "copy, duplicate, reproduce" without consent[^eula]. A second copy of the executable on disk is a copy of the Program in plain words. Avoiding it is easy: run the installed binaries in place, with `-profiles` and a mission path per run[^startup]. That keeps the question from arising. Whether a junctioned or copied root is a breach is **owner's judgment**. The framework does not need one.
- **Network.** The EULA forbids networking the Program "except if expressly authorized otherwise in the applicable documentation"[^eula]. Multiplayer and dedicated servers are documented features[^gcur]. Running them on the loopback interface stays inside that.

What this forces:
- The runner starts every process from the Steam installation, never from a copy.
- It runs one session at a time on the test machine, which already matches the "one game at a time" working rule.
- Each process gets its own profile folder and `-name`[^startup].

Touches CR-01, CR-04, CR-41. Whether several rendering clients under one Steam account can join one local server is a technical question, **needs a spike** in T5 (pass condition: two clients join and stay 10 minutes).

## (c) Reusing code in a project that may be open-sourced later

| Source | License, from its file | Can its code go into this project and stay open-sourceable? |
|---|---|---|
| CBA_A3, most addons | GPL-2.0[^cba] | Only if the whole project becomes GPL-2.0. Calling CBA functions at runtime is not derivative, by CBA's own statement[^cba]. Load CBA and call it, do not copy it. |
| CBA `keybinding`, `settings` | MIT[^cba] | Yes, with the copyright notice kept. |
| ACE3 | GPL-2.0-or-later, plus a pbo exception and a naming rule[^ace] | Same as CBA: the whole project would become GPL. Calling ACE functions and loading ACE at runtime is fine. |
| ACE3 `addons/apl` and other APL content | APL[^ace][^apl] | No. NonCommercial and ArmaOnly are not open-source terms. |
| Any APL, APL-SA or APL-ND material | NonCommercial, ArmaOnly; SA requires the same license; ND forbids adaptation[^apl] | No. |
| BI wiki text and examples, including extension skeleton code on the Extensions page | Non-commercial; derived works must stay licensed non-commercial[^wikiterms] | No. Write extension code from the documented function signatures, or from MIT sources, not by pasting wiki examples. Link to wiki pages instead of copying their text. |
| `lepitinox/a3-bridge`, `Zihuange/arma3-mcp`, `a3ref/mcp`, Intercept, Pythia, ArmaTesqf, `vscode-sqf-debugger`, IntelPackage | MIT[^tools] | Yes, with the notice kept. |
| `ScottTMConnors/arma3-wiki-mcp`, `Nighthawk42/arma3-mcp` | MIT for code only; their wiki data is non-commercial[^tools] | Code yes, data no. |
| arma-rs | Repository LICENSE GPL-2.0; crates declare MIT, and crates.io lists MIT[^armars] | As a dependency from crates.io under its declared MIT, probably yes. The conflict is for the owner to settle, ideally by asking the author. |
| HEMTT | GPL-2.0[^tools] | Run it as a separate tool, do not copy its code. |
| `realbadidas/Arma-Remote-Code-Executor` | GPL-3.0[^tools] | Only if the whole project became GPL-3.0. |
| ArmaDebugEngine | Own license: ArmaOnly, NonCommercial, NonMonetized; no sharing or adaptation without written permission[^ade] | No. |
| `Neefay/arma3-llm-harness`, `headswe/arma3-mcp`, `JTM-rootstorm/arma-mcp`, `g0mb4/arma-ws` | No LICENSE file[^tools] | No. All rights reserved by default. Ideas only. `arma-ws`'s README grant ("Use, modify at your will, just please mention me") is informal, and its weight is **owner's judgment**. |

Other points for (c):
- The project's own license, when it is open-sourced, is the owner's choice. Bohemia's FAQ says a mod made entirely by its author may use any license[^licfaq], and the rules allow original tools for commercial and non-commercial use[^gcur].
- The project's name and any logo must not suggest it is Bohemia's or official, and must not use Bohemia's logos[^gcur]. A derivative of ACE may not suggest an official ACE release[^ace]. Whether "arma" in the repository name is acceptable is **owner's judgment**.

What this forces:
- Every piece of code taken from elsewhere is recorded with its source, commit and license, as CLAUDE.md requires. Only MIT and other permissive code is taken.
- The fixture mod and the framework mod contain no APL content and no wiki text.

Touches the PRD's section 7 constraint, and every component.

## Privacy of logs and results

- **Steam IDs reach the extension on every call.** Since 2.12, the engine passes the extension a context whose first value is "Steam 64bit userID of the local machine, same as getPlayerUID", before every call unless the extension sets `RVFeature_ContextNoDefaultCall`[^wikiext]. arma-rs exposes this as the call context's caller[^armars].
- **Scripts can read Steam IDs too.** `getPlayerUID` returns the player's steamID64 in multiplayer[^uid]. Tools in T2 return names and UIDs as data: `headswe`'s `list_players` returns "Name, UID, owner id"[^headswe].
- **The profile name is the account's unless set.** Player names come from the profile, which `-name=` sets per process[^startup]. The test machine's Steam account name may appear wherever the framework does not set one. Whether the RPT and server logs of a local session write player names and Steam IDs is **needs a spike**: pass when a grep of every RPT and server log of a two-client session for the account's steamID64 and Steam name finds none, after the framework's settings are applied.
- **This is personal data.** Under the GDPR, personal data is information about an identifiable person, by reference to "an identifier such as a name, an identification number ... an online identifier"[^gdpr]. A steamID64 and a Steam name are that.
  - Processing "by a natural person in the course of a purely personal or household activity" is outside the GDPR[^gdpr]. The owner testing on their own machine likely is.
  - Committing result files or logs to a repository that is later made public publishes them, and that is no longer purely personal. This is **owner's judgment**, but avoiding it costs little.

What this forces:
- Every process gets a neutral `-name` (such as `client1`) and its own `-profiles` folder.
- The result file and the stored logs (CR-24, CR-51) replace any steamID64 or account name with a placeholder before they are written next to the run.
- The extension sets `RVFeature_ContextNoDefaultCall` unless it needs the context[^wikiext].
- Run folders are not committed by default.

Touches CR-24, CR-51, CR-53. An agent tool that returns players must return the framework's names, not Steam identities.

## Findings and what they force

1. **Use only documented interfaces.** The extension interface is documented by Bohemia[^wikiext], while patching or hooking the engine meets the EULA's, the rules' and Steam's bans on reverse engineering and tampering[^eula][^gcur][^ssa]. This forces a design with no engine hooks or memory patterns, and no Present hook. Touches CR-10, CR-25, CR-27.
2. **The licensed purpose is non-commercial mod testing.** The game EULA excludes occupational and commercial use[^eula]. Bohemia's tools license names testing non-commercial content as its purpose[^toolseula]. This forces a stated position in the spec, and a question to Bohemia before any commercial use. Touches the PRD's section 7.
3. **Run in place, one session at a time, local only.** No copy of the executables[^eula]. Loopback and LAN only, per the documented dedicated-server use[^gcur]. Never with BattlEye, never against public servers[^wikiext][^ssa]. Touches CR-01, CR-41, CR-61.
4. **Load CBA and ACE, never copy them.** Their GPL code would force the GPL on the project, while calling them does not[^cba][^ace]. Reuse comes only from MIT sources, with notices. Code without a license file, APL material and wiki text are excluded[^tools][^apl][^wikiterms]. Touches every component.
5. **Steam identities stay out of results.** The extension context and `getPlayerUID` carry steamID64s[^wikiext][^uid], which are personal data[^gdpr]. This forces neutral `-name`s, redaction in the result file and stored logs, and run folders kept out of git. Touches CR-24, CR-51, CR-53.

## Needs the owner's judgment

- Whether mod testing with automated scripts fits the game EULA's "entertainment and recreation" purpose, and Steam's automation clause[^eula][^ssa].
- Whether several game processes from one installation count as "one (1) copy"[^eula].
- Whether a read-only Present hook, which T2's Neefay uses for screenshots, "modifies the Program"[^eula][^neefay].
- How to treat arma-rs's GPL-2.0 LICENSE file against its MIT crates[^armars].
- Whether `arma-ws`'s README grant counts as a license.
- The project's name, before it is open-sourced[^gcur].
- Committing logs that may hold Steam identities[^gdpr].

## Could not verify

- Whether the Arma 3 Server, a separate Steam tool, has its own EULA. Bohemia's license page lists none, and the rules cover dedicated servers under the game[^gcur]. **Unverified.**
- What the RPT and server logs of a local session write about players. **Needs a spike**, as above.

[^eula]: https://www.bohemia.net/en/licenses/arma3-end-user-license, updated 20/08/2026.
[^toolseula]: https://www.bohemia.net/en/licenses/bohemia-interactives-tools-end-user-license, updated 20/08/2026.
[^gcur]: https://www.bohemia.net/en/community/game-content-usage-rules, updated 21/08/2026.
[^apl]: https://www.bohemia.net/en/licenses/arma-public-license, /arma-public-license-share-alike and /arma-public-license-nd.
[^licfaq]: https://www.bohemia.net/en/licenses/faq.
[^ssa]: https://store.steampowered.com/subscriber_agreement/, last updated September 10, 2026, sections 2 and 4.
[^wikiterms]: https://community.bistudio.com/wiki/Meta:General_disclaimer, revision of 2023-05-25.
[^wikiext]: https://community.bistudio.com/wiki/Extensions, revision 379582.
[^uid]: https://community.bistudio.com/wiki/getPlayerUID, revision 353295.
[^startup]: https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters, revision 378577.
[^gdpr]: https://gdpr-info.eu/art-4-gdpr/ (Article 4(1)) and https://gdpr-info.eu/art-2-gdpr/ (Article 2(2)(c)).
[^cba]: https://github.com/CBATeam/CBA_A3 at 8b15a04b, LICENSE.md, README.md, addons/*/license.txt.
[^ace]: https://github.com/acemod/ACE3 at 45c25247, LICENSE, addons/apl/LICENSE.
[^tools]: research/11-arma-tools.md, the T2 survey of repositories and LICENSE files.
[^neefay]: https://github.com/Neefay/arma3-llm-harness at 21aac67e, docs/ARCHITECTURE.md, host/src/arma_harness/serverroot.py.
[^headswe]: https://github.com/headswe/arma3-mcp at d778d75e, README.md.
[^armars]: https://github.com/BrettMayson/arma-rs at 8e7b243c, LICENSE and Cargo.toml; https://crates.io/api/v1/crates/arma-rs.
[^ade]: https://github.com/dedmen/ArmaDebugEngine at e01f80e4, LICENSE.
