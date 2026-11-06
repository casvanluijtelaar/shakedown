// [] call skdfx_fnc_throw: raises one script error on the machine it runs on, whose text contains
// _skdfx_undefined, an undefined local variable used in an expression; then returns nil
// (spec/contract/fixture-mod.md, "A script error on demand").
private _skdfx_sum = _skdfx_undefined + 1;
nil
