// @check US-037 Every step is recorded in start order with its path, machine, state and clocks
// @tags US-037 acceptance

/* Each wait below polls the frame counter, so every step spans frames and time. */
["outer", {
    private _outerFrame = diag_frameNo;
    ["inner one", {
        private _frame = diag_frameNo;
        waitUntil { diag_frameNo >= _frame + 3 };
        ["clocks inside inner one", [diag_tickTime, time, diag_frameNo]] call skd_fnc_evidence;
    }] call skd_fnc_step;
    ["inner two", {
        ["deepest", {
            private _frame = diag_frameNo;
            waitUntil { diag_frameNo >= _frame + 3 };
        }] call skd_fnc_step;
    }] call skd_fnc_step;
    waitUntil { diag_frameNo >= _outerFrame + 10 };
}] call skd_fnc_step;

["last", {
    ["p1", { diag_frameNo }] call skd_fnc_on
}] call skd_fnc_step;
