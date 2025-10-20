# docs: consolidate IA + archive legacy files

## Summary
- Reorganise documentation into guide, reference, contributing, and changelog sections.
- Move historical/marketing content into `/docs/_archive` with an index of reasons.
- Update README messaging to reflect pre-alpha status and link to new docs.

## Rationale
A lean doc structure makes it easier to onboard contributors and stage demos. Archiving legacy plans preserves history without overwhelming new readers.

## Files Changed
- README.md
- docs/guide/**/*
- docs/reference/**/*
- docs/contributing.md
- docs/changelog.md
- docs/_archive/**/*
- docs/_archive/INDEX.md

## Acceptance Criteria
- [ ] Docs tree matches target IA.
- [ ] All archived files listed in `/docs/_archive/INDEX.md` with reasons.
- [ ] README links resolve to existing files.
- [ ] `npm run docs:check` passes.
