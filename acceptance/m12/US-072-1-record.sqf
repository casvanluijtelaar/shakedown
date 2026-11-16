// @check US-072 p2 hears the signal p1 gives, each machine keeps it with its clocks, and the server records EntityKilled
// @tags US-072 acceptance
// @setups ded

["record on server p1 and p2", {
    [["server", "p1", "p2"], "skdfx_signalled"] call skd_fnc_record;
    ["server", "EntityKilled", "mission"] call skd_fnc_record;
}] call skd_fnc_step;

["p1 signals", {
    ["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
}] call skd_fnc_step;

private _args = ["p2 waits for the signal", {
    ["p2", "skdfx_signalled", 10] call skd_fnc_waitEvent
}] call skd_fnc_step;
["args on p2", _args] call skd_fnc_evidence;
[_args select 0, "==", ["p1"] call skd_fnc_player, "the signal names the unit of p1 on p2"] call skd_fnc_expect;

["each machine kept one signal with its clocks", {
    {
        private _m = _x;
        [_m, "skdfx_signalled", 10] call skd_fnc_waitEvent;
        private _events = [_m, "skdfx_signalled"] call skd_fnc_events;
        [count _events, "==", 1, "signals recorded on " + _m] call skd_fnc_expect;
        private _e = _events select 0;
        private _at = _e get "at";
        ["event on " + _m, [_e get "args", _at get "utc", _at get "tick", _at get "time", _at get "frame"]] call skd_fnc_evidence;
        [(_e get "args") select 0, "==", ["p1"] call skd_fnc_player, "the arguments on " + _m] call skd_fnc_expect;
        [typeName (_at get "utc"), "==", "STRING", "utc on " + _m] call skd_fnc_expect;
        { [typeName (_at get _x), "==", "SCALAR", _x + " on " + _m] call skd_fnc_expect } forEach ["tick", "time", "frame"];
    } forEach ["server", "p1", "p2"];
}] call skd_fnc_step;

["the server records EntityKilled", {
    private _car = ["C_Offroad_01_F", AGLToASL [120, 100, 0], 0] call skd_fnc_create;
    ["car", _car] call skd_fnc_evidence;
    _car setDamage 1;
    private _a = ["server", "EntityKilled", 10] call skd_fnc_waitEvent;
    ["EntityKilled args", _a] call skd_fnc_evidence;
    [_a select 0, "==", _car, "EntityKilled names the car"] call skd_fnc_expect;
    private _killed = ["server", "EntityKilled"] call skd_fnc_events;
    [count _killed, "==", 1, "EntityKilled events on the server"] call skd_fnc_expect;
    private _at = (_killed select 0) get "at";
    { [typeName (_at get _x), "==", "SCALAR", _x + " of EntityKilled"] call skd_fnc_expect } forEach ["tick", "time", "frame"];
}] call skd_fnc_step;
