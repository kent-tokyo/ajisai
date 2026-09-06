# Container image

The checked-in `Dockerfile` builds a pinned Rust 1.97 server and runs it as the
unprivileged `ajisai` user. The server speaks line-delimited JSON-RPC on stdin
and stdout; it does not expose a hidden network listener.

For a read-only root filesystem, provide only explicit writable mounts:

```bash
docker run --rm --read-only --tmpfs /tmp:rw,noexec,nosuid \
  --mount type=bind,src="$PWD/project",dst=/workspace/project,readonly \
  ghcr.io/kent-tokyo/ajisai:v2.0.0
```

The image build, digest, SBOM, checksum, signature, and runtime smoke test must
be recorded for each published tag. The example tag above is a target, not a
claim that the image has already been published.
