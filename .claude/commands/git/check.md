---
description: Run nx quality checks (lint, build, test) - uses affected when possible
allowed-tools: Bash(bun nx:*), Bash(git diff:*)
---

# Quality Checks

Smart quality checks using nx affected when possible to save time.

## Step 1: Determine Affected Projects 🔍

Check what's changed:

!`git diff --cached --name-only`

Count affected projects:

!`bun nx affected:graph --base=HEAD~1 --head=HEAD 2>/dev/null || echo "all"`

**Strategy:**
- If 1-5 projects affected → Run checks only on affected projects (fast ⚡)
- If 6+ projects or can't determine → Run all checks (comprehensive 🔍)
- If no staged changes → Run all checks (default behavior)

## Step 2: Run Quality Checks ⚙️

### Option A: Affected Only (when applicable)
```bash
bun nx affected -t lint --parallel 16
bun nx affected -t build --parallel 16
bun nx affected -t test --parallel 16
```

**Benefits:**
- Faster execution (only check what changed)
- Good for pre-commit workflow
- Reduces CI/CD time

### Option B: All Projects (when needed)
```bash
bun nx run-many -t lint --parallel 16
bun nx run-many -t build --parallel 16
bun nx run-many -t test --parallel 16
```

**Use when:**
- Many projects affected
- Changes to shared code/configs
- Pre-push/pre-release checks
- Can't determine affected projects

## Step 3: Run Checks

Execute the appropriate strategy from Step 1:

1. **Lint**: Check code quality and style
2. **Build**: Verify project builds successfully
3. **Test**: Run test suites

## Step 4: Report 📊

After running all checks, provide a summary:

**Show:**
- ✅/❌ Lint results (errors, warnings, fixable issues)
- ✅/❌ Build status (success/failure)
- ✅/❌ Test results (passed/failed/skipped counts)
- ⚡ Which strategy was used (affected vs all)
- 📦 How many projects were checked

**If checks fail:**
- List specific errors by project
- Suggest fixes (e.g., `cargo clippy --fix`, `bun nx lint --fix`)
- Show which projects failed

**Example summary:**
```
⚡ Affected Strategy: 3 projects checked (vs 17 total)
✅ Lint: Passed (2 warnings in domain_users)
✅ Build: Passed
✅ Test: Passed (23 tests)

Time saved: ~40 seconds
```

---

**Pro tips:**
- Use affected for pre-commit (fast iteration)
- Use all projects for pre-push/CI (thorough)
- Run `bun nx affected:graph` to visualize dependencies
