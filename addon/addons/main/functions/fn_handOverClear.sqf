/*
    Removes the list of a hand over (MAN-01) from this machine's screen, if it shows one.
*/
private _ctrl = uiNamespace getVariable ["skd_handOver", controlNull];
if (!isNull _ctrl) then { ctrlDelete _ctrl };
uiNamespace setVariable ["skd_handOver", controlNull];
