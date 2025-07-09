# Build System Overview

This document explains the two-tier build system for the Ambient Light Control desktop application.

## Build Types

The project uses a simplified two-tier build system:

1. **Development Builds** - Automatic builds for testing
2. **Release Builds** - Manual builds for production

## Development Builds

Development builds automatically create pre-release versions for every commit pushed to the `main` branch. These builds are intended for testing and development purposes.

## How It Works

### Automatic Triggers

- **Trigger**: Every push to the `main` branch
- **Workflow**: `.github/workflows/dev-build.yml`
- **Platforms**: macOS (Universal), Windows (x64), Linux (x64)

### Version Generation

Development versions follow this format:
```
{base-version}.dev.{timestamp}.{commit-hash}
```

Example: `2.0.0-alpha.dev.202501091430.a1b2c3d`

Where:
- `2.0.0-alpha` - Base version from configuration
- `dev` - Development build identifier
- `202501091430` - Timestamp (YYYYMMDDHHMM)
- `a1b2c3d` - Short commit hash

### Build Process

1. **Version Update**: Updates version in all configuration files:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`

2. **Cross-Platform Build**: Builds for all supported platforms:
   - **macOS**: Universal binary (Intel + Apple Silicon)
   - **Windows**: x64 MSI and NSIS installers
   - **Linux**: x64 DEB package and AppImage

3. **Release Creation**: Creates a pre-release on GitHub with:
   - Descriptive release notes
   - Build artifacts for all platforms
   - Development build warnings

4. **Cleanup**: Automatically removes old development releases (keeps latest 10)

## Artifacts

Each development build produces the following artifacts:

### macOS
- **DMG**: Disk image installer
- **APP**: Application bundle

### Windows
- **MSI**: Windows Installer package
- **EXE**: NSIS installer

### Linux
- **DEB**: Debian/Ubuntu package
- **AppImage**: Portable application

## Usage

### For Developers

1. **Push to main**: Development builds are triggered automatically
2. **Check releases**: Visit the GitHub releases page to download builds
3. **Test locally**: Use the provided script for local version updates

### Local Development

To update version locally for testing:

```bash
# Generate automatic development version
./scripts/update-dev-version.sh

# Or specify a custom version
./scripts/update-dev-version.sh "2.0.0-alpha.dev.custom.test"
```

To revert version changes:
```bash
git checkout -- package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
```

## Release Management

### Development Releases
- **Location**: GitHub Releases page
- **Naming**: `dev-{timestamp}-{commit-hash}`
- **Type**: Pre-release
- **Retention**: Latest 10 builds (older ones auto-deleted)

### Production Releases
- **Workflow**: `.github/workflows/release.yml`
- **Trigger**: Manual workflow dispatch
- **Type**: Full release

## Important Notes

### ⚠️ Development Build Warnings

- Development builds are **not suitable for production use**
- May contain bugs, incomplete features, or breaking changes
- Features and APIs may change without notice
- No stability guarantees

### Version Conflicts

- Development builds update version numbers in configuration files
- Always revert version changes before committing other changes
- Use the provided script for consistent version formatting

### Build Artifacts

- All builds are cross-platform and include debug information
- Artifacts are available for 90 days (GitHub default)
- Download links are provided in release notes

## Troubleshooting

### Build Failures

1. Check the Actions tab for build logs
2. Verify all configuration files have correct syntax
3. Ensure dependencies are properly specified

### Version Issues

1. Use the provided script for version updates
2. Check that all three configuration files are updated consistently
3. Verify version format matches expected pattern

### Artifact Issues

1. Check that build completed successfully for all platforms
2. Verify artifact upload steps completed without errors
3. Ensure GitHub token has appropriate permissions

## Configuration

### Workflow Configuration

The development build workflow can be customized by editing `.github/workflows/dev-build.yml`:

- **Base version**: Update `BASE_VERSION` variable
- **Retention**: Change the number in the cleanup script (default: 10)
- **Platforms**: Modify the build matrix
- **Triggers**: Adjust the `on` section

### Version Script

The local version update script can be customized by editing `scripts/update-dev-version.sh`:

- **Base version**: Update `BASE_VERSION` variable
- **Timestamp format**: Modify the `date` command
- **File paths**: Adjust paths if project structure changes

## Release Builds

Release builds are manually triggered production builds with semantic versioning.

### Features

- **Third-party Actions**: Uses proven GitHub Actions from the marketplace
- **Multiple version modes**: Semantic auto, manual input, or increment-based
- **Semantic auto mode**: Uses `paulhatch/semantic-version` for commit-based versioning
- **Manual version input**: Full semantic version control with validation
- **Automatic increment**: Patch, minor, or major version bumps
- **Version validation**: Ensures proper semantic version format
- **Automatic prerelease detection**: Detects alpha/beta/rc versions
- **Custom release notes**: Optional custom release notes input
- **Configuration updates**: Updates all version files automatically
- **Reliable release creation**: Uses `softprops/action-gh-release` for GitHub releases

### Usage

1. **Navigate to Actions**: Go to GitHub repository Actions tab
2. **Select workflow**: Choose "Release Build" workflow
3. **Run workflow**: Click "Run workflow" button
4. **Choose version mode**:
   - **Semantic Auto**: Uses commit messages with `(MAJOR)` or `(MINOR)` patterns
   - **Manual**: Enter complete version (e.g., `1.0.0`, `2.1.0-beta.1`)
   - **Patch**: Auto-increment patch (1.0.0 → 1.0.1)
   - **Minor**: Auto-increment minor (1.0.0 → 1.1.0)
   - **Major**: Auto-increment major (1.0.0 → 2.0.0)
5. **Add notes** (optional): Custom release notes
6. **Set prerelease** (optional): Mark as prerelease if needed

### Version Format

Release builds use semantic versioning with two input modes:

#### Semantic Auto Mode (Recommended)
Uses `paulhatch/semantic-version` action with commit message patterns:
- **Commit with `(MAJOR)`**: Triggers major version increment
- **Commit with `(MINOR)`**: Triggers minor version increment
- **Default**: Patch version increment
- **Example**: `feat: add new feature (MINOR)` → triggers minor version bump

#### Manual Mode
Enter complete semantic version:
- **Major.Minor.Patch**: `1.0.0`, `2.1.3`
- **Prerelease**: `1.0.0-alpha.1`, `2.0.0-beta.2`, `1.0.0-rc.1`
- **Build metadata**: `1.0.0+build.1` (optional)

#### Automatic Increment Mode
Choose increment type based on latest Git tag:
- **Patch**: `1.0.0` → `1.0.1` (bug fixes)
- **Minor**: `1.0.0` → `1.1.0` (new features, backward compatible)
- **Major**: `1.0.0` → `2.0.0` (breaking changes)

#### Third-Party Actions Used
- **`paulhatch/semantic-version@v5.4.0`**: Git-based semantic versioning
- **`softprops/action-gh-release@v1`**: Reliable GitHub release creation
- **Benefits**: Proven, maintained, and feature-rich solutions

### Automatic Prerelease Detection

The system automatically detects prerelease versions:
- Versions with `-alpha`, `-beta`, `-rc` are marked as prerelease
- Manual prerelease option overrides automatic detection
- Prerelease versions appear separately in GitHub releases

### Release Notes

- **Custom notes**: Use the release notes input field
- **Auto-generated**: Default template with installation instructions
- **Build information**: Includes version, commit, and build date

## Workflow Comparison

| Feature | Development Build | Release Build |
|---------|------------------|---------------|
| **Trigger** | Automatic (main push) | Manual (workflow dispatch) |
| **Version Input** | Auto-generated with timestamp | Manual or auto-increment |
| **Version Format** | `{base}.dev.{timestamp}.{hash}` | Semantic versioning |
| **Purpose** | Testing and development | Production release |
| **Frequency** | Every commit | On-demand |
| **Validation** | None | Semantic version validation |
| **Increment Options** | None | Patch/Minor/Major/Manual |
| **Prerelease Support** | Always prerelease | Optional with suffixes |
| **Release Notes** | Auto-generated | Custom or auto-generated |
| **Cleanup** | Auto (keeps 10) | Manual |
| **Base Version Source** | Configuration files | Latest Git tag |
