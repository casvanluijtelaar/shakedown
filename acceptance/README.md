# Acceptance suite

The acceptance suite belongs to the test developer. It drives the running system through its public interface, the way the documentation says a person drives it, and checks what each user story promises. It is written from the contract (`spec/`, `docs/`, `plan/milestones/`) and never from the source.

## Layout

- `docs/` - how the harness was built and what it learned (OKF bundle).
- `findings/` - one Finding concept per defect or document gap, in `findings/mNN/`. The format is in [findings/format.md](findings/format.md).
- `coverage/` - one Coverage concept per milestone: the result of the last full run.
- The harness itself: plain TypeScript with no dependencies, run with Node. Its folders are added in the first tester round.

## Running it

The commands to run the suite, and the filter that runs only the checks a finding names, are written here by the first tester round that builds the harness. Until then there is nothing to run.

## Who reads what

- The **test developer** owns everything in this folder.
- The **backend developer** reads this README and `findings/`, nothing else. It runs the checks a finding names through the filter, which is not reading them, and writes only `addressed` entries and findings for a frontend.
- A **frontend developer** writes only its own findings and the `addressed` entries on findings raised for it.
- The **technical writer** writes only findings for the backend and `addressed` entries on document gaps.
