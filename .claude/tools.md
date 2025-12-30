# Claude Tools for TaphoaMMO V2

## Quick Reference

### Context Management

#### Load Module Context
```
Before working on module X:
1. Read .claude/context/{module}.md
2. Check refs to related modules
3. Review V1 docs if feature exists
```

#### Update Context
```
After completing work:
1. Update .claude/context/{module}.md với changes
2. Add lesson to .claude/lessons.md
3. Update refs nếu có breaking changes
```

### Development Commands

#### Create Module
```bash
# Follow .claude/skills/create-module.md
mkdir -p src/modules/{name}
# Create files in order: domain → dto → repo → service → handler → routes → mod
```

#### Run & Test
```bash
cargo run                    # Start server
cargo test                   # Run tests
cargo fmt && cargo clippy    # Format & lint
```

#### Test Endpoint
```bash
# Health
curl http://localhost:8080/health

# With auth
curl -H "Authorization: Bearer {token}" http://localhost:8080/api/{path}
```

### Documentation Patterns

#### Flow Diagram
```
[Step 1] → [Step 2] → [Step 3]
    ↓           ↓
[Branch]   [Branch]
```

#### Data Model (define only)
```rust
struct Entity {
    field: Type,
}
```

#### API Endpoint Table
| Method | Path | Auth | Desc |
|--------|------|------|------|
| GET | /path | - | ... |

### File Locations

| Type | Location |
|------|----------|
| Rules | .claude/CLAUDE.md |
| Module Context | .claude/context/{module}.md |
| Skills | .claude/skills/{skill}.md |
| Lessons | .claude/lessons.md |
| Research | .claude/research/{topic}.md |
| V1 Docs | docs/v1/*.md |
| V2 Designs | docs/v2/*.md |

### Memory Update Trigger
Sau mỗi task hoàn thành, tự động:
1. Summarize work done
2. Add to lessons.md nếu learned something
3. Update context file nếu module changed
