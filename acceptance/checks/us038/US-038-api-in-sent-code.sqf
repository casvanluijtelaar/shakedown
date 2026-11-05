// @check US-038 A check API call inside code sent to a machine is the author's to fix
// @tags US-038 acceptance

["send an assertion to p1", {
    ["p1", {
        [1, "==", 1, "an assertion where none may run"] call skd_fnc_expect;
        true
    }] call skd_fnc_on
}] call skd_fnc_step;
