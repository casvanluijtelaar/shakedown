// The fixture mod @skd_fixture (spec/contract/fixture-mod.md). It needs nothing but the game. Each
// behaviour of the contract is added with the first story or demo that uses it; a setup without
// CBA loads this mod alone.
class CfgPatches {
    class skdfx_main {
        name = "Shakedown fixture";
        author = "casvanluijtelaar";
        units[] = {"skdfx_Crate"};
        weapons[] = {};
        requiredVersion = 2.18;
        requiredAddons[] = {"A3_Functions_F", "A3_Weapons_F"};
        version = 0.0;
        versionStr = "0.0.5";
        versionAr[] = {0, 0, 5};
    };
};

class CfgFunctions {
    class skdfx {
        class main {
            file = "\skdfx\addons\main\functions";
            class init { postInit = 1; };
            class crateInit {};
            class open {};
            class setFlag {};
            class setFlagLater {};
            class throw {};
            // The reload target inside the mission (contract "A reload target inside the mission").
            class version {};
        };
    };
};

class CfgVehicles {
    class Box_NATO_Ammo_F;
    // The crate and its action (contract "The crate and its action").
    class skdfx_Crate: Box_NATO_Ammo_F {
        scope = 2;
        displayName = "Shakedown fixture crate";
        class EventHandlers {
            init = "_this call skdfx_fnc_crateInit";
        };
    };
};
