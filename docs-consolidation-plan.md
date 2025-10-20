# Docs Consolidation Plan

## Doc Map

| Path | Purpose | Keep/Merge/Delete | New Destination | Owner | Conflicts / Duplication |
| --- | --- | --- | --- | --- | --- |
| README.md | Marketing-heavy pitch with unrealistic feature list | Rewrite | README.md | core | Conflicts with actual feature scope described in STATUS.md |
| STATUS.md | High-level progress tracker | Keep | docs/changelog.md (summary link) | core | Overlaps with CURRENT_ANNOTATION_STATUS.md |
| CURRENT_ANNOTATION_STATUS.md | Token/annotation status notes | Merge | docs/reference/grammar.md | compiler | Duplicates README claims |
| CODEGEN_PROGRESS_SUMMARY.md | Notes on codegen status | Merge | docs/guide/compiler-overview.md | compiler | Overlaps with STDLIB_COMPLETION_SUMMARY.md |
| FULLSTACK_GUIDE.md | Out-of-date marketing deck | Archive | docs/_archive/FULLSTACK_GUIDE.md | marketing | Conflicts with new guide |
| DEPLOYMENT_SUMMARY.md | Deployment brainstorm | Archive | docs/_archive/DEPLOYMENT_SUMMARY.md | core | Superseded by future deploy docs |
| README in docs/README.md | Legacy docs index | Replace | docs/guide/index.md | core | Duplicates root README |
| docs/GETTING_STARTED.md | Old quickstart | Merge | docs/guide/quickstart.md | core | Conflicts with docs/guides/QUICKSTART.md |
| docs/api/PACKAGE_MANIFEST_SPEC.md | Early manifest spec | Archive | docs/_archive/api/PACKAGE_MANIFEST_SPEC.md | compiler | Superseded by CLI reference |
| docs/api/REGISTRY_API_SPEC.md | Registry API draft | Archive | docs/_archive/api/REGISTRY_API_SPEC.md | infra | Out of scope for MVP |
| docs/architecture/* | Future architecture plans | Archive | docs/_archive/architecture/... | core | Duplicates engineering docs |
| docs/archived/* | Already archived | Re-index | docs/_archive/legacy/... | core | Needs index entry |
| docs/community/INITIATIVES.md | Community roadmap | Archive | docs/_archive/community/INITIATIVES.md | marketing | Outdated |
| docs/engineering/* | Launch plans | Archive | docs/_archive/engineering/... | core | Conflicts with new changelog |
| docs/events/CONFERENCE_TALK_PLAN.md | Talk outline | Archive | docs/_archive/events/CONFERENCE_TALK_PLAN.md | marketing | Marketing duplicate |
| docs/guides/* | Misc guides | Merge | docs/guide/*.md | core | Overlaps with GETTING_STARTED |
| docs/marketing/* | Marketing series drafts | Archive | docs/_archive/marketing/... | marketing | Outdated |
| docs/tutorials/ADVANCED_TUTORIALS.md | Placeholder tutorials | Archive | docs/_archive/tutorials/ADVANCED_TUTORIALS.md | guide | Superseded by 5-minute tutorial |
| examples/*/README.md | Example instructions (mostly missing) | Create | examples/*/README.md | core | Need to align with showcase roadmap |

## Lean Docs IA

```
README.md
/docs/guide/
  index.md
  quickstart.md
  tutorial-5-minutes.md
  concepts.md
/docs/reference/
  cli.md
  grammar.md
  config.md
/docs/contributing.md
/docs/changelog.md
/docs/_archive/
  INDEX.md
/examples/
  <showcase>/README.md
```

## Change Plan

| Action | Source | Destination | Notes |
| --- | --- | --- | --- |
| Rewrite | README.md | README.md | Update positioning, quickstart, links |
| Create | docs/guide/index.md | - | New landing page referencing quickstart/tutorial |
| Merge | docs/GETTING_STARTED.md + docs/guides/QUICKSTART.md | docs/guide/quickstart.md | Deduplicate instructions |
| Author | docs/guide/tutorial-5-minutes.md | - | Step-by-step CLI walkthrough |
| Summarize | CODEGEN_PROGRESS_SUMMARY.md | docs/guide/concepts.md | Section on pipeline maturity |
| Create | docs/reference/cli.md | - | Table of commands + status |
| Create | docs/reference/grammar.md | - | Token/annotation reference |
| Create | docs/reference/config.md | - | raven.toml layout |
| Write | docs/contributing.md | - | Conventional commit rules, testing |
| Write | docs/changelog.md | - | Keep a Changelog template |
| Move | docs/api/* | docs/_archive/api/* | Preserve drafts |
| Move | docs/architecture/* | docs/_archive/architecture/* | Preserve with reason |
| Move | docs/engineering/* | docs/_archive/engineering/* | Preserve |
| Move | docs/events/* | docs/_archive/events/* | Preserve |
| Move | docs/marketing/* | docs/_archive/marketing/* | Preserve |
| Move | docs/community/* | docs/_archive/community/* | Preserve |
| Move | docs/tutorials/* | docs/_archive/tutorials/* | Preserve |
| Move | docs/archived/* | docs/_archive/legacy/* | Normalize |
| Add | docs/_archive/INDEX.md | - | Table linking all archived docs |
| Update | README.md, docs/guide/index.md | - | Link to showcase roadmap |

## Search & Replace Checklist

- Replace references to `raven compile` -> `raven build` where applicable.
- Update old script mentions (`serve.py`, `scripts/dev-server.js`) to `raven dev`.
- Normalize casing for "RavensOne" vs "Ravensone".
- Replace outdated feature claims (ORM, auth, package registry) with "planned" or remove.
- Update instructions pointing to `docs/guides/QUICKSTART.md` to new `/docs/guide/quickstart.md` path.
