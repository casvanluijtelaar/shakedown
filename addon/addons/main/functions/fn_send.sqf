/*
    Sends a message to the runner through the extension, which queues it and returns at once.

    Arguments: [message] - HashMap with String keys, values already encoded (DM-13).
*/
params ["_message"];
private _answer = "shakedown" callExtension ["send", [toJSON _message]];
if !((_answer select 0) isEqualTo "ok") then {
    diag_log text format ["[shakedown] send failed: %1", _answer];
};
