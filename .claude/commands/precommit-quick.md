---
description: Quick pre-commit workflow (checks + commit message, no AI review)
allowed-tools: []
---

# Quick Pre-Commit Workflow ⚡

Fast pre-commit workflow without AI code review. Perfect for:
- Minor changes (typos, formatting)
- Documentation updates
- Quick fixes
- When you're confident in your changes

## Step 1: Quality Checks ⚙️

Run `/git:check` to verify code quality:
- Lint checks
- Build verification
- Test suite

**Critical**: Stop if any checks fail. Fix issues before proceeding.

## Step 2: Generate Commit Message 📝

Run `/git:commit-msg` to generate conventional commit message.

The message will:
- Follow conventional commits format
- Include detailed description
- Be ready to copy/paste

## Step 3: Ready to Commit ✅

**To commit:**
```bash
git commit -m "..."  # Use the generated message
```

**Or ask me to:**
- Revise the commit message
- Run full `/precommit` with AI review
- Make additional changes

---

**Workflow Summary:**
1. ⚙️  Quality checks
2. 📝 Generate commit message
3. ✅ Commit

**Pro tip**: Use `/precommit` (full version) for important logic changes, refactoring, or new features.
