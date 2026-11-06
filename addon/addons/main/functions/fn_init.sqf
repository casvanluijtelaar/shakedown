/*
    The in-game agent's start, at every mission start (postInit).

    Registers the handlers for orders and script errors and the entity recorder, gives a
    singleplayer player the Killed handler that keeps the mission alive (RST-05), captures the
    reset baseline once the mission runs (RST-02), then tells the runner this machine is ready
    (P-02 step 7). Outside a Shakedown session the extension has no runner, and the agent does
    nothing.
*/
private _status = "shakedown" callExtension ["status", []];
private _fields = parseSimpleArray (_status select 0);
if ((_fields select 0) isEqualTo "refused") exitWith {
    diag_log text "[shakedown] no runner in this game's environment; the agent stays idle";
};
skd_machine = _fields select 2;
skd_ops = ["==", "!=", "<", "<=", ">", ">=", "~=", "in", "contains"];

private _starts = (uiNamespace getVariable ["skd_missionStarts", 0]) + 1;
uiNamespace setVariable ["skd_missionStarts", _starts];

// Every script error on this machine is sent to the runner at once (ERR-01), stamped with the run
// and check this machine was told of (skd_context, set by skd_fnc_context) and the step it ran in:
// on the lead the innermost running step of the check; elsewhere the step of the newest order still
// running here, or none, and the runner then takes the step running on the lead at that time
// (P-05 steps 6 and 7). Compile errors are only reported through ScriptError, so the agent also
// records the errors raised while it compiles code it was sent, and marks them as such.
skd_compiling = false;
skd_compileErrors = [];
skd_context = ["", ""];
skd_orderPaths = [];
// The count of script errors on this machine and the newest one's text, by which a wait sees that
// its condition raised one (OBS-01).
skd_errorCount = 0;
skd_lastError = "";
skd_execs = createHashMap;
addMissionEventHandler ["ScriptError", {
    params ["_text", "_file", "_line", "_position", "_content", "_stack"];
    if (skd_compiling) then {
        skd_compileErrors pushBack [_text, _line];
    };
    skd_errorCount = skd_errorCount + 1;
    skd_lastError = format ["%1 (%2, line %3)", _text, [_file, "no file"] select (_file isEqualTo ""), _line];
    private _path = [];
    if (!isNil "skd_check" && { !(skd_check get "done") } && { (skd_check get "id") isEqualTo (skd_context select 1) }) then {
        _path = call skd_fnc_stepPath;
    } else {
        if (count skd_orderPaths > 0) then { _path = (skd_orderPaths select -1) select 1 };
    };
    // Each frame as "file:line" and its scope; the frames' variables are left out.
    private _frames = (if (isNil "_stack") then { [] } else { _stack }) apply {
        _x params [["_f", ""], ["_l", 0], ["_scope", ""]];
        if (_scope isEqualTo "") then { format ["%1:%2", _f, _l] } else { format ["%1:%2 %3", _f, _l, _scope] }
    };
    [createHashMapFromArray [
        ["kind", "error"],
        ["machine", skd_machine],
        ["text", _text],
        ["file", _file],
        ["line", _line],
        ["stack", _frames],
        ["at", call skd_fnc_clocks],
        ["key", skd_context select 0],
        ["check_id", skd_context select 1],
        ["step_path", _path],
        // The newest order still running here, whose code the error most likely came from: an
        // exec answers with the errors stamped with its order (CTL-05).
        ["order", if (count skd_orderPaths > 0) then { (skd_orderPaths select -1) select 0 } else { "" }],
        ["compile", skd_compiling]
    ]] call skd_fnc_send;
}];

// The entity recorder (SCN-04): every entity created on this machine, whatever created it. The
// baseline empties the list, so it holds what was created since then.
skd_created = [];
skd_cleanups = [];
addMissionEventHandler ["EntityCreated", {
    params ["_entity"];
    skd_created pushBack _entity;
}];
// On a dedicated session the mission's respawn gives a dead player a new unit at once (RST-05);
// the body it leaves goes to the recorder, so the next reset deletes it.
addMissionEventHandler ["EntityRespawned", {
    params ["_new", "_old"];
    if (!isNull _old) then { skd_created pushBack _old };
}];

// Orders a machine routed to another through the runner (P-03): their answers, the polls this
// machine asked others for, and the polls it runs for another machine.
skd_routes = createHashMap;
skd_routeCount = 0;
skd_polls = createHashMap;
skd_stops = createHashMap;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_function", "_data"];
    if (_name isEqualTo "shakedown" && {_function isEqualTo "order"}) then {
        [_data] spawn skd_fnc_receive;
    };
}];

[_starts] spawn {
    params ["_starts"];
    // The baseline is the state the mission has once its first frames ran, after every mod's
    // own start, with the player standing still on the ground in a settled animation state: in the first frames the engine still
    // has the unit at height 0 ASL before it sets it on the terrain (RST-02). The wait has a
    // ceiling of 30 s of real time.
    // A dedicated server and a headless client have no player of their own to wait for.
    private _t = diag_tickTime;
    if (hasInterface) then {
        waitUntil {
            (!isNull player && { time > 0 } && { isTouchingGround player || { vehicle player isNotEqualTo player } }
                && { vectorMagnitude velocity player < 0.1 }
                // A move between two animation states is named "<from>_<to>"; the baseline waits
                // for the state it ends in.
                && { ((animationState player) find "_") < 0 }) || { diag_tickTime - _t > 30 }
        };
    } else {
        waitUntil { time > 0 || { diag_tickTime - _t > 30 } };
    };
    if (!isMultiplayer && { !isNull player }) then {
        player addEventHandler ["Killed", { _this call skd_fnc_killed }];
    };
    call skd_fnc_baseline;
    private _pv = productVersion;
    [createHashMapFromArray [
        ["kind", "ready"],
        ["machine", skd_machine],
        ["mission_start", _starts],
        ["game_version", format ["%1.%2 %3", ((_pv select 2) / 100) toFixed 2, _pv select 3, toLower (_pv select 4)]],
        ["player", hasInterface && { !isNull player }],
        ["baseline", [skd_baseline] call skd_fnc_encode]
    ]] call skd_fnc_send;
};
