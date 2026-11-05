// @check US-038 A body with a syntax error is the author's to fix
// @tags US-038 acceptance

/* The harness replaces the line below with its fixed form for the rerun. */
private _value = (1 + ;

["the fixed body computes two", {
    [_value, "==", 2, "the value computed in the body"] call skd_fnc_expect;
}] call skd_fnc_step;
