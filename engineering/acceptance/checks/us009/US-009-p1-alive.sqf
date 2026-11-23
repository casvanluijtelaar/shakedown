// @check US-009 The check after p1 was killed finds p1 alive
// @tags US-009 RST-05 acceptance

["p1 alive", {
    ["p1", {alive player}, "==", true, "p1 alive"] call skd_fnc_expectOn;
}] call skd_fnc_step;
