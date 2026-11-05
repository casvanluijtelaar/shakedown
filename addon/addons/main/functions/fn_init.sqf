/*
    The in-game agent's start, at every mission start (postInit).

    Registers the handlers for orders and script errors, then tells the runner this machine is
    ready (P-02 step 7). Outside a Shakedown session the extension has no runner, and the agent
    does nothing.
*/
private _status = "shakedown" callExtension ["status", []];
private _fields = parseSimpleArray (_status select 0);
if ((_fields select 0) isEqualTo "refused") exitWith {
    diag_log text "[shakedown] no runner in this game's environment; the agent stays idle";
};
skd_machine = _fields select 2;

private _starts = (uiNamespace getVariable ["skd_missionStarts", 0]) + 1;
uiNamespace setVariable ["skd_missionStarts", _starts];

// Compile errors are only reported through ScriptError, so the agent records the errors raised
// while it compiles code it was sent.
skd_compiling = false;
skd_compileErrors = [];
addMissionEventHandler ["ScriptError", {
    params ["_text", "_file", "_line"];
    if (skd_compiling) then {
        skd_compileErrors pushBack [_text, _line];
    };
}];

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_function", "_data"];
    if (_name isEqualTo "shakedown" && {_function isEqualTo "order"}) then {
        [_data] spawn skd_fnc_receive;
    };
}];

private _pv = productVersion;
[createHashMapFromArray [
    ["kind", "ready"],
    ["machine", skd_machine],
    ["mission_start", _starts],
    ["game_version", format ["%1.%2 %3", ((_pv select 2) / 100) toFixed 2, _pv select 3, toLower (_pv select 4)]],
    ["player", !isNull player]
]] call skd_fnc_send;
