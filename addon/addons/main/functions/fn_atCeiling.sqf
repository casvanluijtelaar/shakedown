/*
    Called by a wait that reached its ceiling. When the check's own timeout has passed too, the
    ceiling was cut to the time left, so the wait stands still and the check ends with
    check-timeout, as the check's timeout decides (US-039). Otherwise it returns at once.
*/
if (diag_tickTime >= (skd_check get "deadline") - 0.05) then {
    waitUntil { false };
};
