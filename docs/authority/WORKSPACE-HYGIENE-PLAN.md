# Workspace Hygiene Plan — Broken Divinity

**Created:** 2026-08-06
**Status:** Draft — awaiting owner review and approval
**Scope:** File organization, documentation conventions, ongoing maintenance

---

## 1. Current Problems

### 1.1 Root-level clutter
Eight loose files at the workspace root are either stale, auto-generated, or
misplaced:

| File | Problem | Action |
|---|---|---|
| `detailed_analysis.json` | Unknown origin, no consumer | Archive or delete |
| `results.json` | Unknown origin, no consumer | Archive or delete |
| `testing.log` | Auto-generated, in .gitignore? | Add to .gitignore, delete |
| `PLAN-2026-05-26-ALPHA-READINESS.md` | 2.5 months stale, superseded | → `docs/archive/` |
| `PROTOTYPE_ANALYSIS_REPORT.md` | One-off report, stale | → `docs/archive/` |
| `UX_PLAYTEST_REPORT.md` | Useful historical UX data | → `docs/` |
| `KNOWN_ISSUES.md` | Self-declared "historical only" | → `docs/archive/` |

Files that stay at root: `AGENTS.md`, `GDD.md`, `Kernel.md`, `Kernel-direction.md`,
`CHANGELOG.md`, `README.md`, `Cargo.toml`, `Cargo.lock`, `justfile`.

### 1.2 metrics/ directory is misnamed
Contains playtest screenshots (`.png`), markdown notes, and a SQLite database.
No actual metrics collection happens here. The Python metrics framework in
`testing/` also writes here. Two competing "metrics" systems.

### 1.3 graphify-out/ is auto-generated
14 JSON files plus an HTML report. These are build artifacts from the graphify
skill, not source files. Should be in `.gitignore` or generated to a
`.artifacts/` directory.

### 1.4 testing/ mixes Python and Rust
- Python testing framework (7 `.py` files, `requirements.txt`) — 37.5% success
  rate at last measurement (May 2026). Unclear if maintained.
- Rust contract registry artifacts (`.ron` files, baselines, manifests)
- Markdown reports and matrices
- SQLite database (`metrics.db`)

### 1.5 docs/ mixes active plans with working handoff files
`UI9-C-CONTEXT-CANDIDATE-HANDOFF-*` files are implementation work-products,
not permanent documentation. They live alongside authority docs like
`ARCHITECTURE_GUARDRAILS.md` and `DECISIONS-TO-LOCK.md`.

### 1.6 No retirement process
Documents accumulate indefinitely. Nothing forces stale docs to be reviewed
or archived. The `.mex/ROUTER.md` was 3.5 months stale before today.

---

## 2. Target Directory Layout

```
broken-divinity/
├── AGENTS.md                 # Development contract (stays)
├── GDD.md                    # Product design authority (stays)
├── Kernel.md                 # Technical authority (stays)
├── Kernel-direction.md       # Appendix to Kernel (stays)
├── CHANGELOG.md              # Release notes (stays)
├── README.md                 # Project overview (stays)
├── Cargo.toml                # Workspace (stays)
├── Cargo.lock                # Locked deps (stays)
├── justfile                  # Task runner (stays)
├── rust-toolchain.toml       # TO CREATE — pin toolchain
│
├── .artifacts/               # TO CREATE — generated output, gitignored
│   └── graphify/             #   graphify-out/ moves here
│
├── config/                   # User-facing config (stays)
│   └── default.toml
│
├── content/                  # Game data — RON files (stays)
│
├── crates/                   # Active Rust workspace (stays)
│
├── docs/
│   ├── README.md             # Documentation index (stays)
│   ├── active/               # TO CREATE — current implementation plans
│   │   ├── FOUNDATION-TEST-AND-UX-HARDENING-PLAN.md
│   │   ├── FOUNDATION-BASIC-COLONY-LOOP-PLAN.md
│   │   └── FOUNDATION-UI-IMPROVEMENT-PLAN.md
│   ├── authority/            # TO CREATE — locked decisions and guardrails
│   │   ├── DECISIONS-TO-LOCK.md
│   │   ├── ARCHITECTURE_GUARDRAILS.md
│   │   ├── AUTHORITATIVE-TESTING-STANDARD-AND-MIGRATION-PLAN.md
│   │   ├── DEPENDENCY_MATRIX.md
│   │   ├── DOCUMENT-INVENTORY.md
│   │   ├── MVP-SCENARIO.md
│   │   └── MIGRATION-AND-DEPRECATION.md
│   ├── reference/            # TO CREATE — design docs, lore, gameplay specs
│   │   ├── GAME-STATUS-2026-08-01.md
│   │   ├── PHASE_EXIT_CRITERIA.md
│   │   ├── FOUNDATION-UI-STYLE-MOCKUPS.md
│   │   ├── UX_PLAYTEST_REPORT.md
│   │   ├── lore/
│   │   ├── gameplay/
│   │   └── mockups/
│   ├── decisions/            # Historical decision log (stays)
│   ├── tech/                 # Technical references — ALL marked ARCHIVED
│   │   ├── architecture.md
│   │   └── ui-design.md
│   ├── archive/              # Completed plans + retired docs (stays)
│   │   ├── FOUNDATION-RECOVERY-PLAN.md
│   │   ├── FOUNDATION-STABILIZATION-PLAN.md
│   │   ├── FOUNDATION-MVP-CORRECTION-PLAN.md
│   │   ├── PLAN-2026-05-26-ALPHA-READINESS.md
│   │   ├── PROTOTYPE_ANALYSIS_REPORT.md
│   │   ├── KNOWN_ISSUES.md
│   │   ├── DEV-PLAN-LEGACY.md
│   │   └── GDD-LEGACY.md
│   └── handoff/              # TO CREATE — active UI9-C working artifacts
│       ├── UI9-C-CONTEXT-CANDIDATE-HANDOFF-PROMPT-v2.md
│       ├── UI9-C-CONTEXT-CANDIDATE-HANDOFF-PROMPT.md
│       ├── UI9-C-CONTEXT-CANDIDATE-HANDOFF-BODY-v3.md
│       └── UI9-C-CONTEXT-CANDIDATE-HANDOFF-BODY-v4.md
│
├── legacy/                   # Archived prototype code (stays)
│   ├── README.md
│   └── src/
│
├── scripts/                  # Build and test scripts (stays)
│
├── testing/
│   ├── README.md             # (stays)
│   ├── foundation-contracts.ron       # Authority — stays
│   ├── FOUNDATION-TEST-EVIDENCE.md    # Authority — stays
│   ├── FOUNDATION-REQUIREMENT-MAP.md  # Authority — stays
│   ├── VISUAL-ACCEPTANCE-MATRIX.md    # Authority — stays
│   ├── COMPREHENSIVE_TEST_REPORT.md   # Historical — archive?
│   ├── allowed-ignored-tests.txt      # Authority — stays
│   ├── WEEK2_IMPLEMENTATION_SUMMARY.md # Historical — archive?
│   ├── UI9-C-CONTEXT-BASELINE-*.ron   # Active handoff artifacts
│   ├── UI9-C-CONTEXT-CANDIDATE-HANDOFF-*.ron  # Active handoff artifacts
│   └── py-legacy/            # TO CREATE — isolate Python framework
│       ├── test_framework.py
│       ├── metrics_collector.py
│       ├── balance_analytics.py
│       └── ...
│
├── metrics/                  # REPURPOSE or ARCHIVE
│   └── README.md             # TO CREATE — explain what lives here now
│
└── asset/                    # (stays)
```

---

## 3. Hygiene Rules (Ongoing)

### 3.1 Where to put new files

| What you're creating | Where it goes |
|---|---|
| Active implementation plan | `docs/active/` |
| Authority/guardrail doc | `docs/authority/` |
| Design reference (lore, gameplay) | `docs/reference/` |
| One-off historical report | `docs/archive/` |
| Implementation handoff prompt | `docs/handoff/` |
| Playtest screenshot | `metrics/` (or delete after report) |
| Generated artifact (build output) | `.artifacts/` (gitignored) |
| Test baseline or manifest | `testing/` |

### 3.2 Retirement checklist
When a plan or document is superseded:
1. Move it to `docs/archive/`
2. Update `docs/DOCUMENT-INVENTORY.md` to remove it from the active list
3. If it was the active implementation authority, ensure the new plan is
   recorded in `DOCUMENT-INVENTORY.md` and `README.md`

### 3.3 Monthly hygiene audit (15 minutes)
On the first of each month:
1. Check for new loose files at workspace root — archive or delete
2. Check `docs/active/` — any plan that hasn't been touched in 30 days?
   Mark it for owner review
3. Check `.mex/ROUTER.md` — is the project state section accurate?
4. Run `cargo tree --workspace --depth 1` — any new unused dependencies?
5. Check `testing/` for new `.log` files — add to `.gitignore` or delete

### 3.4 Files that should be in .gitignore
```
# Generated artifacts
.artifacts/
graphify-out/
testing.log
metrics/*.db

# Python virtualenv
testing/__pycache__/
testing/*.pyc
```

---

## 4. Implementation Phases

### Phase A — Safe moves (no code impact, no broken links)
**Estimated: 15 minutes**

1. Create `.artifacts/` directory, add to `.gitignore`
2. Move `graphify-out/` → `.artifacts/graphify/`
3. Archive root-level stale files:
   - `PLAN-2026-05-26-ALPHA-READINESS.md` → `docs/archive/`
   - `PROTOTYPE_ANALYSIS_REPORT.md` → `docs/archive/`
   - `KNOWN_ISSUES.md` → `docs/archive/`
   - `UX_PLAYTEST_REPORT.md` → `docs/reference/` (or `docs/` if we don't split yet)
4. Delete `detailed_analysis.json`, `results.json` (if confirmed disposable)
5. Delete `testing.log` and add to `.gitignore`
6. Move Python testing files to `testing/py-legacy/` with a README

### Phase B — Docs subdirectory split (creates new folders)
**Estimated: 30 minutes — READ CAREFULLY before executing**

1. Create `docs/active/`, `docs/authority/`, `docs/reference/`, `docs/handoff/`
2. Move files into their buckets per the layout above
3. Update `docs/DOCUMENT-INVENTORY.md` with new paths
4. Update `docs/README.md` with new structure
5. Update `README.md` authority section with new paths
6. Update `.mex/ROUTER.md` path references
7. Update `AGENTS.md` path references to authority docs if needed
8. Run `scripts/test-gate.sh` to verify no protected-file hashes broke

### Phase C — Tooling and prevention
**Estimated: 20 minutes**

1. Create `rust-toolchain.toml` pinning the current stable toolchain
2. Create `metrics/README.md` explaining what belongs there
3. Create `testing/py-legacy/README.md` explaining the Python framework's status
4. Add a `just hygiene` command that runs the monthly audit checklist
5. Update `.gitignore` with entries from section 3.4

### Phase D — Optional cleanup (lower priority)
- Remove unused workspace dependencies from `Cargo.toml`:
  `bevy_time`, `color-eyre`, `insta`, `schemars`
- Fully rewrite `.mex/ROUTER.md` for the Bevy 0.18 + Ratatui architecture
- Delete the Python testing framework if confirmed abandoned

---

## 5. Decision Points (for owner)

Before executing, the owner must decide:

1. **docs/ subdirectory split?** The plan proposes `active/`, `authority/`,
   `reference/`, `handoff/`. Simpler alternative: keep all docs flat and just
   use filename prefixes. Which do you prefer?

2. **graphify-out/ disposition?** Move to `.artifacts/` (gitignored) or keep
   visible as committed project data?

3. **Python testing framework?** Isolate in `testing/py-legacy/` with a note,
   or delete entirely?

4. **metrics/ repurpose?** Keep as playtest-screenshot storage, or archive
   screenshots and repurpose for actual metrics?

5. **Phase B worth it?** The subdirectory split is the most disruptive change
   (updates many cross-references). Is the clarity worth the churn?
