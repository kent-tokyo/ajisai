# Ajisai risk register (working baseline)

This register is a planning artifact, not an independent security review.

| ID | Area | Risk | Baseline control | Release state |
|---|---|---|---|---|
| R-01 | Filesystem | traversal or symlink escape | reject parent components; project-root policy pending | open |
| R-02 | Network | SSRF or metadata access | `--no-network`, URL validation, blocked local/metadata hosts, optional REST host allowlist | open |
| R-03 | Secrets | credentials in documents/logs | redacted debug output; secret references pending | open |
| R-04 | Parsing | oversized/malformed XML, CSV, archives | bounded CSV channels and parser errors; fuzzing pending | open |
| R-05 | Execution | partial output after failure | CSV temp-file commit and drop cleanup | partial |
| R-06 | Extensions | arbitrary native code | no stable extension SDK; capability ADR accepted | open |
| R-07 | Dependencies | known transitive advisories | cargo-deny configuration; remediation pending | blocker |

Severity policy: any exploitable high/critical issue, silent partial commit, or
unbounded resource path blocks a stable release.
