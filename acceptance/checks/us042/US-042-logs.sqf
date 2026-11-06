// @check US-042 LOG-03 LOG-04 p2 logs three numbered lines during this check
// @tags US-042 LOG-03 LOG-04 acceptance
// @setups ded

["p2", { diag_log "acc-m04 US-042 line 1"; diag_log "acc-m04 US-042 line 2"; diag_log "acc-m04 US-042 line 3"; true }] call skd_fnc_on;
["p1", { diag_log "acc-m04 US-042 a line of p1"; true }] call skd_fnc_on;
