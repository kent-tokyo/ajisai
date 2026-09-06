# Golden workflows for the rebuild

These workflows are test contracts for Phase 0 onward. The IDs remain stable even if the UI or native format changes.

| ID | User goal | Minimum path | Success evidence |
|---|---|---|---|
| GW-01 | Clean a CSV and write a CSV | select file -> preview -> select/derive -> filter -> review -> run | output hash, row/error counts, recoverable run record |
| GW-02 | Load a file into a database | choose source/destination -> test connections -> map schema -> review -> run | parameterized operation, commit result, redacted logs |
| GW-03 | Reuse a Hop pipeline safely | assess `.hpl` -> review supported/different/unsupported -> import -> validate -> preview | compatibility report and no silent loss |
| GW-04 | Recover a failed run | inspect failure -> fix config or input -> retry/resume where eligible | explicit terminal state and no false success |
| GW-05 | Deploy from CLI/CI | export project -> `doctor` -> dry-run -> run with JSON events | stable exit code, machine-readable result, reproducible artifact |

## Deliberately unsupported probes

- An active or unsafe XML construct must be rejected during assessment.
- A pipeline requesting a denied filesystem/network capability must fail before execution.
- A blocking transform exceeding its memory/disk policy must be rejected or explicitly spill; it must not silently consume unbounded memory.
