// The fixture's CBA keybind (spec/contract/fixture-mod.md, "A CBA keybind"; ACT-02, ACT-03), on
// every machine with an interface: mod skd_fixture, action signal, F7 (DIK 65) with no modifier.
// Down adds 1 to skdfx_signalDown and up adds 1 to skdfx_signalUp on the player's unit, both
// public; down also raises the CBA global event skdfx_signalled with [unit].
[
    "skd_fixture", "signal", "Fixture signal",
    {
        private _unit = player;
        _unit setVariable ["skdfx_signalDown", (_unit getVariable ["skdfx_signalDown", 0]) + 1, true];
        ["skdfx_signalled", [_unit]] call CBA_fnc_globalEvent;
        false
    },
    {
        private _unit = player;
        _unit setVariable ["skdfx_signalUp", (_unit getVariable ["skdfx_signalUp", 0]) + 1, true];
        false
    },
    [65, [false, false, false]], false
] call CBA_fnc_addKeybind;
