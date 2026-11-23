// @check US-042 LOG-03 p2 logs a line in the check after
// @tags US-042 LOG-03 acceptance
// @setups ded

["p2", { diag_log "acc-m04 US-042 a line of the next check"; true }] call skd_fnc_on;
