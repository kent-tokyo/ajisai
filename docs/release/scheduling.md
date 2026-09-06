# Scheduling and automation recipes

Ajisai does not start a hidden always-on scheduler. Use the host scheduler or
CI and invoke the pinned CLI explicitly.

## Cron (Unix)

```cron
*/15 * * * * /opt/ajisai/ajisai-cli run-workflow -p /srv/ajisai/project.hwf --project-root /srv/ajisai --json --record /var/lib/ajisai/last-run.json
```

## CI

```yaml
- run: ./ajisai-cli run-workflow -p project.hwf --project-root . --no-network --json --record ci-run.json
```

## Container

```bash
printf '%s\n' '{"id":1,"method":"ping"}' | \
  docker run --rm -i --read-only --tmpfs /tmp:rw,noexec,nosuid \
  ajisai:release-candidate
```

The pinned image currently exposes the JSON-RPC server entrypoint; invoke the
CLI separately for workflow execution. Pin the CLI/container digest and
preserve JSON records as run artifacts.
