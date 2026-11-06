// @check US-028 A script error skdfx_fnc_throw raises on p2 in spawned code fails the check naming p2 and the step
// @tags US-028 ERR-01 ERR-03 acceptance

["trigger on p2", {
    ["p2", { acc_m03_spawned = false; [] spawn { [] call skdfx_fnc_throw; acc_m03_spawned = true }; true }] call skd_fnc_on;
    ["p2", { acc_m03_spawned }, "the spawned code on p2 went past the error"] call skd_fnc_waitUntil;
}] call skd_fnc_step;
