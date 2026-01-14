# Baseline Strategy Guide

This guide covers best practices for managing visual regression baselines in team environments.

## What Are Baselines?

Baselines are the "expected" screenshots that new screenshots are compared against. They represent the approved visual state of your application.

```
.pixelguard/
├── baseline/           # Approved screenshots (committed to git)
│   ├── button--primary.png
│   ├── card--default.png
│   └── modal--open.png
├── current/            # Latest captures (gitignored)
└── diff/               # Difference images (gitignored)
```

## Core Principles

### 1. Baselines Are Source of Truth

Baselines should always represent the **intended** visual state, not just the current state. Before updating baselines, verify the changes are intentional.

### 2. Baselines Are Code

Treat baseline updates like code changes:
- Review them in pull requests
- Require approval before merging
- Include meaningful commit messages

### 3. Small, Frequent Updates

Update baselines frequently in small batches rather than large bulk updates. This makes reviews easier and history more useful.

## Workflow Strategies

### Strategy 1: Update Per PR (Recommended)

Update baselines in the same PR that causes visual changes.

**Workflow:**
1. Make code changes
2. Run `pixelguard test` to see diffs
3. Review changes in the HTML report
4. If changes are intentional, run `pixelguard test --update`
5. Commit baseline changes with the code changes
6. Create PR with both code and baseline updates

**Pros:**
- Changes and their visual impact are reviewed together
- Clear history of why baselines changed
- Easy to revert if needed

**Cons:**
- Requires running tests locally before pushing
- Can slow down development workflow

### Strategy 2: Dedicated Baseline PRs

Update baselines in separate PRs after code changes land.

**Workflow:**
1. Make code changes in PR #1
2. Merge PR #1 (CI may fail visual tests - that's expected)
3. Create PR #2 to update baselines
4. Review and merge baseline updates

**Pros:**
- Faster code iteration
- Baseline reviews are focused

**Cons:**
- Visual changes may be forgotten
- History is split across PRs
- CI may be red between PRs

### Strategy 3: Scheduled Updates

Update baselines on a schedule (e.g., weekly, per sprint).

**Workflow:**
1. Accumulate visual changes during development
2. At scheduled time, run `pixelguard test --update`
3. Create PR with all baseline updates
4. Review and merge

**Pros:**
- Minimal impact on daily workflow
- Batch review efficiency

**Cons:**
- Large, harder-to-review PRs
- Longer feedback loops
- Risk of approving unintentional changes

## Git Configuration

### Basic Setup

Add to `.gitignore`:
```gitignore
# Pixelguard
.pixelguard/current/
.pixelguard/diff/
.pixelguard/report.html
```

Commit baselines:
```bash
git add .pixelguard/baseline/
git commit -m "feat: add initial visual baselines"
```

### Git LFS for Large Projects

For projects with many screenshots, use Git LFS to avoid bloating the repository.

**Setup:**
```bash
# Install Git LFS
git lfs install

# Track baseline images
git lfs track ".pixelguard/baseline/*.png"

# Commit the tracking file
git add .gitattributes
git commit -m "chore: track baselines with Git LFS"
```

**Benefits:**
- Faster clone/fetch for large repos
- Better handling of binary files
- Reduced repo size on disk

**Considerations:**
- Requires Git LFS installed on all machines
- CI runners need LFS support
- Storage limits on some Git hosts

### .gitattributes for Better Diffs

Add to `.gitattributes`:
```gitattributes
# Treat PNGs as binary (no text diff)
*.png binary

# Or with LFS:
.pixelguard/baseline/*.png filter=lfs diff=lfs merge=lfs -text
```

## Branch Strategies

### Main Branch Baselines (Recommended)

Keep baselines on the main branch, update in PRs.

```
main ─────●─────●─────●─────●─────●
          │     │     │     │
          │     └ PR: Update button styles
          │       └ Includes baseline updates
          │
          └ PR: Add new card component
            └ Includes new baseline
```

**Pros:**
- Simple workflow
- Baselines always match main branch code

### Feature Branch Baselines

Allow baselines to diverge on feature branches, resolve on merge.

```
main ─────●─────────────────●─────
          │                 │
feature ──┴──●──●──●──●─────┘
             │  │
             │  └ Local baseline updates
             │
             └ Start from main's baselines
```

**Conflict Resolution:**
When merging feature branches with baseline changes:
1. Accept all changes from the feature branch
2. Run `pixelguard test` to verify
3. Fix any unexpected differences

## CI/CD Integration

### Basic CI Workflow

```yaml
# .github/workflows/visual-regression.yml
name: Visual Regression

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          lfs: true  # If using Git LFS

      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - run: npm ci
      - run: npm run build-storybook
      - run: npx serve storybook-static -p 6006 &
      - run: npx wait-on http://localhost:6006

      - run: npx pixelguard test --ci

      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: visual-regression-report
          path: .pixelguard/
```

### Handling CI Failures

When CI fails visual tests:

1. **Download the report artifact**
2. **Review the differences**
3. **If intentional:** Update baselines locally, push
4. **If unintentional:** Fix the code, push

### Auto-Update in CI (Caution)

Some teams auto-update baselines in CI. Use with care:

```yaml
- name: Update baselines if changed
  if: github.ref == 'refs/heads/main'
  run: |
    npx pixelguard test --update
    git config user.name "CI Bot"
    git config user.email "ci@example.com"
    git add .pixelguard/baseline/
    git diff --staged --quiet || git commit -m "chore: update visual baselines"
    git push
```

**Risks:**
- May commit unintentional changes
- Loses review opportunity
- Can hide bugs

## Team Workflows

### For Small Teams (1-5 developers)

- Update baselines per PR
- Review changes in PR diff view
- Single approval required

### For Medium Teams (5-20 developers)

- Designate visual test owners
- Require owner approval for baseline changes
- Use PR labels to flag baseline updates
- Consider scheduled review meetings

### For Large Teams (20+ developers)

- Create CODEOWNERS for `.pixelguard/`
- Require 2+ approvals for baseline changes
- Use automated commenting to highlight visual changes
- Consider component-based baseline ownership

## Reviewing Baseline Changes

### What to Look For

1. **Intentional changes** - Do they match the PR description?
2. **Unintentional changes** - Any unexpected side effects?
3. **Rendering artifacts** - Anti-aliasing, font rendering differences?
4. **Missing changes** - Should more shots be affected?

### Using the HTML Report

The report at `.pixelguard/report.html` provides:
- Side-by-side comparison
- Diff overlay visualization
- Comparison slider
- Zoom for detailed inspection

### GitHub PR Review Tips

```markdown
## Visual Changes

This PR updates the button component styling.

### Expected Changes:
- `button--primary.png` - New hover state
- `button--secondary.png` - Updated border radius

### How to Review:
1. Download the report artifact
2. Open `.pixelguard/report.html`
3. Verify changes match expectations
```

## Troubleshooting

### Flaky Baselines

If baselines keep changing unexpectedly:

1. **Add delays** for animations:
   ```json
   { "name": "modal--animated", "delay": 1000 }
   ```

2. **Wait for selectors** for async content:
   ```json
   { "name": "data-table", "waitFor": ".loaded" }
   ```

3. **Increase threshold** for anti-aliasing:
   ```json
   { "threshold": 0.05 }
   ```

4. **Mock dynamic content** (dates, random data)

### Large Baseline Updates

For bulk updates (e.g., font change, theme update):

1. Create a dedicated PR with clear description
2. List all affected components
3. Consider splitting into smaller PRs by component
4. Request review from multiple team members

### Merge Conflicts in Baselines

Binary files don't merge well. When conflicts occur:

1. Accept one version (usually the target branch)
2. Run `pixelguard test`
3. Update if needed: `pixelguard test --update`
4. Commit the resolution

## Best Practices Summary

| Do | Don't |
|----|-------|
| Review baselines like code | Blindly accept all changes |
| Include context in commits | Use vague commit messages |
| Update baselines with related code | Let baselines drift |
| Use Git LFS for large projects | Commit hundreds of PNGs directly |
| Mock dynamic/random content | Expect stability with live data |
| Set appropriate thresholds | Use 0% threshold (too strict) |
