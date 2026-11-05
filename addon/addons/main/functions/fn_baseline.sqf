/*
    Captures the reset baseline at the mission's start (RST-02): the player unit's position (ASL),
    direction, loadout and animation state, the date, and the overcast, fog and rain. From here the
    entity recorder counts what is created (SCN-04), and the groups that exist are the mission's own.
*/
skd_baseline = createHashMapFromArray [
    ["position", getPosASL player],
    ["direction", getDir player],
    ["loadout", getUnitLoadout player],
    ["animation", animationState player],
    ["side", side group player],
    ["date", date],
    ["overcast", overcast],
    ["fog", fog],
    ["rain", rain]
];
skd_created = [];
skd_baseGroups = +allGroups;
skd_cleanups = [];
