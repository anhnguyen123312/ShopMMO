# Skill: Research & Document

## When to Use
Khi cần research workflow, library, hoặc best practice

## Process

### 1. Define Scope
```
Topic: [what to research]
Goal: [what to achieve]
Time: [estimate]
```

### 2. Search Sources
Priority order:
1. Official docs
2. GitHub examples
3. Community discussions

### 3. Document Format
```markdown
# Research: {topic}

## Summary
1-2 sentences what this is

## Key Findings
- Point 1
- Point 2

## How to Use
[minimal example or flow]

## Refs
- [link1]
- [link2]

## Decision
Use/Don't use và lý do
```

## Output Rules
- ✅ Max 1 page
- ✅ Bullet points
- ✅ Code snippets chỉ khi cần define pattern
- ❌ Không copy paste full docs
- ❌ Không dài dòng giải thích

## Save Location
```
.claude/research/{topic}.md
```

## Example Topics
- Payment gateway integration
- MongoDB transaction patterns
- Redis caching strategies
- TOTP 2FA implementation
