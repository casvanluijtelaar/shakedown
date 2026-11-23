// @check US-033 the first check of the walk-away suite: every machine of the session answers
// @tags US-033 acceptance
// @timeout 30

["every machine answers", {
    ["all", { 1 }, "==", 1, "one from every machine", 10] call skd_fnc_expectEach;
}] call skd_fnc_step;
