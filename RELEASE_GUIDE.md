# Release Guide — ajisai 0.3.0

## Pre-Release Checklist

### 1. Code Quality
- [ ] All tests pass: `cargo test --workspace`
- [ ] TypeScript compilation: `npm run check` (from electron/)
- [ ] Linting: No warnings in code
- [ ] Code review: At least one approval

### 2. Documentation
- [ ] CHANGELOG.md updated with all changes
- [ ] README.md reflects current features
- [ ] API documentation up-to-date
- [ ] Installation instructions verified

### 3. Versioning
- [ ] Version bumped in electron/package.json: `0.3.0`
- [ ] Git tag created: `git tag v0.3.0`
- [ ] Cargo.toml versions updated (if applicable)

### 4. Performance Validation
- [ ] 50+ node pipeline test: >30 FPS
- [ ] Memory usage within limits: <200MB
- [ ] Bundle size acceptable: <100MB app size
- [ ] No regressions from previous version

### 5. Cross-Platform Testing
- [ ] macOS: Works on both Intel and Apple Silicon
- [ ] Windows: Both NSIS and portable installers work
- [ ] Linux: AppImage and deb packages functional
- [ ] First-run experience verified on all platforms

## Release Process

### Step 1: Prepare Release Branch

```bash
# Create release branch
git checkout -b release/0.3.0

# Update versions
nano electron/package.json          # 0.3.0
nano Cargo.toml                     # 0.3.0 (if applicable)
nano CHANGELOG.md                   # Add 0.3.0 section

# Commit changes
git add -A
git commit -m "chore: bump version to 0.3.0"
```

### Step 2: Create Tag

```bash
# Create annotated tag
git tag -a v0.3.0 -m "Release 0.3.0: UI Polish & Performance Optimization"

# Verify tag
git show v0.3.0

# Push tag
git push origin v0.3.0
```

### Step 3: GitHub Actions Build

1. Go to **Actions** tab on GitHub
2. Wait for "Release" workflow to complete
3. Monitor build output for errors
4. Verify all platform builds succeeded

**Workflow runs:**
- macOS (Intel + Apple Silicon)
- Windows (NSIS + Portable)
- Linux (AppImage + deb)

### Step 4: Create GitHub Release

```bash
# Using GitHub CLI (gh)
gh release create v0.3.0 \
  --title "ajisai 0.3.0: UI Polish & Performance" \
  --notes-file CHANGELOG.md \
  electron/dist/* \
  target/release/ajisai-server
```

Or manually:
1. Go to **Releases** tab
2. Click "Draft a new release"
3. Select tag `v0.3.0`
4. Copy CHANGELOG.md content as description
5. Upload binaries
6. Publish

### Step 5: Post-Release

```bash
# Merge release branch to main
git checkout main
git merge release/0.3.0
git push origin main

# Announce release
# - Update website
# - Post announcement
# - Send notifications
```

## Code Signing & Notarization

### macOS Code Signing

```bash
# Requires: Apple Developer Certificate
# Set environment variables:
export APPLE_ID="your-apple-id@example.com"
export APPLE_ID_PASSWORD="your-app-specific-password"
export APPLE_TEAM_ID="XXXXXXXXXX"

# electron-builder will automatically sign
npm run build
```

### Windows Code Signing

```bash
# Requires: Code signing certificate (.pfx file)
# Set environment variables:
export WIN_CSC_LINK="path/to/certificate.pfx"
export WIN_CSC_KEY_PASSWORD="certificate-password"

npm run build
```

## Auto-Update Verification

After release, verify auto-update works:

1. Install version 0.2.0 (or previous)
2. App checks for updates
3. New version 0.3.0 is detected
4. Download and install automatically
5. App restarts with new version

**Check logs:** `Help > About > Check for Updates`

## Troubleshooting

### Build Fails on macOS
```bash
# Check Xcode installation
xcode-select --install

# Clear cache
rm -rf node_modules/.vite
npm ci
npm run build
```

### Code Signing Certificate Errors
```bash
# List installed certificates
security find-identity -v -p codesigning

# Import certificate
security import certificate.p12 -k ~/Library/Keychains/login.keychain
```

### GitHub Actions Timeout
- Increase timeout in `.github/workflows/release.yml`
- Check network connectivity
- Verify sufficient GitHub Actions minutes

## Release Checklist Template

```markdown
## Release 0.3.0

- [ ] All tests pass
- [ ] CHANGELOG updated
- [ ] Version bumped
- [ ] Performance validated
- [ ] Tag created
- [ ] Actions workflow completed
- [ ] Binaries uploaded
- [ ] Release published
- [ ] Website updated
- [ ] Announcement posted
```

## Version History

| Version | Date | Focus |
|---------|------|-------|
| 0.3.0 | 2026-06-09 | UI Polish & Performance |
| 0.2.0 | 2026-05-31 | Metrics & Analytics |
| 0.1.0 | 2026-03-15 | Initial Release |

---

**Next Release:** 0.4.0 (scheduled for Q3 2026)
- Advanced data preview
- Column profiling
- Plugin system
