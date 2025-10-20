# Configuration

RavensOne uses a `raven.toml` manifest stored at the project root. The file is intentionally small while the package ecosystem evolves.

```toml
[package]
name = "demo-app"
version = "0.1.0"
```

## Fields

| Key | Description | Status |
| --- | --- | --- |
| `package.name` | Project identifier (snake-case). | Required |
| `package.version` | Semantic version string. | Required |
| `targets.client` | Future override for client build output. | Planned |
| `targets.server` | Future override for server build output. | Planned |
| `dependencies` | Table of package dependencies. | Planned |

The CLI currently ignores unknown keys but prints warnings when encountering sections marked *Planned*.

## Environment Files

- `.env` (optional) — forwarded to future deploy tooling.
- `.ravenrc` (planned) — runtime configuration overrides.

## Templates

`raven init` and `raven new` copy templates from the CLI installation directory. See `templates/` for the latest scaffold content.

## Related Docs

- [CLI reference](./cli.md)
- [Concepts](../guide/concepts.md)
