# GitHub Actions Workflows

This directory contains GitHub Actions workflows for automated CI/CD processes.

## Workflows Overview

### 🚀 `release.yml` - Release Build

**Triggers:** Manual workflow dispatch

**Purpose:** Creates production releases with semantic versioning

**Features:**

- **Third-party Actions**: Uses proven `paulhatch/semantic-version` and `softprops/action-gh-release`
- **Multiple version modes**: Semantic auto, manual input, or increment-based
- **Semantic version validation**: Ensures proper version format
- **Automatic version updates**: Updates all configuration files
- **Cross-platform builds**: All supported platforms
- **Custom or auto-generated release notes**: Flexible release documentation
- **Automatic prerelease detection**: Based on version format

**Usage:**

1. Go to Actions tab in GitHub
2. Select "Release Build" workflow
3. Click "Run workflow"
4. Choose version mode:
   - **Semantic Auto**: Uses commit messages to determine version (MAJOR/MINOR patterns)
   - **Manual**: Enter complete version (e.g., 1.0.0, 2.1.0-beta.1)
   - **Patch**: Auto-increment patch version (1.0.0 → 1.0.1)
   - **Minor**: Auto-increment minor version (1.0.0 → 1.1.0)
   - **Major**: Auto-increment major version (1.0.0 → 2.0.0)
5. Optionally add custom release notes
6. Choose prerelease option if needed

**Artifacts:**

- **macOS**: DMG installer and .app bundle (Universal binary for Intel and Apple Silicon)

### 🚧 `dev-build.yml` - Development Build

**Triggers:** Push to main branch

**Purpose:** Creates automated development releases for every commit on main branch

**Features:**

- Automatic version generation with timestamp and commit hash
- Cross-platform builds for all supported platforms
- Creates pre-release with development artifacts
- Automatic cleanup of old development releases (keeps latest 10)
- Version updates in all configuration files

**Version Format:** `2.0.0-alpha.dev.YYYYMMDDHHMM.{commit-hash}`

**Artifacts:**

- **macOS**: DMG installer and .app bundle (Universal binary for Intel and Apple Silicon)

### 🧪 `ci.yml` - Continuous Integration
**Triggers:** Push to main/develop, Pull Requests

**Purpose:** Code quality checks and testing

**Features:**
- Frontend build verification
- Rust formatting and linting (rustfmt, clippy)
- Rust unit tests
- Security audits for both frontend and backend dependencies

### 🚀 `release.yml` - Manual Release
**Triggers:** Manual workflow dispatch

**Purpose:** Create tagged releases with built applications

**Features:**
- Manual version input
- Pre-release option
- Automatic release notes generation
- Cross-platform builds and uploads
- Comprehensive installation instructions

**Usage:**
1. Go to Actions tab in GitHub
2. Select "Release" workflow
3. Click "Run workflow"
4. Enter version (e.g., v1.0.0)
5. Choose if it's a pre-release
6. Click "Run workflow"

### 🔄 `dependencies.yml` - Dependency Management
**Triggers:** Weekly schedule (Mondays 9 AM UTC), Manual dispatch

**Purpose:** Automated dependency updates and security monitoring

**Features:**
- Weekly dependency updates
- Automatic PR creation for updates
- Security vulnerability detection
- Automatic issue creation for security alerts

## Setup Requirements

### Repository Secrets
No additional secrets are required beyond the default `GITHUB_TOKEN`.

### Branch Protection (Recommended)
Configure branch protection rules for `main` branch:
- Require status checks to pass before merging
- Require branches to be up to date before merging
- Include status checks: `lint-and-test`, `security-audit`

### Release Process

#### Automated (Recommended)
1. Merge changes to `main` branch
2. Use the manual release workflow to create a new release
3. The workflow will automatically build and upload all platform binaries

#### Manual
1. Create a new tag: `git tag v1.0.0`
2. Push the tag: `git push origin v1.0.0`
3. Create a release on GitHub
4. The build workflow will automatically attach binaries

## Platform-Specific Notes

### macOS
- Builds universal binaries (Intel + Apple Silicon)
- Requires macOS 13.0 or later
- DMG installer includes code signing (if certificates are configured)

### Windows
- Builds for x64 architecture
- Provides both MSI and NSIS installers
- Compatible with Windows 10 and later

### Linux
- Builds for x64 architecture
- Provides DEB package for Debian/Ubuntu
- Provides AppImage for universal Linux compatibility
- Requires WebKit2GTK and other system dependencies

## Troubleshooting

### Build Failures
1. Check the specific platform logs in the Actions tab
2. Ensure all dependencies are properly declared
3. Verify Tauri configuration is correct

### Security Audit Failures
1. Review the security report in the workflow logs
2. Update vulnerable dependencies
3. Consider using `pnpm audit --fix` for frontend issues
4. Use `cargo update` for Rust dependency updates

### Cache Issues
If builds are failing due to cache corruption:
1. Go to Actions tab
2. Click on "Caches" in the sidebar
3. Delete relevant caches
4. Re-run the workflow

## Customization

### Adding New Platforms
To add support for additional platforms, modify the `matrix` section in `build.yml`:

```yaml
matrix:
  include:
    - platform: 'macos-latest'
      args: '--target aarch64-apple-darwin'
      target: 'aarch64-apple-darwin'
```

### Modifying Build Steps
Each workflow can be customized by:
1. Adding new steps
2. Modifying existing commands
3. Adding environment variables
4. Configuring different Node.js/Rust versions

### Adding Code Quality Tools (Optional)
If you want to add code quality tools in the future:
1. **ESLint**: Add ESLint configuration and dependencies for JavaScript/TypeScript linting
2. **Prettier**: Add Prettier for consistent code formatting
3. **TypeScript strict checking**: Enable stricter TypeScript rules and type checking

### Changing Schedule
Modify the `cron` expression in `dependencies.yml` to change the update frequency:
```yaml
schedule:
  - cron: '0 9 * * 1'  # Every Monday at 9 AM UTC
```
