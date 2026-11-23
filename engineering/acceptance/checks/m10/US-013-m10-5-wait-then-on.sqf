// @check US-013 DM-14 Twenty crates, each waited for on every client right after skd_fnc_create, are then read in place there with skd_fnc_on
// @tags US-013 DM-14 m10-5 acceptance
// @timeout 300

// docs/dedicated-sessions.md: wait first, then act. skd_fnc_waitUntil looks the object up again before every
// evaluation, so it finds a just-created crate as soon as it reaches the client; skd_fnc_on that follows gets it.
private _clients = (call skd_fnc_machines) select { (_x select [0, 1]) == "p" };
private _waits = [];
for "_i" from 1 to 20 do {
    [format ["crate %1", _i], {
        params ["_i", "_clients"];
        private _pos = AGLToASL [130 + 3 * _i, 140, 0];
        private _crate = ["Land_WoodenBox_F", _pos, 0] call skd_fnc_create;
        {
            private _t0 = diag_tickTime;
            [_x, { params ["_c", "_p"]; !isNull _c && { ((getPosASL _c) distance2D _p) < 0.1 } },
                format ["crate %1 on %2, in place", _i, _x], 10, [_crate, _pos]] call skd_fnc_waitUntil;
            _waits pushBack (diag_tickTime - _t0);
        } forEach _clients;
        private _read = [_clients, { params ["_c"]; [isNull _c, getPosASL _c] }, [_crate]] call skd_fnc_onEach;
        {
            private _got = _read getOrDefault [_x, [true, [0, 0, 0]]];
            [_got select 0, "==", false, format ["skd_fnc_on on %1 gets crate %2, not objNull", _x, _i]] call skd_fnc_expect;
            [_got select 1, "~=", [_pos, 0.1], format ["crate %1 as %2 reads it with skd_fnc_on", _i, _x]] call skd_fnc_expect;
        } forEach _clients;
    }, [_i, _clients]] call skd_fnc_step;
};
["seconds each wait for a just-created crate took on a client", _waits] call skd_fnc_evidence;
