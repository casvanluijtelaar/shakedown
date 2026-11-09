// @check US-057 skdfx_cba_fnc_version returns "1" on every machine after the session came back from a failed reload, from the start state
// @tags US-057 RLD-02 RST-02 INT-CBA-02 acceptance

["every machine returns '1' from skdfx_cba_fnc_version", {
    ["all", { call skdfx_cba_fnc_version }, "==", "1", "skdfx_cba_fnc_version", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;

["p1 and p2 stand at the start, alive, on foot and unhurt, as the server sees them", {
    {
        private _u = [_x] call skd_fnc_player;
        [alive _u, "==", true, format ["%1 is alive", _x]] call skd_fnc_expect;
        [vehicle _u == _u, "==", true, format ["%1 is on foot", _x]] call skd_fnc_expect;
        [damage _u, "==", 0, format ["the damage of %1", _x]] call skd_fnc_expect;
    } forEach ["p1", "p2"];
}] call skd_fnc_step;
