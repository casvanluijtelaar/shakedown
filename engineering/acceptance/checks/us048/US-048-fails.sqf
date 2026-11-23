// @check US-048 AGT-02 a check that fails on purpose
// @tags US-048 AGT-02 acceptance

["fail on purpose", {
    ["acc-m04 the check fails on purpose"] call skd_fnc_fail;
}] call skd_fnc_step;
