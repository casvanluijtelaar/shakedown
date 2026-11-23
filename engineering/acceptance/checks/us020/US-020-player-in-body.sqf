// @check US-020 ACT-01 player as the target in the body of a dedicated check ends check-error naming skd_fnc_player
// @tags US-020 ACT-01 acceptance

["p1 uses an action on player", {
    ["p1", player, "Open crate"] call skd_fnc_useAction;
}] call skd_fnc_step;

["never runs", {}] call skd_fnc_step;
