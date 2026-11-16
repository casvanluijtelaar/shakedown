// @check US-076 a check that passes about one time in two, decided by random
// @tags US-076 acceptance
// @setups sp

private _coin = random 1;
["coin", _coin] call skd_fnc_evidence;
["the coin falls heads", {
    [_coin, "<", 0.5, "the coin falls heads"] call skd_fnc_expect;
}] call skd_fnc_step;
