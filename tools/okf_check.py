#!/usr/bin/env python3
"""Bundle check for the OKF v0.2 document bundles of this repository.

Fails (exit 1) when a concept has no frontmatter or no `type`, when an
`index.md` misses a file or a subfolder, when a bundle root index has no
`okf_version`, when two concepts share an `id`, or when a story, milestone,
deferred item, finding, report or coverage names an id that does not exist.

Then prints the open and answered findings, the requirements no story covers,
and the stories no coverage run checks. Those lines are information, not
failures.

Run from anywhere: python3 tools/okf_check.py
"""

import re
import sys
from datetime import datetime, timezone
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
BUNDLES = [
    "research",
    "spec",
    "plan",
    "docs",
    "acceptance/docs",
    "acceptance/findings",
    "acceptance/coverage",
]
STATUSES = {"draft", "stable", "deprecated"}
STORY_KINDS = {"feature", "error", "mistake", "system fail", "malicious intent"}
MILESTONE_STATES = {"planned", "in-progress", "complete"}
DEFERRED_STATES = {"open", "planned", "done"}
COVERAGE_STATES = {"pass", "partial", "fail", "unchecked"}
FINDING_ROLES_RAISE = {"test", "developer", "writer"}
FINDING_ROLES_FOR = {"developer", "writer"}

# A requirement ID is defined where it is written in bold at the start of a
# list item, heading or table row in a top-level spec file: `- **R-01**`,
# `### **R-01**`, `| **R-01** |`.
REQ_DEF = re.compile(r"^\s*(?:[-*]|#+|\|)?\s*\*\*([A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*-?\d+)\*\*", re.M)
INDEX_LINK = re.compile(r"\]\(([^)]+)\)")

errors = []


def err(path, msg):
    errors.append(f"{path.relative_to(ROOT)}: {msg}")


def frontmatter(path):
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---", 4)
    if end == -1:
        return None
    try:
        data = yaml.safe_load(text[4:end])
    except yaml.YAMLError as e:
        err(path, f"frontmatter is not valid YAML: {e}")
        return {}
    return data if isinstance(data, dict) else {}


def as_list(v):
    if v is None:
        return []
    return v if isinstance(v, list) else [v]


def as_time(v):
    if isinstance(v, datetime):
        return v if v.tzinfo else v.replace(tzinfo=timezone.utc)
    if isinstance(v, str):
        try:
            t = datetime.fromisoformat(v)
            return t if t.tzinfo else t.replace(tzinfo=timezone.utc)
        except ValueError:
            return None
    return None


def check_index(folder, is_root):
    index = folder / "index.md"
    if not index.exists():
        err(folder / "index.md", "folder has no index.md")
        return
    fm = frontmatter(index)
    if fm is None:
        err(index, "index has no frontmatter")
        fm = {}
    if is_root and str(fm.get("okf_version", "")) != "0.2":
        err(index, 'bundle root index has no okf_version: "0.2"')
    linked = set()
    for target in INDEX_LINK.findall(index.read_text(encoding="utf-8")):
        if "://" in target or target.startswith("#"):
            continue
        target = target.split("#")[0]
        if target.endswith("/index.md"):
            target = target[: -len("index.md")]
        linked.add(target)
        if not (folder / target).exists():
            err(index, f"links to {target}, which does not exist")
    for child in sorted(folder.iterdir()):
        if child.name.startswith("."):
            continue
        if child.is_dir():
            if f"{child.name}/" not in linked:
                err(index, f"does not list the folder {child.name}/")
            check_index(child, False)
        elif child.suffix == ".md" and child.name != "index.md":
            if child.name not in linked:
                err(index, f"does not list {child.name}")


def main():
    concepts = []  # (path, frontmatter, bundle)
    for bundle in BUNDLES:
        base = ROOT / bundle
        if not base.is_dir():
            err(base, "bundle root does not exist")
            continue
        check_index(base, True)
        for path in sorted(base.rglob("*.md")):
            if path.name == "index.md":
                continue
            fm = frontmatter(path)
            if fm is None:
                err(path, "has no frontmatter")
                continue
            if not fm.get("type"):
                err(path, "frontmatter has no type")
                continue
            if not fm.get("title"):
                err(path, "frontmatter has no title")
            if not fm.get("description"):
                err(path, "frontmatter has no description")
            if "status" in fm and fm["status"] not in STATUSES:
                err(path, f"status {fm['status']!r} is not draft, stable or deprecated")
            concepts.append((path, fm, bundle))

    # .worktrees/ holds other checkouts of this repository, checked on their own.
    for path in ROOT.rglob("log.md"):
        if not {".git", ".worktrees", "node_modules"} & set(path.relative_to(ROOT).parts):
            err(path, "log.md is reserved by OKF and not used here; git holds the history")

    by_type = {}
    ids = {}
    for path, fm, _ in concepts:
        by_type.setdefault(fm["type"], []).append((path, fm))
        if "id" in fm:
            i = str(fm["id"])
            if i in ids:
                err(path, f"id {i} is also used by {ids[i].relative_to(ROOT)}")
            else:
                ids[i] = path

    roles = {str(fm.get("id")) for _, fm in by_type.get("Role", [])}
    stories = {str(fm.get("id")): (p, fm) for p, fm in by_type.get("User Story", [])}
    milestones = {str(fm.get("id")): (p, fm) for p, fm in by_type.get("Milestone", [])}

    requirements = set()
    for path in sorted((ROOT / "spec").glob("*.md")):
        requirements.update(REQ_DEF.findall(path.read_text(encoding="utf-8")))

    def check_refs(path, fm, key, known, what):
        for ref in as_list(fm.get(key)):
            if str(ref) not in known:
                err(path, f"{key} names {what} {ref}, which does not exist")

    for path, fm in by_type.get("Role", []):
        if not fm.get("id"):
            err(path, "Role has no id")

    for sid, (path, fm) in stories.items():
        if not re.fullmatch(r"US-\d+", sid):
            err(path, f"story id {sid} is not of the form US-nnn")
        if fm.get("kind") not in STORY_KINDS:
            err(path, f"kind {fm.get('kind')!r} is not one of {sorted(STORY_KINDS)}")
        if str(fm.get("role")) not in roles:
            err(path, f"role {fm.get('role')} is not the id of a Role concept")
        if path.parent.name != str(fm.get("role")):
            err(path, f"story lies in folder {path.parent.name}, not in the folder of its role")
        if not as_list(fm.get("requirements")):
            err(path, "story cites no requirements")
        check_refs(path, fm, "requirements", requirements, "requirement")

    placed = {}
    for mid, (path, fm) in milestones.items():
        if fm.get("state") not in MILESTONE_STATES:
            err(path, f"state {fm.get('state')!r} is not one of {sorted(MILESTONE_STATES)}")
        check_refs(path, fm, "stories", stories, "story")
        check_refs(path, fm, "requirements", requirements, "requirement")
        for s in as_list(fm.get("stories")):
            if str(s) in placed:
                err(path, f"story {s} is also in milestone {placed[str(s)]}")
            placed[str(s)] = mid

    for path, fm in by_type.get("Deferred", []):
        if fm.get("state") not in DEFERRED_STATES:
            err(path, f"state {fm.get('state')!r} is not one of {sorted(DEFERRED_STATES)}")
        if fm.get("milestone") is not None:
            check_refs(path, fm, "milestone", milestones, "milestone")

    for path, fm in by_type.get("Milestone Report", []):
        check_refs(path, fm, "milestone", milestones, "milestone")

    covered_by_check = set()
    for path, fm in by_type.get("Coverage", []):
        check_refs(path, fm, "milestone", milestones, "milestone")
        for r in as_list(fm.get("results")):
            if not isinstance(r, dict):
                err(path, "a results entry is not { story, state, checks }")
                continue
            if str(r.get("story")) not in stories:
                err(path, f"results names story {r.get('story')}, which does not exist")
            if r.get("state") not in COVERAGE_STATES:
                err(path, f"state {r.get('state')!r} for {r.get('story')} is not one of {sorted(COVERAGE_STATES)}")
            if r.get("state") in {"pass", "partial"}:
                covered_by_check.add(str(r.get("story")))

    open_findings, answered = [], []
    for path, fm in by_type.get("Finding", []):
        fid = str(fm.get("id", ""))
        if not re.fullmatch(r"m\d{2}-\d+", fid):
            err(path, f"finding id {fid} is not of the form mNN-n")
        check_refs(path, fm, "milestone", milestones, "milestone")
        check_refs(path, fm, "stories", stories, "story")
        if fm.get("raised_by") not in FINDING_ROLES_RAISE:
            err(path, f"raised_by {fm.get('raised_by')!r} is not one of {sorted(FINDING_ROLES_RAISE)}")
        if fm.get("for") not in FINDING_ROLES_FOR:
            err(path, f"for {fm.get('for')!r} is not one of {sorted(FINDING_ROLES_FOR)}")
        for key, need in (("addressed", ("by", "at", "note")), ("verified", ("by", "at"))):
            for e in as_list(fm.get(key)):
                if not isinstance(e, dict) or any(k not in e for k in need) or as_time(e.get("at")) is None:
                    err(path, f"an entry in {key} is not {{ {', '.join(need)} }} with a valid time")
        if fm.get("status") == "deprecated":
            continue
        addressed = [as_time(e.get("at")) for e in as_list(fm.get("addressed")) if isinstance(e, dict)]
        verified = [as_time(e.get("at")) for e in as_list(fm.get("verified")) if isinstance(e, dict)]
        addressed = [t for t in addressed if t]
        verified = [t for t in verified if t]
        label = f"{fid} ({fm.get('for')}) {fm.get('title')}"
        if not addressed:
            open_findings.append(label)
        elif not verified or max(addressed) > max(verified):
            answered.append(label)

    if errors:
        print(f"Bundle check failed with {len(errors)} error(s):")
        for e in errors:
            print(f"  {e}")
    else:
        print(f"Bundle check passed: {len(concepts)} concepts in {len(BUNDLES)} bundles.")

    cited = {str(r) for _, fm in stories.values() for r in as_list(fm.get("requirements"))}
    report = [
        ("Open findings", open_findings),
        ("Answered findings waiting for a verify", answered),
        ("Requirements no story covers", sorted(requirements - cited)),
        ("Stories no milestone delivers", sorted(set(stories) - set(placed))),
        ("Stories no check covers", sorted(set(stories) - covered_by_check)),
    ]
    for title, items in report:
        if items:
            print(f"\n{title} ({len(items)}):")
            for i in items:
                print(f"  {i}")

    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
