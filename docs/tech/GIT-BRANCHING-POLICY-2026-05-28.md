# Git Branching and Repository Health Policy

Date: 2026-05-28
Status: Active
Owner: Project maintainers

## 1) Purpose

This policy defines how code moves from feature work to integration to production.
Goals:
- Keep `main` stable and release-ready.
- Keep `dev` as the integration branch for active work.
- Make feature delivery predictable, reviewable, and reversible.
- Reduce regressions and merge debt as team size grows.

## 2) Branch Roles

### `main` (production branch)
- Source of truth for stable releases.
- Must always be deployable.
- No direct pushes.
- Only accepts pull requests from `dev` or emergency `hotfix/*` branches.

### `dev` (integration branch)
- Primary working branch for upcoming release content.
- Integrates completed feature branches.
- No direct pushes.
- Only accepts pull requests from `feature/*`, `fix/*`, `chore/*`, `docs/*`, `refactor/*`, `test/*`, or `hotfix/*` as needed.

### Feature and support branches (short-lived)
- Created from latest `dev`.
- Merged back into `dev` when done.
- Deleted after merge.

## 3) Branch Naming Convention

Use lowercase kebab-case:
- `feature/<area>-<summary>`
- `fix/<area>-<summary>`
- `chore/<area>-<summary>`
- `docs/<area>-<summary>`
- `refactor/<area>-<summary>`
- `test/<area>-<summary>`
- `hotfix/<area>-<summary>`
- `release/<yyyy-mm-dd>-<tag>` (optional stabilization branch)

Examples:
- `feature/ui-runtime-inventory-grid`
- `fix/save-load-colony-target-state`
- `hotfix/crash-main-menu-startup`

## 4) Standard Delivery Flow

1. Sync `dev` locally.
2. Create a branch from `dev`.
3. Commit in small, reviewable slices.
4. Open PR into `dev`.
5. Pass all required checks.
6. Obtain required review approvals.
7. Merge PR into `dev`.
8. Delete source branch.

Promotion to production:
1. Freeze `dev` for release validation.
2. Open PR `dev -> main`.
3. Run full release gate.
4. Tag release on merge commit in `main`.

## 5) PR and Review Policy

### Required for every PR
- Clear title and description:
  - Problem
  - Change made
  - Risk/regression surface
  - Test evidence
- Link to relevant plan/handoff/task docs where available.
- No unrelated file churn in the same PR.

### Review rules
- At least 1 approval for normal changes.
- At least 2 approvals for risky changes:
  - save/load format changes
  - state machine transitions
  - core runtime boot flow
  - broad refactors across modules
- Author cannot self-approve.

### PR size guidance
- Target under ~500 changed lines when practical.
- If larger, split by concern (behavior, refactor, tests, docs).

## 6) Merge Strategy

- Use `Squash and merge` for most feature/support PRs into `dev` to keep history concise.
- Use `Merge commit` for `dev -> main` promotions to preserve integration context.
- Avoid rebasing shared branches after PR review starts.
- Never force-push to `main` or `dev`.

## 7) Quality Gates

Minimum gate for PRs into `dev`:
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`

Required gate for `dev -> main`:
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo build --release`
- Any required visual/runtime validation battery defined in current tech docs.

If a gate fails, PR cannot merge.

## 8) Release and Hotfix Policy

### Normal release
- All normal work merges to `dev` first.
- Release candidate is validated from `dev`.
- Only after passing release gate is `dev` promoted to `main`.

### Emergency hotfix
Use only for production-breaking issues.

Flow:
1. Branch from `main`: `hotfix/<area>-<summary>`.
2. Implement minimal fix + focused tests.
3. PR to `main` with expedited review.
4. After merge to `main`, immediately forward-merge or cherry-pick into `dev`.
5. Tag patch release.

Rule: No hotfix is complete until both `main` and `dev` contain the fix.

## 9) Protection Rules (Repository Settings)

Configure branch protection for `main` and `dev`:
- Require pull request before merge.
- Require required status checks to pass.
- Require conversation resolution before merge.
- Require linear history on feature branches (optional), but do not require on `main` if using merge commits for release promotions.
- Restrict who can push directly (ideally nobody except automation if needed).
- Disable force pushes.
- Disable branch deletion for protected branches.

## 10) Commit Hygiene

- Prefer conventional commit style:
  - `feat: ...`
  - `fix: ...`
  - `chore: ...`
  - `docs: ...`
  - `refactor: ...`
  - `test: ...`
- One concern per commit when practical.
- Include tests in same PR as behavior change.
- Do not mix formatting-only changes with behavioral changes unless required.

## 11) Documentation and Traceability

For non-trivial changes, update:
- `CHANGELOG.md` (release-facing impact)
- Relevant plan/handoff docs in `docs/tech/` (execution trace)
- Any architecture or policy docs affected by behavior changes

PRs should make it easy to answer:
- What changed?
- Why did it change?
- How was it validated?

## 12) Anti-Patterns (Do Not Do)

- Direct commit to `main`.
- Long-lived feature branches that drift for weeks.
- Merging failing PRs "to unblock".
- Mixing unrelated fixes in one PR.
- Hotfixing `main` without backporting to `dev`.

## 13) Operational Cadence

- Rebase or merge `dev` into active feature branches at least every 1-2 days.
- Keep PR review turnaround under 24 hours when possible.
- Run a weekly branch cleanup:
  - delete merged remote branches
  - close stale draft PRs
  - revalidate open long-running branches

## 14) Quick Command Reference

Create a feature branch from `dev`:

```bash
git checkout dev
git pull
git checkout -b feature/<area>-<summary>
```

Update branch with latest `dev`:

```bash
git checkout dev
git pull
git checkout feature/<area>-<summary>
git merge dev
```

Prepare release promotion:

```bash
git checkout main
git pull
git checkout dev
git pull
# open PR: dev -> main
```

## 15) Enforcement Notes

This policy is only effective when encoded in branch protection and CI required checks.
If tooling and policy diverge, update tooling first, then update this document.
