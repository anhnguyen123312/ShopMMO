# Git Worktree Workflow Skill

> **Trigger**: Use when implementing features that require isolated development environment, parallel development, or following software development standards with feature branches. Also use when merging feature branches back to master with automated conflict resolution.

## Overview

Git Worktree enables working on multiple branches simultaneously without switching context. This skill defines the standard workflow for feature development using worktrees, including the complete merge cycle.

## When to Use Git Worktree

| Scenario | Use Worktree |
|----------|--------------|
| New feature implementation | YES |
| Bug fixes on separate branches | YES |
| Hotfix while feature in progress | YES |
| Simple single-file changes | NO |
| Quick config updates | NO |

## Standard Workflow

### Phase 1: Setup Worktree

```bash
# 1. Ensure main branch is up to date
git fetch origin
git checkout master && git pull origin master

# 2. Create feature branch and worktree
# Pattern: git worktree add <path> -b <branch-name>
git worktree add ../mmo-<feature-name> -b feature/<feature-name>

# Example for auth implementation:
git worktree add ../mmo-auth -b feature/authorization-system-v2
```

### Phase 2: Development in Worktree

```bash
# 3. Navigate to worktree
cd ../mmo-<feature-name>

# 4. Verify you're on correct branch
git branch --show-current

# 5. Development cycle:
#    - Write code
#    - Run tests: cargo test
#    - Run checks: cargo check && cargo clippy
#    - Commit frequently with descriptive messages
```

### Phase 3: Commit Standards

```bash
# Commit message format:
# <type>(<scope>): <description>
#
# Types: feat, fix, docs, refactor, test, chore, deps
# Scope: module name (auth, wallet, permissions, etc.)

# Examples:
git commit -m "feat(permissions): add Permission enum with type-safe constants"
git commit -m "feat(permissions): implement role CRUD service"
git commit -m "test(permissions): add unit tests for permission validation"
git commit -m "docs(permissions): add OpenAPI documentation for role endpoints"
```

### Phase 4: Verification Before Merge

```bash
# Run full verification suite
cargo check
cargo clippy -- -D warnings
cargo test
cargo build --release

# If any step fails, fix and re-run until all pass
```

### Phase 5: Rebase and Merge to Master

```bash
# 1. Fetch latest from origin
git fetch origin

# 2. Rebase onto master (from feature branch worktree)
git rebase origin/master

# 3. If conflicts occur:
#    a. Resolve conflicts in conflicting files
#    b. git add <resolved-files>
#    c. git rebase --continue
#    d. Repeat until rebase completes

# 4. Run tests again after rebase
cargo test

# 5. Switch to main worktree and merge
cd /Volumes/Data/Git/mmo
git checkout master
git merge feature/<feature-name> --no-ff -m "Merge feature/<feature-name> into master"

# Alternative: Fast-forward merge if linear history
git merge feature/<feature-name> --ff-only
```

### Phase 6: Push and Cleanup

```bash
# 1. Push master to origin
git push origin master

# 2. Return to main worktree (if not already there)
cd /Volumes/Data/Git/mmo

# 3. Remove worktree
git worktree remove ../mmo-<feature-name>

# 4. Delete local branch
git branch -d feature/<feature-name>

# 5. Prune stale references
git worktree prune

# 6. Verify cleanup
git worktree list
git branch -a
```

## Automated Merge Workflow (Agent Execution)

When executing this workflow as an agent:

### Pre-merge Checklist
1. List all worktrees: `git worktree list`
2. For each worktree with uncommitted changes:
   - Check diff: `git diff`
   - Commit all changes with appropriate message
3. Run tests in each worktree: `cargo test`
4. Only proceed if tests pass

### Merge Sequence
For each feature worktree (in order):

```bash
# 1. In feature worktree, rebase onto master
cd <worktree-path>
git fetch origin
git rebase master

# 2. Resolve any conflicts automatically:
#    - For Rust imports: prefer alphabetical order
#    - For Cargo.toml: keep both dependencies
#    - For code conflicts: analyze both versions and merge logically
#    - Run cargo fmt after conflict resolution

# 3. After rebase, verify:
cargo check
cargo test

# 4. If tests fail after merge:
#    - Analyze error
#    - Fix the issue
#    - Commit fix: git commit -m "fix: resolve merge conflict in <module>"
#    - Re-run tests

# 5. Return to main worktree and merge
cd /Volumes/Data/Git/mmo
git merge <branch-name>
```

### Conflict Resolution Rules

| File Type | Resolution Strategy |
|-----------|---------------------|
| Cargo.toml | Keep both dependencies, merge features |
| mod.rs | Combine all module declarations |
| imports | Alphabetical order, remove duplicates |
| tests | Keep both test functions |
| domain/dto | Keep both fields, resolve naming conflicts |

## Worktree Management Commands

| Command | Description |
|---------|-------------|
| `git worktree list` | List all worktrees |
| `git worktree add <path> -b <branch>` | Create new worktree with new branch |
| `git worktree add <path> <existing-branch>` | Create worktree from existing branch |
| `git worktree remove <path>` | Remove a worktree |
| `git worktree prune` | Clean up stale worktree references |

## Directory Naming Convention

```
/Volumes/Data/Git/
├── mmo/                          # Main worktree (master)
├── mmo-auth/                     # Feature: authorization system
├── mmo-wallet-v2/                # Feature: wallet system v2
├── mmo-hotfix-123/               # Hotfix for issue #123
└── mmo-refactor-middleware/      # Refactoring task
```

## Integration with Development Plans

When implementing from a plan document (like `docs/plans/*.md`):

1. **Read the plan** to understand scope and tasks
2. **Create worktree** with appropriate feature name
3. **Create todo list** based on plan tasks
4. **Implement task by task**, committing after each
5. **Run verification** (cargo check, cargo test, cargo clippy)
6. **Rebase and merge** when all tasks complete
7. **Cleanup worktree** after successful merge

## Example: Complete Feature Lifecycle

```bash
# Setup
git worktree add ../mmo-auth -b feature/authorization-system-v2
cd ../mmo-auth

# Implement (following plan tasks)
# ... development work ...

# After each task:
cargo check
cargo test --lib
git add -A
git commit -m "feat(permissions): <task description>"

# Final verification
cargo test
cargo clippy -- -D warnings
cargo build --release

# Rebase onto master
git fetch origin
git rebase origin/master
# ... resolve any conflicts ...
cargo test  # Verify after rebase

# Merge to master
cd /Volumes/Data/Git/mmo
git merge feature/authorization-system-v2
git push origin master

# Cleanup
git worktree remove ../mmo-auth
git branch -d feature/authorization-system-v2
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "fatal: is already checked out" | Branch is checked out in another worktree |
| Worktree path conflicts | Use unique directory names |
| Stale worktree references | Run `git worktree prune` |
| Need to switch worktree branch | Remove and recreate worktree |
| Rebase conflicts | Resolve file by file, run `git rebase --continue` |
| Tests fail after merge | Fix failing tests, commit as "fix: resolve merge issues" |

## Checklist Before Starting Feature

- [ ] Main branch is up to date (`git pull origin master`)
- [ ] Feature branch name follows convention (`feature/<name>`)
- [ ] Worktree directory follows naming (`../mmo-<feature>`)
- [ ] Plan document read and understood
- [ ] Todo list created for tracking

## Checklist Before Merge

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code compiles in release mode (`cargo build --release`)
- [ ] Commits follow message convention
- [ ] Rebased onto latest master
- [ ] No uncommitted changes in worktree

## Checklist After Merge

- [ ] Master pushed to origin
- [ ] Worktree removed
- [ ] Local feature branch deleted
- [ ] `git worktree prune` executed
- [ ] `git worktree list` shows only main worktree
