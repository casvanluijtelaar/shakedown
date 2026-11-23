// @check RUN-01 Selection target a
// @tags RUN-01 RUN-03 acc-sel-x

["pass", {
    [1, "==", 1, "one is one"] call skd_fnc_expect;
}] call skd_fnc_step;
