# Security policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through
[GitHub Security Advisories](https://github.com/kent-tokyo/ajisai/security/advisories/new).
Do not open a public issue or include exploitable proof-of-concept details in a
public pull request. Include the affected version or commit, deployment
platform, reproduction steps, impact, and any safe mitigation.

The maintainers will acknowledge a report within 7 calendar days, triage it,
and coordinate a fix or mitigation and disclosure date with the reporter. These
targets are service goals, not a guarantee; emergencies may require an earlier
private release.

## Supported versions

| Version line | Support status |
| --- | --- |
| `2.x` (latest minor) | Supported after the v2.0.0 stable release |
| `<2.0.0` development baselines | Not a supported production security target |

Security fixes are developed against the latest supported `2.x` minor. Backport
requests are evaluated case by case. Release notes must identify affected
versions, mitigation, and whether migration is required.
