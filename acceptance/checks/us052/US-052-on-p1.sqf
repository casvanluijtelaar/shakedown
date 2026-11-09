// @check US-052 SEC-04 a check that sends code to p1 with skd_fnc_on, for audit.jsonl
// @tags US-052 SEC-04 acceptance
// @setups ded

private _v = ["p1", { 40 + 2 + 0 * 52052 }] call skd_fnc_on;
[_v, "==", 42, "p1 answered 42"] call skd_fnc_expect;
