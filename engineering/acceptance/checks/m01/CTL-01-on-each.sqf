// @check CTL-01 skd_fnc_onEach returns a map from machine name to value
// @tags CTL-01 acceptance

["on all machines", {
    private _each = ["all", { 6 * 7 }] call skd_fnc_onEach;
    [typeName _each, "==", "HASHMAP", "the type skd_fnc_onEach returned"] call skd_fnc_expect;
    [keys _each, "==", ["p1"], "the machines in the map"] call skd_fnc_expect;
    [_each get "p1", "==", 42, "p1's value"] call skd_fnc_expect;
}] call skd_fnc_step;

["on a list of machines", {
    private _each = [["p1"], { _this select 0 }, ["x"]] call skd_fnc_onEach;
    [_each get "p1", "==", "x", "p1's value"] call skd_fnc_expect;
}] call skd_fnc_step;

["a machine whose code returned nothing has no key", {
    private _each = ["all", { nil }] call skd_fnc_onEach;
    [count _each, "==", 0, "the number of keys"] call skd_fnc_expect;
}] call skd_fnc_step;
