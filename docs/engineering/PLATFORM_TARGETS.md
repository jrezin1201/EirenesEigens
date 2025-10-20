# Expanded Platform Targets

We are extending the Raven compiler to support mobile and desktop deployments.

## Roadmap

| Target | Strategy | Owner | Status |
|--------|----------|-------|--------|
| React Native Alternative | Generate bridging layer to Hermes + native UI components | Compiler Team | 🚧 Prototype |
| iOS Native | Swift Package export with Raven runtime embedded | Mobile Team | 📝 Design |
| Android Native | Kotlin Multiplatform bindings + Jetpack Compose adapters | Mobile Team | 📝 Design |
| Desktop (Tauri) | Emit Rust + TypeScript bundles for Tauri shell | Desktop Team | 🚧 Prototype |

## Key Tasks

1. Abstract DOM + HTTP APIs into platform-agnostic facades.
2. Provide native UI bindings (list, form, media) for mobile shells.
3. Integrate push notifications + background tasks on mobile.
4. Expose device APIs (camera, file system) through Raven modules.
5. Package desktop builds with auto-updater + code signing.

## Milestones

- **Prototype Demo** – Dec 5 (React Native alt + Tauri)
- **Developer Preview** – Jan 16 (iOS + Android beta SDKs)
- **General Availability** – Mar 10 (production support + docs)

## Risks & Mitigations

- **Performance**: Validate JIT/AOT strategy with large apps; run benchmarks.
- **API Surface**: Document platform-specific limitations early.
- **Distribution**: Ensure compliance with App Store/Play Store policies.
