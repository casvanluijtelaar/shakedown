// @check US-013 CTL-02 DM-13 DM-14 code on p2 gets its args, and p1's unit passed to p2 arrives as the same object
// @tags US-013 CTL-02 DM-13 DM-14 acceptance
// @setups ded

private _r = ["p2", { [_this select 0, name player] }, [42]] call skd_fnc_on;
["p2 answer", _r] call skd_fnc_evidence;
[_r select 0, "==", 42, "the first element p2 returned is 42"] call skd_fnc_expect;
[_r select 1, "==", "p2", "p2's player is named p2"] call skd_fnc_expect;

private _unit = ["p1", { player }] call skd_fnc_on;
[netId _unit, "==", netId (["p1"] call skd_fnc_player), "the object p1 returned is p1's unit on the server"] call skd_fnc_expect;
private _seen = ["p2", { [netId (_this select 0), name (_this select 0), isPlayer (_this select 0)] }, [_unit]] call skd_fnc_on;
["p2 sees", _seen] call skd_fnc_evidence;
[_seen, "==", [netId _unit, "p1", true], "p2 gets p1's unit as the same object"] call skd_fnc_expect;
