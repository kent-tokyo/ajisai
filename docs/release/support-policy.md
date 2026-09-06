# v2.0.0 support policy

v2.0.0 starts the stable support line. The latest `2.x` minor receives security
fixes and release-blocking correctness fixes. Deprecations require a notice in
release notes and at least one minor release of migration guidance before
removal, unless a security issue requires an emergency change.

Native document schemas and extension APIs use explicit version fields. A
breaking schema or extension change requires a migration path, compatibility
notes, and a rollback instruction. Unsupported development baselines are not
promised production support.

Security reports follow the repository [security policy](../../SECURITY.md).
