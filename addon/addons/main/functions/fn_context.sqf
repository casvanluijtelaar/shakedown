/*
    Marks the start or the end of a run or a check execution on this machine (P-05 step 4,
    LOG-04): writes the marker line "[shakedown] <key> <edge> <utc>" into this machine's RPT, so
    the runner can cut the lines between the two out of it, and, for a check execution, sets or
    clears the run and check that stamp this machine's script errors (ERR-01).

    Arguments: [key, edge, checkId] - key the run id, or "<run id>#<n>" for the run's nth check
    execution; edge "start" or "end"; checkId the check's id for a check execution, "" for a run.
    Returns: true.
*/
params ["_key", "_edge", ["_checkId", ""]];
diag_log text format ["[shakedown] %1 %2 %3", _key, _edge, (call skd_fnc_clocks) get "utc"];
if (_checkId isNotEqualTo "") then {
    skd_context = if (_edge isEqualTo "start") then { [_key, _checkId] } else { ["", ""] };
};
true
