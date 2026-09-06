# Security and dependency audit — updated 2026-09-06

## Commands

- `cargo deny check licenses sources` — **pass**
- `cargo deny check advisories` — **pass** after updating transitive `spin` to 0.9.9
- CI runs the checked-in cargo-deny policy on every change through a dedicated
  dependency-audit job; dependency warnings remain visible and are not waived.
- `RUSTFLAGS=-Dunsafe_code cargo check --workspace --all-targets` — **pass**
- `bash scripts/secret-scan.sh` — **pass**

## Remediated

- `quick-xml` upgraded to `0.41.0` in Hop compatibility and transforms. This
  removes the audited 0.31/0.37 parser path associated with the duplicate
  attribute CPU exhaustion and unbounded namespace allocation advisories.
- `calamine` upgraded to `0.36.1`, removing its transitive `quick-xml 0.31`
  path.
- `anyhow` upgraded to `1.0.103` for RUSTSEC-2026-0190.
- `crossbeam-epoch` upgraded to `0.9.20` for RUSTSEC-2026-0204.

## Unresolved blockers

- `rsa 0.9.10` (Marvin timing attack) is isolated behind the explicit SQLx `mysql`
  feature and is not present in the default Ajisai build graph. MySQL remains
  opt-in and is outside the current candidate connector set until it receives a
  separate dependency and threat-model review.
- `smartstring 1.0.1` (unmaintained), pulled by Rhai only when the opt-in `scripting` feature is enabled; it is absent from the default candidate graph.
- `spin 0.9.9` is now resolved transitively by SQLx 0.9/SQLite dependencies; the prior yanked-package warning is closed.
- Parquet/Arrow upgraded to `59.3.0`; the former `paste` advisory is no longer
  present in the resolved dependency graph.

Workflow variable assignment no longer mutates process-global environment
variables; it uses the shared execution context. CI enforces `RUSTFLAGS=-Dunsafe_code`
for workspace code. Third-party dependency unsafe code is outside that local
lint and still requires independent review. CI also runs a conservative
implementation/configuration secret scan; historical commits and entropy-based
provider scanning remain outside this local check. REST target validation now
rejects private, loopback, link-local, multicast, and unspecified literal IPs;
hostnames are resolved and checked before request dispatch to reduce DNS
rebinding exposure. TLS pinning remains open.

The yanked-package blocker is resolved and the decision is recorded in
[`ADR 0008`](../adr/0008-sqlite-yanked-spin-release-gate.md). Independent
security review and artifact verification remain required before production
release.
