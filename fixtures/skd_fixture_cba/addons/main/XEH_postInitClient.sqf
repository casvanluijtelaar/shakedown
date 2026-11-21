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

// The fixture's ACE interaction actions (spec/contract/fixture-mod.md, "ACE interaction actions";
// ACT-04), only where ACE's interaction menu is loaded: an action on the crate that asks the
// server to open it, as the crate's own action does, and two self actions, one that counts its
// runs on the player's unit and one whose condition is always false.
if (isClass (configFile >> "CfgPatches" >> "ace_interact_menu")) then {
    private _open = ["skdfx_aceOpen", "Open crate (ACE)", "",
        { [_target, _player] remoteExec ["skdfx_fnc_open", 2] },
        { !(_target getVariable ["skdfx_open", false]) }
    ] call ace_interact_menu_fnc_createAction;
    ["skdfx_Crate", 0, ["ACE_MainActions"], _open] call ace_interact_menu_fnc_addActionToClass;
    private _signal = ["skdfx_aceSignal", "Fixture ACE signal", "",
        { _player setVariable ["skdfx_aceSignals", (_player getVariable ["skdfx_aceSignals", 0]) + 1, true] },
        { true }
    ] call ace_interact_menu_fnc_createAction;
    ["CAManBase", 1, ["ACE_SelfActions"], _signal, true] call ace_interact_menu_fnc_addActionToClass;
    private _never = ["skdfx_aceNever", "Fixture ACE never", "",
        { _player setVariable ["skdfx_aceNever", (_player getVariable ["skdfx_aceNever", 0]) + 1, true] },
        { false }
    ] call ace_interact_menu_fnc_createAction;
    ["CAManBase", 1, ["ACE_SelfActions"], _never, true] call ace_interact_menu_fnc_addActionToClass;
};
