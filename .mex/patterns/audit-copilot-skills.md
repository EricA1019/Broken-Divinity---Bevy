---
name: audit-copilot-skills
description: Running a maintenance audit on Copilot customization files (SKILL.md, instructions, MCP config) — broken refs, overlap, bloat, portability
triggers:
  - "skill audit"
  - "skill maintenance"
  - "customization review"
  - "agent-customization-local"
edges:
  - target: context/conventions.md
    condition: when verifying skill content matches project conventions
last_updated: 2026-04-11
---

# Audit Copilot Skills

## Context

Skills live at `~/.copilot/skills/<name>/SKILL.md` (user-level, not in-repo). The `agent-customization-local` skill owns the maintenance checklist; invoke it to get the full checklist reference.

Key locations:
- Skills: `~/.copilot/skills/`
- MCP config: `~/.copilot/mcp-config.json`
- Maintenance checklist: `~/.copilot/skills/skill-creator/references/customization-maintenance-checklist.md`

## Steps

1. **Invoke `agent-customization-local`** to load the maintenance checklist.
2. **Scan all SKILL.md files** — count lines, list all links (relative and absolute).
3. **Check broken references** — verify every link target exists. Distinguish real broken links from illustrative examples inside code blocks.
4. **Check overlap** — compare adjacent skill descriptions for ambiguous boundaries. Look for trigger phrases that could match multiple skills.
5. **Check bloat** — identify skills >100 lines without `references/` directories. Rank by line count.
6. **Check portability** — grep for absolute machine paths. All should be relative.
7. **Prioritize fixes** — broken refs first, then overlap, then extract references for top N bloated skills.
8. **Extract references** — move large sections (formulas, inventories, matrices, code examples) into `references/` subdirectories. Keep SKILL.md for workflow and decision guidance only. Link back with relative paths.
9. **Verify** — confirm all new reference links resolve.

## Gotchas

- **Code-block links are not broken**: The skill-creator SKILL.md has example file references inside markdown code blocks (FORMS.md, REFERENCE.md, etc.). These are illustrative, not real links. Don't flag them.
- **Relative links from `~/.copilot/skills/` to project source don't work as clickable links** — use backtick code references for source paths instead of markdown links.
- **Progressive disclosure must preserve all content** — extracting to references/ means moving content, not deleting it. Verify line counts add up.
- **MCP env vars before `--`**: When adding MCP servers via `copilot mcp add`, `--env K=V` must come BEFORE the `--` separator. Anything after `--` is the command+args.

## Verify

- [ ] All SKILL.md files with links: every target resolves (skip code-block examples)
- [ ] No two skills have overlapping trigger descriptions without explicit boundary phrases
- [ ] Top 5 bloated skills have `references/` directories with linked content
- [ ] No absolute machine paths in any SKILL.md
- [ ] ROUTER.md project state updated with audit results

## Debug

- If a reference link fails: check for typos, check if the file was extracted to a different name, check if the `references/` directory was created.
- If skills still overlap: add explicit "For X, use skill-Y instead" boundary phrases to both skills' descriptions.
- If MCP server fails to connect: run `copilot mcp list` to verify config, check that the binary path exists, test the command manually.

## Update Scaffold

- [ ] Update `.mex/ROUTER.md` "Current Project State" if what's working/not built has changed
- [ ] Update any `.mex/context/` files that are now out of date
- [ ] If this is a new task type without a pattern, create one in `.mex/patterns/` and add to `INDEX.md`
