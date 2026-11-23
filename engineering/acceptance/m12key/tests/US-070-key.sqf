// @check US-070 with lib = "helpers" the library comes from helpers/
// @tags US-070 acceptance
// @setups sp

["the body and p1 call the library from helpers/", {
    [(call skd_lib_rope) get "version", "==", "helpers", "skd_lib_rope in the body"] call skd_fnc_expect;
    ["p1", { (call skd_lib_rope) get "version" }, "==", "helpers", "skd_lib_rope on p1"] call skd_fnc_expectOn;
    [isNil "skd_lib_kit", "==", true, "no skd_lib_kit from a tests/_lib/ this project does not have"] call skd_fnc_expect;
}] call skd_fnc_step;
