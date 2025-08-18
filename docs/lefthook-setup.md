# Lefthook Setup and Usage

This project uses [Lefthook](https://github.com/evilmartians/lefthook) for Git hooks to ensure code quality and consistency.

## What is Lefthook?

Lefthook is a fast and powerful Git hooks manager that runs linting, testing, and other checks automatically when you commit or push code.

## Automatic Setup

Lefthook is automatically installed when you run:

```bash
pnpm install
```

The `prepare` script in `package.json` will automatically install the Git hooks.

## Manual Setup

If you need to manually install or reinstall the hooks:

```bash
npx lefthook install
```

## Git Hooks Configuration

### Pre-commit Hook

Runs automatically before each commit:

- **Frontend Lint**: TypeScript type checking (`tsc --noEmit`)
- **Rust Format Check**: Ensures Rust code is properly formatted (`cargo fmt --check`)
- **Rust Clippy**: Runs Rust linter with strict rules
- **Rust Tests**: Runs all Rust tests

### Pre-push Hook

Runs automatically before pushing to remote:

- **Build Check**: Ensures both frontend and Rust code builds successfully

### Commit Message Hook

Validates commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>[optional scope]: <description>

Examples:
feat: add new LED strip configuration
fix(ui): resolve button alignment issue
docs: update installation instructions
```

**Valid types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`, `revert`

## Manual Commands

You can also run these checks manually:

```bash
# Run all pre-commit checks
npx lefthook run pre-commit

# Run pre-push checks
npx lefthook run pre-push

# Individual commands
pnpm lint              # Frontend TypeScript check
pnpm lint:rust         # Rust clippy linting
pnpm fmt:rust          # Format Rust code
pnpm test:rust         # Run Rust tests
```

## Skipping Hooks

In rare cases, you can skip hooks (not recommended):

```bash
# Skip all hooks
git commit --no-verify -m "emergency fix"

# Skip specific hook
LEFTHOOK=0 git commit -m "skip all lefthook hooks"
```

## Troubleshooting

### Hook not running

```bash
# Reinstall hooks
npx lefthook install
```

### Permission issues

```bash
# Make sure lefthook is executable
chmod +x .git/hooks/*
```

### Update lefthook

```bash
pnpm update lefthook
npx lefthook install
```

## Benefits

- ✅ **Consistent Code Quality**: Automatic formatting and linting
- ✅ **Early Error Detection**: Catch issues before they reach CI/CD
- ✅ **Conventional Commits**: Standardized commit messages
- ✅ **Fast Feedback**: Local checks are faster than CI
- ✅ **Team Consistency**: Same checks for all developers
