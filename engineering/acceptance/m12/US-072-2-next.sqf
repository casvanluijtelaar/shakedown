// @check US-072 the check after a recording starts with no handler of it: an event before skd_fnc_record records nothing
// @tags US-072 acceptance
// @setups ded

["a counter of our own on every machine", {
    {
        private _m = _x;
        private _id = [_m, {
            skd_m12_seen = 0;
            ["skdfx_signalled", { skd_m12_seen = skd_m12_seen + 1 }] call CBA_fnc_addEventHandler
        }] call skd_fnc_on;
        [{ ["skdfx_signalled", _this select 0] call CBA_fnc_removeEventHandler }, [_id], _m] call skd_fnc_cleanup;
    } forEach ["server", "p1", "p2"];
}] call skd_fnc_step;

["p1 signals before any recording, and every machine heard it", {
    ["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
    [["server", "p1", "p2"], { missionNamespace getVariable ["skd_m12_seen", 0] }, "==", 1, "our counter", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;

["a car killed before any recording", {
    private _car = ["C_Offroad_01_F", AGLToASL [130, 100, 0], 0] call skd_fnc_create;
    _car setDamage 1;
    [!alive _car, "==", true, "the first car is dead"] call skd_fnc_expect;
}] call skd_fnc_step;

["recording now finds nothing from before", {
    [["server", "p1", "p2"], "skdfx_signalled"] call skd_fnc_record;
    ["server", "EntityKilled", "mission"] call skd_fnc_record;
    {
        private _n = count ([_x, "skdfx_signalled"] call skd_fnc_events);
        ["signals on " + _x + " right after recording", _n] call skd_fnc_evidence;
        [_n, "==", 0, "signals on " + _x + " right after recording"] call skd_fnc_expect;
    } forEach ["server", "p1", "p2"];
    [count (["server", "EntityKilled"] call skd_fnc_events), "==", 0, "EntityKilled right after recording"] call skd_fnc_expect;
}] call skd_fnc_step;

["the next signal and kill are recorded once each", {
    ["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
    {
        [_x, "skdfx_signalled", 10] call skd_fnc_waitEvent;
    } forEach ["server", "p1", "p2"];
    [["server", "p1", "p2"], { missionNamespace getVariable ["skd_m12_seen", 0] }, "==", 2, "our counter", 10] call skd_fnc_expectEach;
    {
        [count ([_x, "skdfx_signalled"] call skd_fnc_events), "==", 1, "signals on " + _x] call skd_fnc_expect;
    } forEach ["server", "p1", "p2"];
    private _car = ["C_Offroad_01_F", AGLToASL [140, 100, 0], 0] call skd_fnc_create;
    _car setDamage 1;
    ["server", "EntityKilled", 10] call skd_fnc_waitEvent;
    [count (["server", "EntityKilled"] call skd_fnc_events), "==", 1, "EntityKilled on the server"] call skd_fnc_expect;
}] call skd_fnc_step;
