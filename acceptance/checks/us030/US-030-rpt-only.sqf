// @check US-030 A script error written to p1's RPT without the handler firing fails the check from the RPT scan
// @tags US-030 ERR-02 INT-AR-07 acceptance

/* The lines are written as the engine writes a script error from a file, through diag_log, so
   the ScriptError handler never sees them (docs/script-errors-and-logs.md). */
["p1 logs an error", {
    ["p1", {
        diag_log text "Error in expression <acc_m03_rpt = _acc_m03_rpt_only + 1;>";
        diag_log text "  Error position: <_acc_m03_rpt_only + 1;>";
        diag_log text "  Error Undefined variable in expression: _acc_m03_rpt_only";
        diag_log text "File acc_m03\fn_rptOnly.sqf..., line 7";
        true
    }] call skd_fnc_on;
}] call skd_fnc_step;
