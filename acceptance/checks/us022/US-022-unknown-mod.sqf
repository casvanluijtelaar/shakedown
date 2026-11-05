// @check US-022 A keybind of a mod with no keybinds names the mods that have keybinds
// @tags US-022 ACT-03 acceptance

["press a keybind of a mod that registered none", {
    ["p1", "skd_no_such_mod", "signal"] call skd_fnc_pressKey;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
