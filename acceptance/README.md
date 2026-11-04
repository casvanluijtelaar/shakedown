# Acceptance suite

The acceptance suite belongs to the test developer. It drives the built framework through its public interface (the SQF check API, the command line, the agent tools and the result file) against the game, the way the documentation says a person drives it, and checks what each user story promises. It is written from the contract (`spec/`, `docs/`, `plan/milestones/`) and never from the source.

## Layout

- `docs/` - how the harness was built and what it learned (OKF bundle).
- `findings/` - one Finding concept per defect or document gap, in `findings/mNN/`. The format is in [findings/format.md](findings/format.md).
- `coverage/` - one Coverage concept per milestone: the result of the last full run.
- The harness itself, in the language `spec/07-tech-stack.md` sets for it. Its folders are added in the first tester round.

## Running it

The commands to run the suite, and the filter that runs only the checks a finding names, are written here by the first tester round that builds the harness. Until then there is nothing to run.

## Who reads what

- The **test developer** owns everything in this folder.
- The **framework developer** reads this README and `findings/`, nothing else. It runs the checks a finding names through the filter, which is not reading them, and writes only `addressed` entries.
- The **technical writer** writes only findings for the framework developer and `addressed` entries on document gaps.
