// @check US-037 Evidence attached by a passing check is kept in the result
// @tags US-037 acceptance

["attach four values", {
    ["number", 42] call skd_fnc_evidence;
    ["text", "hello"] call skd_fnc_evidence;
    ["list", [1, "two", [3]]] call skd_fnc_evidence;
    ["flag", true] call skd_fnc_evidence;
}] call skd_fnc_step;
