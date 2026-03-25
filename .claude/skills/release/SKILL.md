---
name: release
description: Automate version releases — analyze git log since last tag to suggest semver bump, update version, commit, tag, and push. Supports two independent release targets: core (crates.io + PyPI, tag v*) and vscode (VS Code Marketplace, tag vscode-v*). Use when the user says "release", "bump version", "cut a release", "tag a new version", "publish a new version", or any variation of preparing a version release.
---

# Release

Automate the version release workflow for beanfmt. This is a rigid skill — follow the steps exactly in order.

There are two independent release targets with separate version numbers:

| Target | Version source | Tag format | Publishes to |
|---|---|---|---|
| **core** | `Cargo.toml` | `v0.7.0` | crates.io + PyPI + GitHub Release |
| **vscode** | `editors/code/package.json` | `vscode-v0.7.0` | VS Code Marketplace + GitHub Release |

## Step 0: Determine release target

Ask the user which target to release. If the user already specified (e.g. "release vscode", "release core"), skip the question.

- **core** — Rust library + Python bindings (version in `Cargo.toml`)
- **vscode** — VS Code extension (version in `editors/code/package.json`)

## Step 1: Analyze changes and suggest version bump

Determine the latest tag for the chosen target:

- **core**: `git tag --sort=-v:refname | grep -E '^v[0-9]' | head -1`
- **vscode**: `git tag --sort=-v:refname | grep -E '^vscode-v' | head -1`

Then list commits since that tag: `git log <latest-tag>..HEAD --oneline`

For **core**, filter to commits affecting core files (exclude `editors/code/**`-only changes).
For **vscode**, filter to commits affecting `editors/code/**` or WASM-related changes.

Classify commits using Conventional Commits:

| Commit prefix | Bump type |
|---|---|
| `feat:` or `feat(…):` | minor |
| `fix:` or `fix(…):` | patch |
| `BREAKING CHANGE` in body, or `!:` suffix | major |
| `chore:`, `docs:`, `ci:`, `refactor:`, `test:`, `perf:`, `style:` | patch (if any feat/fix also present, those win) |

Pick the highest bump level among all commits. If there are no relevant commits since the last tag, stop and tell the user there is nothing to release.

Present the analysis to the user:

```
Target: core (or vscode)
Current version: x.y.z
Commits since <tag>:
  - feat(foo): add bar
  - fix(baz): correct qux

Suggested bump: minor → x.(y+1).0

Proceed? (or specify a different version)
```

Wait for user confirmation. The user may override the version.

## Step 2: Update version numbers

### For core release:

1. `Cargo.toml` — line starting with `version = "…"` in the `[package]` section
2. `Cargo.lock` — run `cargo generate-lockfile` after updating `Cargo.toml`

Use the Edit tool to update `Cargo.toml`, then run `cargo generate-lockfile`.

### For vscode release:

1. `editors/code/package.json` — the `"version"` field

Use the Edit tool to update `editors/code/package.json`.

## Step 3: Commit

Create a commit with message:

- **core**: `chore: bump version to <new_version>`
- **vscode**: `chore(vscode): bump version to <new_version>`

Stage only the changed files by name — do not use `git add -A`.

## Step 4: Create tag

- **core**: `git tag v<new_version>`
- **vscode**: `git tag vscode-v<new_version>`

## Step 5: Push

Show the user the push commands that will be executed:

```
git push origin <current-branch>
git push origin <tag>
```

Wait for user confirmation, then execute both push commands.
