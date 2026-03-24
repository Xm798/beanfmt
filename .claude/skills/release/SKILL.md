---
name: release
description: Automate version releases — analyze git log since last tag to suggest semver bump, update version in Cargo.toml and editors/code/package.json, commit, tag, and push. Use when the user says "release", "bump version", "cut a release", "tag a new version", "publish a new version", or any variation of preparing a version release.
---

# Release

Automate the version release workflow for beanfmt. This is a rigid skill — follow the steps exactly in order.

## Step 1: Analyze changes and suggest version bump

Run `git tag --sort=-v:refname | head -1` to find the latest tag, then `git log <latest-tag>..HEAD --oneline` to list all commits since that tag.

Classify commits using Conventional Commits:

| Commit prefix | Bump type |
|---|---|
| `feat:` or `feat(…):` | minor |
| `fix:` or `fix(…):` | patch |
| `BREAKING CHANGE` in body, or `!:` suffix | major |
| `chore:`, `docs:`, `ci:`, `refactor:`, `test:`, `perf:`, `style:` | patch (if any feat/fix also present, those win) |

Pick the highest bump level among all commits. If there are no commits since the last tag, stop and tell the user there is nothing to release.

Present the analysis to the user:

```
Current version: x.y.z
Commits since vx.y.z:
  - feat(foo): add bar
  - fix(baz): correct qux

Suggested bump: minor → x.(y+1).0

Proceed? (or specify a different version)
```

Wait for user confirmation. The user may override the version.

## Step 2: Update version numbers

Two files contain the version and must be updated together:

1. `Cargo.toml` — line starting with `version = "…"` in the `[package]` section
2. `editors/code/package.json` — the `"version"` field

Use the Edit tool to update both files to the new version.

## Step 3: Commit

Create a commit with message: `chore: bump version to <new_version>`

Stage only the two changed files by name — do not use `git add -A`.

## Step 4: Create tag

Run `git tag v<new_version>` to create a lightweight tag on the commit.

## Step 5: Push

Show the user the push commands that will be executed:

```
git push origin <current-branch>
git push origin v<new_version>
```

Wait for user confirmation, then execute both push commands.
