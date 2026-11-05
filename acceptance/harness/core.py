"""The check registry and the shared state of one harness run."""

import json
from pathlib import Path

from . import schema

CHECKS = []


class Check:
    def __init__(self, name, phase, needs, fn):
        self.name, self.phase, self.needs, self.fn = name, phase, list(needs), fn


def check(name, phase, needs=()):
    """Registers an acceptance check.

    phase: "offline" runs once with no game; "session" runs once per repeat inside one game
    session; "end" runs once after the session stopped. needs: the SQF check ids a "session"
    check reads from the batch run.
    """
    def wrap(fn):
        CHECKS.append(Check(name, phase, needs, fn))
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

    def run(self, ids, ceiling=600):
        before = set(self.sh.run_folders())
        answer = self.sh("run", *ids, "--json", ceiling=ceiling)
        out = answer.json()
        new = sorted(set(self.sh.run_folders()) - before)
        result = json.loads(Path(out["result_file"]).read_text(encoding="utf-8"))
        self.runs.append((answer, out, result, new))
        return answer, out, result

    def validate(self, result):
        return schema.validate(result, self.schema)


def execution(result, check_id):
    found = [c for c in result["checks"] if c["check_id"] == check_id]
    assert len(found) == 1, f"the result file holds {len(found)} executions of {check_id}, expected 1"
    return found[0]
