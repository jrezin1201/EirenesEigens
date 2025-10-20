# Advanced Tutorials Roadmap

These tutorials expand on the quick start and focus on production-grade
workflows. Each tutorial includes runnable `.raven` code, step-by-step
explanations, and troubleshooting tips.

| Tutorial | Summary | Status |
|----------|---------|--------|
| `01-authentication-strategies` | Multi-factor auth, social login, SSO, and session hardening patterns. | ✅ Drafted |
| `02-data-modeling` | Designing relational + document schemas with migrations. | ✅ Drafted |
| `03-real-time-collaboration` | WebSockets, CRDTs, optimistic UI, and offline sync. | 🚧 In progress |
| `04-operations` | Observability, logging, tracing, and error budgets. | 🚧 In progress |
| `05-deployment-deep-dive` | Blue/green deploys, preview branches, and Terraform automation. | 📝 Planned |

## Tutorial Outlines

### 1. Authentication Strategies

- Credential storage with Argon2id + pepper rotation
- Device-based trust scoring and WebAuthn
- Passwordless flows using magic links
- Role-based access control with hierarchical policies

### 2. Data Modeling for Raven ORM

- Schema composition with modules
- Deriving resolvers from entity definitions
- Handling migrations and rollbacks
- Query performance profiling with the Raven CLI

### 3. Real-Time Collaboration (In Progress)

- Document-level CRDT implementation
- Reconciling offline edits on reconnect
- Presence + cursor sharing via the RPC bridge
- Scaling fan-out with Redis and NATS adapters

### 4. Operational Excellence (In Progress)

- Structured logging patterns and correlation IDs
- Distributed tracing with OpenTelemetry exporters
- Feature flags, kill switches, and progressive delivery
- Incident response runbooks and severity matrices

### 5. Deployment Deep Dive (Planned)

- Infra-as-code blueprints for Fly.io, AWS, and Cloudflare
- Canary deployments with automatic rollbacks
- Integrating load testing into CI pipelines
- Hardening TLS, secrets management, and runtime policies

## Publication Schedule

- **Week 1** – Release Tutorials 1 & 2 with accompanying livestreams
- **Week 2** – Ship Tutorial 3 draft and solicit community feedback
- **Week 3** – Publish Tutorial 4 alongside observability dashboards
- **Week 4** – Finalize Tutorial 5 after the infrastructure pass

## Measurement

- Track completion rates via the docs analytics dashboard
- Run surveys during office hours to capture friction points
- Monitor GitHub Discussions for follow-up questions
