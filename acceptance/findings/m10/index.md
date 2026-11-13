---
type: Index
title: M10 findings
description: The findings of M10 The grappling-hook suite.
---

# M10 findings

* [A relative `mods.<name>.path` with `..` makes the game exit at start](01-a-relative-mod-path-with-dot-dot-makes-the-game-exit.md) - With the project in acceptance\shakedown\ and mods.agh.path = '..\..\.hemttout\build', session start ended exit 4 because the game refused the joined path with .. in -mod, though the docs say a relative path is relative to the project folder.
* [A cleanup runs on the lead machine only, so on a dedicated session it cannot undo what a check did on a client; the docs tell a check to use one for exactly that](02-a-cleanup-cannot-undo-a-clients-state-on-a-dedicated-session.md) - A cleanup runs on the lead machine only, which is the server on a dedicated session, so it cannot undo an action, allowDamage, an event handler or a variable a check set on a client, though the docs tell a check to use one for exactly that.
* [`skd_fnc_pressKey` does not return what the keybind's code returned, so a check cannot assert that a key passes through to the game](03-press-key-does-not-return-what-the-keybinds-code-returned.md) - skd_fnc_pressKey returns true once the keybind's code ran, so a check cannot assert through it that a key passes through to the game, as arma-grappling-hook's CTL-01 promises off a rope.
* [No way to share helper code between checks, and the porting page does not say what to do with an old suite's shared library](04-no-way-to-share-helper-code-between-checks.md) - With no #include, no preprocessor and no file a check can load, the port repeats a 200-line helper block in eight check files, and the porting page does not say what to do with an old suite's shared library.
* [The docs do not say how an object in `args` is looked up on a machine that has not got it in place yet; inside `skd_fnc_on` a just-created object never came right within 10 s](05-the-docs-do-not-say-when-an-object-in-args-is-looked-up.md) - skd_fnc_on sent at once after skd_fnc_create on a dedicated session gave code on p1 an object that never came right within 10 s, and the docs do not say which functions look an object in args up once and which at each evaluation.
