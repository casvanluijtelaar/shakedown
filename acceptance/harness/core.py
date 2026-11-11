"""The check registry and the shared state of one harness run."""

import json
from pathlib import Path

from . import schema

CHECKS = []
BATCHES = []
AFTER_SESSION = []  # fn(ctx, setup), called after each session stopped, such as to put back a fixture file

# The setups of the generated project, in the order their sessions run. The last one's session
# is the one the "end" checks look at.
SETUPS = ["ded", "rld", "rld7", "rld7sp", "cba", "sp"]


class Check:
    def __init__(self, name, phase, needs, fn, setup):
        self.name, self.phase, self.needs, self.fn, self.setup = name, phase, list(needs), fn, setup


def check(name, phase, needs=(), setup="sp"):
    """Registers an acceptance check.

    phase: "offline" runs once with no game; "start" runs once before the first session and
    starts and stops what it needs itself; "session" runs once per repeat inside the session of
    its setup; "end" runs once after the last session stopped; "nosteam" needs no steam process
    running, so it runs, before the "start" checks, only when a --check value selects it.
    needs: the SQF check ids a "session" check reads from the runs its setup's batches make.
    """
    def wrap(fn):
        CHECKS.append(Check(name, phase, needs, fn, setup))
        return fn
    return wrap


def batch(setup):
    """Registers the runs one module makes per repeat in the session of a setup.

    The function is called as fn(ctx, checks) with the selected session checks of that setup, and
    stores what it ran on ctx for the checks to judge.
    """
    def wrap(fn):
        BATCHES.append((setup, fn))
        return fn
    return wrap


class Context:
    def __init__(self, sh, project, repo, log):
        self.sh, self.project, self.repo, self.log = sh, Path(project), Path(repo), log
        self.schema = json.loads((self.repo / "spec" / "contract" / "result.schema.json").read_text(encoding="utf-8"))
        self.session = None          # the Session JSON of `session start`
        self.stopped = None          # the Session JSON of `session stop`
        self.stop_answer = None
        self.runs = []               # every (answer, run_result JSON, result file dict, new run folders)
        self.batch = None            # this repeat's batch run: (answer, run_result, result)
        self.fixed = None            # this repeat's rerun of the fixed syntax error, or None
        self.runner_pid = None
        self.game_pids = set()
        self.session_runs = []       # the runs of the current session
        self.m01 = {}                # what M1's batches ran this repeat
        self.m02 = {}                # what M2's batches ran this repeat
        self.m03 = {}                # what M3's batches ran this repeat
        self.m03_keep = {}           # what M3's end checks read after the sessions stopped
        self.m04 = {}                # what M4's batches did this repeat
        self.m04_keep = {}           # what M4's start checks did, for its session and end checks
        self.m06 = {}                # what M6's batch did this repeat
        self.m06_keep = {}           # what M6's batch did over every repeat
        self.m07 = {}                # what M7's batch did this repeat
        self.m07_keep = {}           # setup -> the game-restart reload durations of M7 over every repeat
        self.sessions = {}           # setup -> the Session JSON of its `session start`
        self.start_answer = None     # the answer of the current session's `session start`
        self.stops = {}              # setup -> (the `session stop` answer, game pids alive when it returned)
        self.failed_starts = []      # the Session JSON of every start the "start" checks made fail

    def project_ids(self):
        """The id of every check file in the generated project's checks folder."""
        root = self.project / "checks"
        return sorted(p.relative_to(root).with_suffix("").as_posix() for p in root.rglob("*.sqf"))

    def run(self, ids, ceiling=600, tags=()):
        before = set(self.sh.run_folders())
        args = list(ids) + [a for t in tags for a in ("--tag", t)]
        answer = self.sh("run", *args, "--json", ceiling=ceiling)
        out = answer.json()
        new = sorted(set(self.sh.run_folders()) - before)
        result = json.loads(Path(out["result_file"]).read_text(encoding="utf-8"))
        self.runs.append((answer, out, result, new))
        self.session_runs.append((answer, out, result, new))
        return answer, out, result

    def validate(self, result):
        return schema.validate(result, self.schema)


def execution(result, check_id):
    found = [c for c in result["checks"] if c["check_id"] == check_id]
    assert len(found) == 1, f"the result file holds {len(found)} executions of {check_id}, expected 1"
    return found[0]
