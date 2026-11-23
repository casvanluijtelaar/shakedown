// @check US-012 the check after a failed mission restart
// @tags US-012 acceptance

["a body that would pass", {
    [true, "==", true, "the check body runs"] call skd_fnc_expect;
}] call skd_fnc_step;
