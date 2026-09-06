# Benchmark contract

`manifest.json` pins the first deterministic fixture and its expected output hash.
Use `python3 scripts/run-benchmark.py --release --iterations 3 --output /tmp/ajisai-benchmark.json`
to produce a local report. The harness builds once, then records the first direct
binary invocation as `cold` and later invocations as `warm`; it also records
output bytes, a platform-normalized cumulative child peak RSS value, and checks
every output hash. Reports are diagnostic until the same workload is run on
Apache Hop with matched hardware and semantics; no performance number is asserted
by this file. For repeated runs against an already-built binary, add
`--skip-build` to avoid recompiling while preserving the workload and hash checks.
On Windows, the RSS field is `null` because the standard library has no portable
child-resource counter.

## Required comparison dimensions

- cold and warm startup;
- throughput and elapsed time;
- peak RSS and temporary disk use;
- output correctness hash and schema;
- cancellation, error, and partial-output behavior.

The first comparison oracle is Apache Hop 2.19.0. Runs must use identical input semantics, selected compression/encoding, durability policy, and hardware. A benchmark that does not meet those conditions is diagnostic only and cannot support a product claim.

## Initial corpus IDs

`csv-narrow-1m`, `csv-wide-1m`, `csv-null-heavy-10m`, `join-skewed-5m`, and `mixed-io-1m` remain reserved IDs. Generators and expected hashes must be added before Phase 8 measurement gates are evaluated.
