/*
    Captures the reset baseline at the mission's start (RST-02): the player unit's position (ASL),
    direction, loadout and animation state, on a machine with a player; the date, and the
    overcast, fog and rain. From here the entity recorder counts what is created (SCN-04), and the
    groups that exist are the mission's own.
*/
skd_baseline = createHashMapFromArray [
    ["date", date],
    ["overcast", overcast],
    ["fog", fog],
    ["rain", rain]
];
if (hasInterface && { !isNull player }) then {
    skd_baseline set ["position", getPosASL player];
    skd_baseline set ["direction", getDir player];
    skd_baseline set ["loadout", getUnitLoadout player];
    skd_baseline set ["animation", animationState player];
    skd_baseline set ["side", side group player];
};
skd_created = [];
skd_baseGroups = +allGroups;
skd_cleanups = [];
