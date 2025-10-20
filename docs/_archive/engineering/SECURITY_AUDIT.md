# Security Audit Plan (Pre v1.0)

## Objectives

- Validate compiler, runtime, and package registry attack surface
- Perform threat modeling for auth flows, storage, and RPC bridge
- Deliver remediation playbook and responsible disclosure policy

## Scope

1. **Compiler + CLI**
   - Token/AST parsing (input sanitization)
   - Code generation + bundling (path traversal, template injection)
   - Package manager commands (`raven pkg *`)

2. **Runtime**
   - RPC layer authentication/authorization
   - WebSocket message validation + rate limiting
   - File system sandboxing + environment variable access

3. **Registry + Auth Services**
   - OAuth + password flows
   - JWT signing + refresh tokens
   - Database schema + encryption at rest

## Activities

- [x] Hire third-party security firm (Trail of Bits)
- [x] Prepare architecture + threat model documentation
- [ ] Conduct manual code review + fuzz testing (scheduled Nov 8-15)
- [ ] Run dependency audit (Cargo, npm, OS packages)
- [ ] Validate incident response + logging pipelines

## Deliverables

- Comprehensive findings report with severity classification
- Mitigation backlog tracked in Linear + GitHub Projects
- Updated security guidelines in `docs/SECURITY.md`
- Public disclosure summary after remediation window

## Timeline

- **Oct 28** – Kickoff, scope confirmation
- **Nov 8-15** – On-site + remote testing
- **Nov 18** – Draft report delivered
- **Nov 22** – Remediation plan sign-off
- **Dec 1** – Publish sanitized summary to community
