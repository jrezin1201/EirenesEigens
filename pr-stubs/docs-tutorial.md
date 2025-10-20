# docs: 5-minute tutorial and quickstart aligned to CLI

## Summary
- Add a concise quickstart that mirrors the current CLI workflow.
- Author a 5-minute tutorial with realistic commands and caveats.
- Update guide index to point readers to new walkthrough content.

## Rationale
The previous getting-started docs promised unimplemented features. The new tutorial provides an honest flow we can demo today.

## Files Changed
- docs/guide/index.md
- docs/guide/quickstart.md
- docs/guide/tutorial-5-minutes.md

## Acceptance Criteria
- [ ] Tutorial steps can be followed on a clean checkout.
- [ ] Quickstart matches CLI behaviour and flags.
- [ ] Links resolve and mention known limitations.
- [ ] `npm run docs:check` passes.
