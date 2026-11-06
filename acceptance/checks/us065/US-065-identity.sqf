// @check US-065 A check that attaches, logs and fails with the Steam ID and the Steam name leaves placeholders
// @tags US-065 LOG-02 NFR-PV-01 NFR-PV-02 acceptance

private _id = ["read them on p1", {
    ["p1", { [getPlayerUID player, profileNameSteam] }] call skd_fnc_on
}] call skd_fnc_step;
["uid", _id select 0] call skd_fnc_evidence;
["name", _id select 1] call skd_fnc_evidence;

["log them on p1", {
    ["p1", { diag_log format ["uid %1 name %2", getPlayerUID player, profileNameSteam]; true }] call skd_fnc_on;
}] call skd_fnc_step;

["fail with them", {
    [format ["uid %1 name %2", _this select 0, _this select 1]] call skd_fnc_fail;
}, _id] call skd_fnc_step;
