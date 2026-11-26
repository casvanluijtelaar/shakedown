---
type: Research
title: "Starting a player-hosted session with nobody at the keyboard"
description: "What Bohemia's pages and the known tools say about starting a non-dedicated host and its mission with no person at the machine: -host with a server config and -autoInit, the framework's addon pressing the multiplayer displays' buttons with ctrlActivate, hostMission, Eden's multiplayer preview and -init, and what headswe/arma3-mcp does; none is documented to work unattended, so each way is a candidate for spike SPK-24, with what it forces on the framework's machine kinds, its start sequence, and the extension and token on the host."
status: draft
generated: { by: "claude-code/developer", at: 2026-11-26T18:47:10Z }
sources:
  - { id: startup, resource: "https://community.bistudio.com/wiki/Arma_3:_Startup_Parameters (oldid 373124, Wayback snapshot 2025-04-09; the live wiki answered every request with a Cloudflare challenge on 2026-11-26, and a search snippet of the live page dated 2026-05-10 shows the same host and autoInit text), entries host, setUpHost, server, port, config, autoInit, init, command and <path>\\mission.sqm", read: 2026-11-26 }
  - { id: servercfg, resource: "https://community.bistudio.com/wiki/Arma_3:_Server_Config_File (oldid 370737, Wayback snapshot 2025-04-09), the introduction, allowedFilePatching, persistent, class Missions and Mission Rotation", read: 2026-11-26 }
  - { id: hostmission, resource: "https://community.bistudio.com/wiki/hostMission (oldid 369139, Wayback snapshot 2025-04-19)", read: 2026-11-26 }
  - { id: playmission, resource: "https://community.bistudio.com/wiki/playMission (oldid 348509, Wayback snapshot 2023-12-19)", read: 2026-11-26 }
  - { id: ctrlactivate, resource: "https://community.bistudio.com/wiki/ctrlActivate (oldid 369263, Wayback snapshot 2025-03-25)", read: 2026-11-26 }
  - { id: uicontext, resource: "https://community.bistudio.com/wiki/isUIContext (oldid 310464, Wayback snapshot 2025-02-14)", read: 2026-11-26 }
  - { id: servercommand, resource: "https://community.bistudio.com/wiki/serverCommand (oldid 370059, Wayback snapshot 2025-04-05)", read: 2026-11-26 }
  - { id: srvcmds, resource: "https://community.bistudio.com/wiki/Multiplayer_Server_Commands (oldid 370389, Wayback snapshot 2025-05-18), #login, #mission, #restart", read: 2026-11-26 }
  - { id: descext, resource: "https://community.bistudio.com/wiki/Description.ext (oldid 373282, Wayback snapshot 2025-05-10), skipLobby and hostDoesNotSkipLobby", read: 2026-11-26 }
  - { id: idd, resource: "https://community.bistudio.com/wiki/Arma_3:_IDD_List (oldid 347325, Wayback snapshot 2025-02-10), IDD_MULTIPLAYER 8, IDD_SERVER 17, IDD_SERVER_SETUP 20, IDD_SERVER_GET_READY 52, IDD_HOST_SETTINGS 132", read: 2026-11-26 }
  - { id: pipe, resource: "https://community.bistudio.com/wiki/Arma_3:_Named_Pipe (oldid 369901, Wayback snapshot 2025-03-16)", read: 2026-11-26 }
  - { id: funclib, resource: "https://community.bistudio.com/wiki/Arma_3:_Functions_Library (oldid 369667, Wayback snapshot 2025-03-19), the preStart attribute", read: 2026-11-26 }
  - { id: hc, resource: "https://community.bistudio.com/wiki/Arma_3:_Headless_Client (oldid 352522, Wayback snapshot 2025-03-26)", read: 2026-11-26 }
  - { id: edenactions, resource: "https://community.bistudio.com/wiki/Eden_Editor:_Actions (oldid 364681, Wayback snapshot 2025-03-22), MissionPreviewMP; https://community.bistudio.com/wiki/do3DENAction (oldid 369620, Wayback snapshot 2025-02-17)", read: 2026-11-26 }
  - { id: uif, resource: "https://github.com/Javaec/Arma3sqf a3/ui_f/scripts/GUI/RscDisplayHostSettings.sqf, RscDisplayMultiplayerSetup.sqf and RscDisplayMultiplayer.sqf (a copy of the game's UI scripts last pushed 2015-03-10, no license file; read for what the scripts do, nothing taken)", read: 2026-11-26 }
  - { id: headswe, resource: "https://github.com/headswe/arma3-mcp at d778d75e87bf7590793297e1afb63558c8b88f14 (the latest commit on 2026-11-26), README.md, addons/main/functions/fn_init.sqf and the file tree (no license file per the GitHub API)", read: 2026-11-26 }
  - { id: search, resource: "Web searches on 2026-11-26 for an unattended -host start, -host with -config and autoInit, and scripts that host a mission from the main menu; GitHub code searches for hostMission, RscDisplayHostSettings and ctrlActivate on the multiplayer displays", read: 2026-11-26 }
  - { id: r11, resource: "engineering/research/11-arma-tools.md, the headswe/arma3-mcp section and its table row", read: 2026-11-26 }
  - { id: r14, resource: "engineering/research/14-sessions.md, section 1 and implication 8", read: 2026-11-26 }
  - { id: r32, resource: "engineering/research/32-spike-connect-times.md", read: 2026-11-26 }
  - { id: r33, resource: "engineering/research/33-client-to-another-server.md", read: 2026-11-26 }
  - { id: r35, resource: "engineering/research/35-spikes-ace-and-keys.md, SPK-11", read: 2026-11-26 }
  - { id: r37, resource: "engineering/research/37-launching-clients-without-focus.md, section 4", read: 2026-11-26 }
  - { id: spec, resource: "engineering/spec/04-integrations.md (INT-AR-02 to INT-AR-05), engineering/spec/03-functional-requirements.md (SES-03, SES-04, SEC-02, SEC-05), engineering/spec/01-domain-model.md (DM-10 to DM-12)", read: 2026-11-26 }
---

# Starting a player-hosted session with nobody at the keyboard

Gap H9 keeps the player-hosted session out of the framework until a spike shows an unattended way to start one. A player-hosted session is a game client, `arma3_x64.exe`, that is also the server: it has a player and a window, and other clients join it. Research 14 found the start option `-host` and no way to load the host's mission with nobody in the lobby[^r14]. This research asks what starts such a host and its mission with no person at the machine, and what each way forces on the framework. No game was started for it: this is a documents round. What it says about the game rests on Bohemia's pages, read through the Wayback Machine because the live wiki answered every request with a Cloudflare challenge, and on the repository's own records. The web and GitHub searches found no report of anyone starting a hosted session unattended[^search], so every way below is a candidate for the spike, not a known path.

## 1. What a hosted session needs that a dedicated one gets from server.cfg

A dedicated session gets its mission, its safety and its machines from the `server.cfg` the runner writes (`INT-AR-05`)[^spec]. A host needs the same things from somewhere:

- **The mission.** `class Missions` with `persistent = 1` and `-autoInit` start the mission with nobody connected. `-autoInit` is skipped without `persistent = 1`[^startup][^servercfg].
- **Headless clients.** A server takes a headless client only from an address in `headlessClients[]` of its server config[^hc].
- **File patching on clients.** `allowedFilePatching` sets whether clients that load unpacked data may join: 0 none, 1 headless clients only, 2 all clients. The default is 0[^servercfg]. Every game client of a session starts with `-filePatching` (`SES-09`)[^spec].
- **Safety.** `loopback = 1`, a password and `BattlEye = 0` keep the session's server off the network and other people out (`SEC-05`)[^spec]. A host that reads no server config takes its password and visibility from its host settings display, `IDD_HOST_SETTINGS` (132)[^idd]. Whether that display can make a host loopback-only is not documented.

So the first question for every way is whether the host reads a server config at all. Without one, a hosted session has no headless clients and no file-patching clients, and is not loopback-only unless some other setting does it.

## 2. The ways

### 2.1 `-host` with `-config` and `-autoInit`

**What the sources say.** The wiki's whole entry for `-host` is "Start a non-dedicated multiplayer host", with the example `arma3_x64.exe -host`. It sits under "Client Network Options", beside `-connect`, `-port` and `-password`. `-config` and `-autoInit` sit under "Server Options", and every example of them is `arma3server_x64.exe`[^startup]. The server config page says the file's name "is determined by the -config command line option when launching the dedicated server"[^servercfg]. No page says whether `arma3_x64.exe -host` reads `-config`, takes `-port` as its listening port, or honours `-autoInit`. The searches found no report either way[^search]. `-setUpHost` "Start[s] the game in the setup server display", and `-server` starts a dedicated server from the client executable, which has no player and is not a hosted session[^startup].

**What it would force.**

- *Machine kinds.* A new kind: a game client that is the server. Everything else is as on a dedicated session.
- *Start sequence.* The same as a dedicated one (`P-01`): generate the session mission and `server.cfg`, copy the mission to `<game>\MPMissions\`, start the host with the player options of `INT-AR-02` plus `-host -port=<base port> -config=<session folder>\server.cfg -autoInit`, wait for it to be `ready`, then start every client at once.
- *Extension and token.* Unchanged. The runner starts the host's `arma3_x64.exe`, so it gets the four environment variables of `SEC-02`, and the in-game agent connects in the session mission's `postInit`.

Nothing new goes into the addon, and nothing runs before a mission. That makes it the cheapest way to try first. If the host ignores `-config`, this way fails at once. If it reads `-config` but ignores `-autoInit`, it sits in a lobby or a mission list, and 2.2 can take it from there.

### 2.2 The framework's addon pressing the multiplayer displays' buttons

**What the sources say.**

- `ctrlActivate control` "launches action attached to given (button based) control" and "ignores control disabled state". It "only triggers engine-based actions (cancel or ok codes)". Since 2.18, `control ctrlActivate true` simulates the click in full, with its UI event handlers. The page adds: "Use isUIContext to distinguish between scripted activation and user activation"[^ctrlactivate].
- The multiplayer displays have fixed IDDs: `IDD_MULTIPLAYER` 8, `IDD_SERVER` 17, `IDD_SERVER_SETUP` 20, `IDD_SERVER_GET_READY` 52, `IDD_HOST_SETTINGS` 132[^idd]. Their scripts in `ui_f` only set the player's name, upper-case the texts and move the focus. Hosting itself is the engine's, behind the OK buttons[^uif].
- An addon's function with `preStart = 1` is called "upon game start, before title screen, but after all addons are loaded"[^funclib]. That is how SQF of the framework's addon can run in the main menu, before any mission.
- A host skips the lobby when the mission has `skipLobby = 1`, unless it also sets `hostDoesNotSkipLobby = 1` (2.06)[^descext]. The session mission has `skipLobby = 1` and `briefing = 0` (`INT-AR-04`)[^spec]. If the host honours both, the host gets its role and goes into the mission once the mission is chosen, with no role display to press through **(inference)**.

**What the repository's own trial says.** In M16, `ctrlActivate` on the Abort button of a dialog the code opened itself did nothing in 60 s on a client whose window did not render, because no one was connected to the desktop[^r33]. Scripts in `uiNamespace` kept running, and display 46 was null. Whether display buttons act in a window that renders is not known. Research 37 has no other record of UI code without a rendered window[^r37]. The spike runs with the desktop shown and nobody giving input, which is how the framework already runs dedicated clients.

**What it would force.**

- *Machine kinds.* As in 2.1.
- *Start sequence.* The host starts with `-host` or `-setUpHost`. A `preStart` function in the framework's addon spawns a loop in `uiNamespace`. It must stay inert unless the runner started the game: it acts only when `"shakedown" callExtension ["status", []]` returns a session id and the machine name of the host. That call already exists (`INT-AR-03`). The loop chooses the mission whose name starts with `shakedown_<session id>.` from the mission list of the display hosting opens, then presses that display's OK. It stops at its own ceiling and says what it saw, so the runner can name the display a failed start stopped at. Then the runner starts the clients, as on a dedicated session.
- *Extension and token.* The token comes from the environment as `SEC-02` says. Calling `status` in the main menu makes the host's extension connect before any mission, so its `connected` comes before its mission loads, where every other kind connects in `postInit`[^r32]. `SES-10`'s reason would read that state differently for a host.
- *Safety.* This is code in the framework's addon that drives the game's own menus in every game that loads `@shakedown`. The guard on `status` is all that keeps it from hosting a session in a person's own game. It sends no input outside the game, so it stays inside the rule on the desktop.

### 2.3 `hostMission` from the addon

**What the sources say.** `hostMission [configEntry, activeDisplay]` will "host the multiplayer mission defined in the config class and opens the lobby with the scenario loaded. Must be called in UI action from a dialog." Its examples take `configFile >> "CfgMissions" >> "MPMissions" >> <class>`. A note from 2011 says it "must be called in the main menu or something similar"[^hostmission]. `ctrlActivate`'s own page tells scripted activation from user activation by `isUIContext`[^ctrlactivate]. `isUIContext` is true only in code started by a UI control event, and false again in code it spawns[^uicontext]. `serverCommand` on a client has the same kind of rule, "must be executed from UI context"[^servercommand].

**What it would force.**

- *The mission as a config class.* The session mission is a folder the runner writes per session (`INT-AR-04`)[^spec]. `hostMission` takes a config entry. The framework's addon would need one fixed `CfgMissions >> MPMissions` class whose mission the runner supplies per session. Whether such a class can name a folder the runner writes, through file patching, is not documented. The server config page allows a mission folder in `MPMissions` only through `class Missions`[^servercfg].
- *A UI action.* The call would have to run in a UI event of a control on the main menu. If the engine checks `isUIContext`, a button pressed by `ctrlActivate`, even in full with 2.18, is scripted activation and may be refused **(inference)**. Nothing a script can do is documented to make a UI context.
- *Start sequence, extension and token.* As in 2.2, with the guard on `status`. After `hostMission` the lobby opens. With `skipLobby = 1` the host may pass it, or 2.2 presses its OK.

### 2.4 Eden's multiplayer preview

**What the sources say.** A `<path>\mission.sqm` on the command line loads that mission into the editor[^startup]. `do3DENAction "MissionPreviewMP"` previews the open scenario "in multiplayer"[^edenactions]. headswe's addon runs its code from `Cfg3DEN` event handlers when the editor opens or loads a scenario[^headswe], so an addon can act in the editor without a mission.

**What it would force.** The session mission would go under the host's profile folder, as in singleplayer, and the host would start with its `mission.sqm`. A `Cfg3DEN` handler in the framework's addon, behind the same guard, would call `MissionPreviewMP`. Whether the preview opens the host settings display and the lobby, and so needs 2.2's buttons as well, is not documented. A preview ends back in the editor, so a mission restart and a reload would run through the editor. Every other way runs them through the multiplayer screens. It puts Eden in every hosted session.

### 2.5 `-init`

`-init=<command>` runs "a scripting command once in the main menu", for a singleplayer mission in the game's `Missions` folder with `playMission`[^startup][^playmission]. It could carry the first step of 2.2 or 2.3 on the host's command line. Only the runner's start would have it, so it would need no guard in the addon. It is not a UI action **(inference)**, so it does not answer 2.3's question. A command line is visible to every process of the user, so it may carry the mission's name but never the token (`SEC-02`).

### 2.6 How headswe/arma3-mcp gets a hosted session

Research 11 lists it as working on hosted sessions[^r11]. It starts none. Its README says the bridge "works while a mission runs (editor preview, singleplayer, a hosted game or a dedicated server)". On a host it is loaded "like any local mod" with BattlEye off, and "only the main menu has nothing that answers"[^headswe]. Its addon tells its extension `isServer`, `hasInterface` and `isDedicated` at mission start, and by default only the server, or the singleplayer game, listens[^headswe]. A host is the server, so the bridge listens there once a person has hosted a mission by hand. The repository has no code that starts a game, hosts a mission or presses a menu button[^headswe]. It shows that a host runs an addon's extension and SQF like any other server, which every way above relies on, and nothing about starting one.

### 2.7 Ruled out

- **Keys or clicks from outside the game.** `CLAUDE.md` forbids input to the desktop, and key messages posted to the game's own window reached neither a key handler nor a CBA keybind (`SPK-11`)[^r35].
- **The named pipe.** `-command=<name>` "works for client and hosting client". It takes `shutdown`, `message`, `reply`, `session` and `connect`, and nothing that hosts. Its `session` answer holds `"hosting": true` on a host, with the island and the mission[^pipe]. During a start the runner could read that to see that a host has reached a mission before its agent is `ready`. It does not start one.
- **Server commands.** `#mission` and `#restart` need a logged-in admin, or `serverCommandPassword` with the alternative syntax of `serverCommand`, and a running server[^srvcmds][^servercommand]. They can change a hosted session's mission once one runs, but cannot start one from the main menu.

## 3. What every way forces

- **Machine kinds.** The host is a game client and the server at once: `isServer` and `hasInterface`. `DM-10` names `server`, `p1` to `pN` and `hcN` by role[^spec]. A host fills two of those roles. Singleplayer already lets `server` name `p1` (`DM-10`), and the same rule fits a host: the host is `server` and `p1`, and the joining clients are `p2` to `pN`. The lead machine is the host. `skd_fnc_player "server"` returns the host's player, where on a dedicated session it is `objNull`. Headless clients exist only if the host reads a server config (section 1).
- **The start sequence.** The host first, alone, up to `ready` in the session mission. Then every client at once, `-connect=127.0.0.1 -port=<the host's port> -password=<the session password>`, as `P-01` does for a dedicated session. The ports `SES-03` checks are the host's. Research 32 measured a singleplayer client connecting in a median 33 s and a dedicated client in 36.6 s[^r32], so a host and two clients should be `ready` well inside the 180 s of `SES-04` **(inference)**. Any menu-driving way (2.2 to 2.4) adds its own time.
- **The extension and the token.** On the host, as on every machine: the extension comes from `@shakedown` on `-mod`, and the four variables of `SEC-02` come from the runner's start. Only the menu-driving ways add anything: SQF of the framework's addon that runs before the agent, behind the guard on `status`.
- **The mission.** `INT-AR-04`'s session mission holds a playable unit for each `pN` and a `HeadlessClient_F` for each `hcN`. On a host `p1`'s unit is the host's. `maxPlayers` and the `Header` hold as they are.

## 4. What the research cannot settle

1. Whether `arma3_x64.exe -host` reads `-config`, listens on `-port` and honours `-autoInit` with `persistent = 1`. Only a game run settles it.
2. Which display `-host` and `-setUpHost` open, and whether `ctrlActivate` on its OK buttons hosts the mission while the window renders and nobody gives input.
3. Whether `hostMission` refuses a call made from a scripted button press, and whether a fixed `CfgMissions` class can name a mission folder the runner writes.
4. Whether a host that reads no server config can be made loopback-only with a password and file-patching clients. If not, a hosted session runs as a server on the network and breaks `SEC-05`.

## 5. Spike SPK-24

Each way is tried in the order of section 2, cheapest first, and the round stops at the first that passes. On a Windows machine with Arma 3 and Steam running, the desktop shown and nobody giving input, with the change built into a scratch branch: the runner starts a host and two game clients loading `@skd_fixture`.

**Pass condition.** The runner starts a host and two game clients with no person, the host loads the session mission, every machine's extension connects, a check runs on the host and on a client and passes, and every process is stopped by its process id, 10 of 10 within the start ceiling of `SES-04`.

**If it fails.** H9 stays. Each way tried, and why it failed, is written into this research. Nothing sends keys from outside the game.

[^startup]: Arma 3 Startup Parameters, oldid 373124.
[^servercfg]: Arma 3 Server Config File, oldid 370737.
[^hostmission]: hostMission, oldid 369139.
[^playmission]: playMission, oldid 348509.
[^ctrlactivate]: ctrlActivate, oldid 369263.
[^uicontext]: isUIContext, oldid 310464.
[^servercommand]: serverCommand, oldid 370059.
[^srvcmds]: Multiplayer Server Commands, oldid 370389.
[^descext]: Description.ext, oldid 373282.
[^idd]: Arma 3 IDD List, oldid 347325.
[^pipe]: Arma 3 Named Pipe, oldid 369901.
[^funclib]: Arma 3 Functions Library, oldid 369667.
[^hc]: Arma 3 Headless Client, oldid 352522.
[^edenactions]: Eden Editor Actions, oldid 364681, and do3DENAction, oldid 369620.
[^uif]: The game's UI scripts in Javaec/Arma3sqf, as of 2015.
[^headswe]: headswe/arma3-mcp at d778d75e.
[^search]: The web and GitHub code searches of 2026-11-26.
[^r11]: engineering/research/11-arma-tools.md
[^r14]: engineering/research/14-sessions.md
[^r32]: engineering/research/32-spike-connect-times.md
[^r33]: engineering/research/33-client-to-another-server.md
[^r35]: engineering/research/35-spikes-ace-and-keys.md
[^r37]: engineering/research/37-launching-clients-without-focus.md
[^spec]: engineering/spec/04-integrations.md, 03-functional-requirements.md and 01-domain-model.md
