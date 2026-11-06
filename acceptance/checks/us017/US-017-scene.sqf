// @check US-017 A placed player, a created soldier, a date and a weather are the same on every machine
// @tags US-017 SCN-01 SCN-02 SCN-03 API-03 acceptance

/* One file for singleplayer and a dedicated session: it names the player with skd_fnc_player
   and every machine with the group all. */

private _p1 = ["p1"] call skd_fnc_player;
private _pos = AGLToASL [120, 130, 0];

["place p1 and read it on every machine", {
    params ["_p1", "_pos"];
    private _done = ["p1", _pos, 135] call skd_fnc_place;
    [_done, "==", true, "the value skd_fnc_place returned"] call skd_fnc_expect;
    ["all", { getPosASL (_this select 0) }, "~=", [_pos, 0.1], "where every machine sees p1", 5, [_p1]] call skd_fnc_expectEach;
    ["all", { getDir (_this select 0) }, "~=", [135, 3], "where every machine sees p1 face", 5, [_p1]] call skd_fnc_expectEach;
}, [_p1, _pos]] call skd_fnc_step;

["create a soldier", {
    private _unit = ["B_Soldier_F", AGLToASL [125, 130, 0], 90] call skd_fnc_create;
    [side group _unit, "==", west, "the side of the soldier's group"] call skd_fnc_expect;
    [getDir _unit, "~=", [90, 3], "where the soldier faces"] call skd_fnc_expect;
}] call skd_fnc_step;

["set the date and the weather", {
    private _date = [[2035, 6, 1, 19, 30]] call skd_fnc_setDate;
    [_date, "==", true, "the value skd_fnc_setDate returned"] call skd_fnc_expect;
    private _weather = [0.8, 0.2, 0] call skd_fnc_setWeather;
    [_weather, "==", true, "the value skd_fnc_setWeather returned"] call skd_fnc_expect;
    ["all", { date select [0, 5] }, "==", [2035, 6, 1, 19, 30], "every machine's date to the minute", 1] call skd_fnc_expectEach;
    ["all", { overcast }, "~=", [0.8, 0.01], "every machine's overcast", 1] call skd_fnc_expectEach;
}] call skd_fnc_step;
