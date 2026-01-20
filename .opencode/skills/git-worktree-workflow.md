# Git Worktree Workflow Skill

> **Trigger**: Use when implementing features that require isolated development environment, parallel development, or following software development standards with feature branches.

## Overview

Git Worktree enables working on multiple branches simultaneously without switching context. This skill defines the standard workflow for feature development using worktrees.

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

### Phase 4: Push and PR

```bash
# 6. Push feature branch
git push -u origin feature/<feature-name>

# 7. Create PR (using gh CLI)
gh pr create --title "feat: <description>" --body "## Summary
- <bullet points of changes>

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Related
Closes #<issue-number>"
```

### Phase 5: Cleanup

```bash
# After PR is merged:

# 8. Return to main worktree
cd /Volumes/Data/Git/mmo

# 9. Update main branch
git checkout master && git pull origin master

# 10. Remove worktree
git worktree remove ../mmo-<feature-name>

# 11. Delete local branch (optional, usually auto-deleted after merge)
git branch -d feature/<feature-name>

# 12. List remaining worktrees to verify
git worktree list
```

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
6. **Push and create PR** when all tasks complete

## Example: Implementing Authorization System

```bash
# Setup
git worktree add ../mmo-auth -b feature/authorization-system-v2
cd ../mmo-auth

# Implement (following plan tasks)
# Task 1: Add permission constants
# Task 2: Update role domain model
# Task 3: Create role management service
# ... etc

# After each task:
cargo check
cargo test --lib
git add -A
git commit -m "feat(permissions): <task description>"

# Final verification
cargo test
cargo clippy -- -D warnings
cargo build --release

# Push and PR
git push -u origin feature/authorization-system-v2
gh pr create --title "feat(auth): implement dynamic authorization system v2" --body "..."
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "fatal: is already checked out" | Branch is checked out in another worktree |
| Worktree path conflicts | Use unique directory names |
| Stale worktree references | Run `git worktree prune` |
| Need to switch worktree branch | Remove and recreate worktree |

## Checklist Before Starting Feature

- [ ] Main branch is up to date (`git pull origin master`)
- [ ] Feature branch name follows convention (`feature/<name>`)
- [ ] Worktree directory follows naming (`../mmo-<feature>`)
- [ ] Plan document read and understood
- [ ] Todo list created for tracking

## Checklist Before PR

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code compiles in release mode (`cargo build --release`)
- [ ] Commits follow message convention
- [ ] PR description is complete
- [ ] Related issues are linked
