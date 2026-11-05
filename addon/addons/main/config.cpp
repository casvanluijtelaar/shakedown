class CfgPatches {
    class shakedown_main {
        name = "Shakedown";
        author = "casvanluijtelaar";
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.18;
        requiredAddons[] = {"A3_Functions_F"};
        version = 0.0;
        versionStr = "0.0.2";
        versionAr[] = {0, 0, 2};
    };
};

class CfgFunctions {
    class skd {
        // The in-game agent: receives orders from the extension, runs them, sends results back,
        // and records and resets the mission's state.
        class agent {
            file = "\z\shakedown\addons\main\functions";
            class init { postInit = 1; };
            class receive {};
            class send {};
            class clocks {};
            class encode {};
            class decode {};
            class cross {};
            class runExec {};
            class runCheck {};
            class end {};
            class argError {};
            class guard {};
            class ceiling {};
            class atCeiling {};
            class targets {};
            class compileFor {};
            class runOn {};
            class poller {};
            class stopPoller {};
            class show {};
            class compare {};
            class wait {};
            class expectPoll {};
            class actionState {};
            class baseline {};
            class killed {};
            class reset {};
        };
        // The check API (spec/contract/check-api.md).
        class api {
            file = "\z\shakedown\addons\main\functions";
            class machines {};
            class setup {};
            class player {};
            class step {};
            class on {};
            class onEach {};
            class waitUntil {};
            class tryWaitUntil {};
            class expect {};
            class expectOn {};
            class expectEach {};
            class expectAgree {};
            class create {};
            class place {};
            class setDate {};
            class setWeather {};
            class useAction {};
            class pressKey {};
            class cleanup {};
            class evidence {};
            class fail {};
            class invalid {};
        };
    };
};
