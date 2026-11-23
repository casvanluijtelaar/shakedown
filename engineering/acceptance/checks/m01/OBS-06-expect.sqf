// @check OBS-06 Polled assertions hold and return the values they read
// @tags OBS-06 acceptance

["expect on p1", {
    private _last = ["p1", { 1 + 1 }, "==", 2, "one plus one on p1"] call skd_fnc_expectOn;
    [_last, "==", 2, "the value skd_fnc_expectOn returned"] call skd_fnc_expect;
}] call skd_fnc_step;

["expect a value that comes later", {
    private _last = ["p1", { diag_tickTime > (_this select 0) }, "==", true, "half a second has passed", 5, [diag_tickTime + 0.5]] call skd_fnc_expectOn;
    [_last, "==", true, "the value skd_fnc_expectOn returned"] call skd_fnc_expect;
}] call skd_fnc_step;

["expect on each machine", {
    private _each = ["all", { 3 }, ">=", 1, "three is at least one"] call skd_fnc_expectEach;
    [_each get "p1", "==", 3, "p1's value in the map skd_fnc_expectEach returned"] call skd_fnc_expect;
}] call skd_fnc_step;

["every machine agrees", {
    private _each = ["all", { getPosASL player }, "every machine agrees where p1 is", 0.1] call skd_fnc_expectAgree;
    [typeName (_each get "p1"), "==", "ARRAY", "p1's value in the map skd_fnc_expectAgree returned"] call skd_fnc_expect;
}] call skd_fnc_step;

["the operators", {
    [[1, 2], "!=", [2, 1], "!="] call skd_fnc_expect;
    [1, "<", 2, "<"] call skd_fnc_expect;
    [2, "<=", 2, "<="] call skd_fnc_expect;
    [3, ">", 2, ">"] call skd_fnc_expect;
    [3, ">=", 3, ">="] call skd_fnc_expect;
    [10.05, "~=", [10, 0.1], "~= on a number"] call skd_fnc_expect;
    [[0, 0, 0], "~=", [[0, 0.05, 0], 0.1], "~= on a position"] call skd_fnc_expect;
    ["b", "in", ["a", "b"], "in"] call skd_fnc_expect;
    [["a", "b"], "contains", "a", "contains on an array"] call skd_fnc_expect;
    ["rope holds", "contains", "hold", "contains on a string"] call skd_fnc_expect;
}] call skd_fnc_step;
