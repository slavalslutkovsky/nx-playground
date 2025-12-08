---
description: Full pre-commit workflow (check, review, commit message)
allowed-tools: []
---

# Pre-Commit Workflow

Complete pre-commit workflow with quality checks, optional AI review, and commit message generation.

## Step 1: Quality Checks ⚙️

Run `/git:check` to verify code quality:
- Lint checks
- Build verification
- Test suite

**Critical**: Stop if any checks fail. Fix issues before proceeding.

## Step 2: AI Code Review (Optional) 🤖

Ask user if they want AI review:
- Skip for minor changes (typos, formatting)
- Recommended for logic changes, refactoring, new features

If user wants AI review:

**IMPORTANT**: Run both reviews in parallel for faster execution:
- `/git:gemini-review` for Gemini analysis
- `/git:coderabbit-review` for CodeRabbit analysis

Use SlashCommand tool twice in a single message to run them concurrently.

**Note**: Parallel execution saves time (~30-40 seconds vs 60+ seconds sequential).

## Step 3: Review Analysis 🔍

If AI reviews were run:

### Critical Issues Found:
- 🔴 **STOP** - Do not generate commit message
- List all critical issues by category
- Provide specific fix suggestions
- User should fix issues and run `/precommit` again

### Only Warnings/Suggestions:
- 🟡 Show warnings for user awareness
- Proceed to commit message generation
- User can address in follow-up commit

### All Clear:
- 🟢 Proceed to commit message generation

If no AI reviews:
- Proceed to commit message generation

## Step 4: Generate Commit Message 📝

Run `/git:commit-msg` to generate conventional commit message.

The message will:
- Follow conventional commits format
- Include detailed description
- Be ready to copy/paste

## Step 5: Final Instructions ✅

Remind user:

**To commit:**
```bash
git commit -m "..."  # Use the generated message
```

**Or ask me to:**
- Revise the commit message
- Make additional changes
- Fix any remaining issues

---

**Workflow Summary:**
1. ⚙️  Quality checks (required)
2. 🤖 AI review (optional, ask user)
3. 🔍 Analyze results
4. 📝 Generate commit message
5. ✅ Ready to commit

**Pro tip**: For quick commits, skip AI review. For important changes, always review.