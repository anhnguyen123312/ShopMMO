# .claude Directory Index

## Quick Start
1. Read [CLAUDE.md](CLAUDE.md) - main rules
2. Check [context/](context/) for module you're working on
3. Follow [skills/](skills/) for specific tasks

## Structure
```
.claude/
├── CLAUDE.md          # Main rules & refs
├── lessons.md         # Learnings & patterns
├── tools.md           # Commands & workflows
│
├── context/           # Module-specific context
│   ├── auth.md        # ✅ Reference implementation
│   ├── wallet.md      # 🚧 V2 in progress
│   ├── product.md     # 📋 Planned
│   ├── order.md       # 📋 Planned
│   ├── shop.md        # 📋 Planned
│   └── escrow.md      # 📋 Part of wallet
│
├── skills/            # How-to guides
│   ├── create-module.md
│   ├── brainstorm.md
│   ├── research.md
│   └── write-docs.md
│
├── templates/         # Reusable templates
│   └── v2-feature-design.md
│
└── research/          # Research notes (add as needed)
```

## Status Legend
- ✅ Implemented
- 🚧 In Progress
- 📋 Planned/Not Started

## Module Dependency Graph
```
         ┌─────────────────────────────────┐
         │              AUTH               │
         │        (Users, Roles)           │
         └───────────────┬─────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
    ┌───────┐       ┌─────────┐      ┌────────┐
    │ SHOP  │◄─────►│ PRODUCT │      │ WALLET │
    └───┬───┘       └────┬────┘      └────┬───┘
        │                │                │
        │                ▼                │
        │           ┌─────────┐           │
        └──────────►│  ORDER  │◄──────────┘
                    └────┬────┘
                         │
                    ┌────▼────┐
                    │ ESCROW  │
                    │(DISPUTE)│
                    └─────────┘
```

## V1 → V2 Feature Map
| V1 Feature | V1 Doc | V2 Context | V2 Design |
|------------|--------|------------|-----------|
| Auth/2FA | 01-authentication.md | context/auth.md | - |
| Roles | 02-user-roles.md | context/auth.md | - |
| Products | 04-products-inventory.md | context/product.md | TBD |
| Wallet | 06-wallet-payment.md | context/wallet.md | docs/v2/01-wallet-system-design.md |
| Orders | all.md | context/order.md | TBD |

## Workflow Reminders

### Before Starting Task
```
1. Which module? → Read context/{module}.md
2. New feature? → Check V1 doc first
3. Brainstorm? → Use skills/brainstorm.md
4. Write docs? → Use skills/write-docs.md
```

### After Completing Task
```
1. Update context/{module}.md
2. Add to lessons.md if learned something
3. Check refs still valid
```
