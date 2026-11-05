// @check US-022 A misspelled keybind name is the author's to fix
// @tags US-022 acceptance

["press a keybind that is not registered", {
    ["p1", "skd_fixture", "signl"] call skd_fnc_pressKey;
}] call skd_fnc_step;

["never runs", {
    true
}] call skd_fnc_step;
