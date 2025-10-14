# Branchspawn CLI Release Plan

## Overview
This document outlines the complete release strategy for the branchspawn CLI tool, including branching strategy, semantic versioning, and GitHub Actions workflows.

## Git Branching Strategy

### Branch Structure
```
main (production)
├── develop (integration)
├── feature/feature-name
├── release/v1.0.0
└── hotfix/critical-fix
```

### Branch Purposes

#### `main`
- **Purpose**: Production-ready, stable code
- **Protection**: Requires PR reviews, status checks
- **Direct commits**: Not allowed
- **Releases**: All releases are tagged from this branch

#### `develop`
- **Purpose**: Integration branch for ongoing development
- **Merges from**: Feature branches, release branches
- **Merges to**: Release branches
- **Testing**: Continuous integration runs on all commits

#### `feature/*`
- **Purpose**: New feature development
- **Naming**: `feature/descriptive-name`
- **Base**: Created from `develop`
- **Merge to**: `develop` via Pull Request
- **Cleanup**: Deleted after merge

#### `release/*`
- **Purpose**: Release preparation and stabilization
- **Naming**: `release/v1.2.0`
- **Base**: Created from `develop`
- **Changes**: Only bug fixes, no new features
- **Merge to**: Both `main` and `develop`
- **Lifecycle**: Short-lived (days, not weeks)

#### `hotfix/*`
- **Purpose**: Critical production fixes
- **Naming**: `hotfix/critical-issue-name`
- **Base**: Created from `main`
- **Merge to**: Both `main` and `develop`
- **Release**: Can trigger immediate release

## Semantic Versioning

### Version Format: `MAJOR.MINOR.PATCH`

- **MAJOR** (1.0.0 → 2.0.0): Breaking changes
- **MINOR** (1.0.0 → 1.1.0): New features, backward compatible
- **PATCH** (1.0.0 → 1.0.1): Bug fixes, backward compatible

### Pre-release Versions
- **Release Candidate**: `v1.2.0-rc.1`
- **Beta**: `v1.2.0-beta.1`
- **Alpha**: `v1.2.0-alpha.1`

### Tagging Convention
- All releases tagged as `v{version}` (e.g., `v1.0.0`)
- Tags created only on `main` branch
- Annotated tags with release notes

## Release Process

### Regular Release Flow
1. **Feature Development**
   ```bash
   git checkout develop
   git checkout -b feature/new-feature
   # ... development work ...
   # Create PR to develop
   ```

2. **Release Preparation**
   ```bash
   git checkout develop
   git checkout -b release/v1.1.0
   # Update version in Cargo.toml
   # Update CHANGELOG.md
   # Final testing and bug fixes
   ```

3. **Release Finalization**
   ```bash
   # Create PR from release/v1.1.0 to main
   # After merge, create tag on main
   git checkout main
   git tag -a v1.1.0 -m "Release version 1.1.0"
   git push origin v1.1.0
   ```

4. **Backmerge**
   ```bash
   # Merge main back to develop to sync changes
   git checkout develop
   git merge main
   ```

### Hotfix Flow
1. **Create Hotfix**
   ```bash
   git checkout main
   git checkout -b hotfix/critical-security-fix
   # ... fix the issue ...
   # Update version (patch increment)
   ```

2. **Deploy Hotfix**
   ```bash
   # Create PR to main
   # After merge, tag immediately
   git checkout main
   git tag -a v1.0.1 -m "Hotfix: Critical security fix"
   git push origin v1.0.1
   ```

3. **Sync with Develop**
   ```bash
   git checkout develop
   git merge main
   ```

## GitHub Actions Workflows

### 1. CI/CD Pipeline (`ci.yml`)
**Triggers**: PR to main/develop, push to main/develop
**Jobs**:
- Code quality checks (clippy, fmt)
- Unit tests
- Integration tests
- Security audit
- Build verification

### 2. Release Pipeline (`release.yml`)
**Triggers**: Tags matching `v*`
**Jobs**:
- Cross-platform builds (Linux, macOS, Windows)
- Binary optimization
- Create GitHub release
- Upload release assets
- Update package managers (if applicable)

### 3. Hotfix Pipeline (`hotfix.yml`)
**Triggers**: Push to `hotfix/*` branches
**Jobs**:
- Fast CI checks
- Emergency build verification
- Notification to team

## Version Management

### Cargo.toml Version Updates
- Manual updates in release branches
- Automated verification in CI
- Consistent with git tags

### Changelog Management
- Maintain `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/)
- Update during release preparation
- Include in release notes

## Security Considerations

### Branch Protection
- `main`: Require PR reviews, dismiss stale reviews
- `develop`: Require status checks
- No direct pushes to protected branches

### Secrets Management
- Use GitHub secrets for sensitive data
- Rotate tokens regularly
- Minimal permission principle

### Release Signing
- Sign releases with GPG (recommended)
- Verify build integrity
- Secure artifact storage

## Rollback Strategy

### Failed Release
1. Revert merge commit on `main`
2. Delete problematic tag
3. Investigate and fix issues
4. Create new release

### Critical Issues
1. Use hotfix process for urgent fixes
2. Communicate with users via GitHub releases
3. Update documentation if needed

## Monitoring and Metrics

### Release Health
- Download statistics
- User feedback
- Issue reports
- Performance metrics

### Process Metrics
- Release frequency
- Lead time
- Failure rate
- Recovery time

## Getting Started

### Initial Setup
1. Create `develop` branch from `main`
2. Set up branch protection rules
3. Configure GitHub Actions
4. Update team on new process

### First Release
1. Follow regular release flow
2. Start with patch version increment
3. Test all workflows
4. Document lessons learned

## Tools and Resources

### Recommended Tools
- **Cargo**: Rust package manager
- **cross**: Cross-compilation tool
- **cargo-audit**: Security vulnerability scanner
- **cargo-deny**: Dependency verification

### Documentation
- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Semantic Versioning Spec](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Rust Release Checklist](https://github.com/rust-lang/rfcs/blob/master/text/1122-language-semver.md)