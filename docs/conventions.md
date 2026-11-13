# Engine Conventions

## Coordinates and units

- Use a right-handed coordinate system.
- `+Y` is up.
- A default camera looks along `-Z`.
- World distances are expressed in meters.
- Time is expressed in seconds at gameplay boundaries; use precise duration types inside timing infrastructure.
- Angles are expressed in radians internally.
- Rendering calculations use linear color. Conversion from sRGB assets and to the presentation format occurs at explicit boundaries.

## Time and scheduling

- Simulation uses a fixed 60 Hz step.
- Rendering and operating-system events are not assumed to run at the simulation rate.
- A single render pass may process at most five fixed simulation steps.
- Excess accumulated time beyond that bound is discarded and counted as lost simulation time. This prevents an unrecoverable catch-up spiral; the policy can be revisited when real gameplay requirements exist.
- No engine-owned worker threads are introduced during Milestone 0.
- Subsystem execution order must be explicit and documented when additional subsystems are added.

## Identity and ownership

- Use typed identifiers for entities, assets, and resources.
- Reusable entity or resource slots must include generation checks so stale handles can be detected.
- The runtime owns subsystem initialization and shutdown order.
- Resources are reached through explicit owners and handles, not a global mutable engine singleton.
- Long-lived references must not be scattered across subsystem boundaries.
- Expected resource, loading, and platform failures must return errors rather than panic.

## Data and serialization

- Every persisted scene or asset format includes an explicit format version from its first implementation.
- Changes to persisted formats must be documented and must either preserve compatibility or fail with a useful version error.
- Machine-local absolute paths must not be embedded in portable scene data.

## API and module boundaries

- Example games consume the engine through the public `engine` crate.
- Internal subsystem types are not re-exported merely for convenience.
- A third-party type crosses a public boundary only when that dependency is deliberately part of the engine's API contract.
- New abstractions require a real portability, ownership, testing, or replacement seam. Uniform wrapper traits are not a goal.
- Crates are introduced when they contain real functionality, not as empty placeholders for future systems.

## Diagnostics and quality

- Use `rustfmt` defaults unless the project later adopts a specific formatting rule.
- Clippy warnings are denied in the project's quality checks.
- Executables configure log subscribers; library crates emit structured diagnostics without installing global subscribers.
- Startup, shutdown, and recoverable failures must be diagnosable without flooding normal output.
- Tests use explicit inputs and observable outcomes. Rendering and performance acceptance cannot be inferred from non-GPU unit tests.

## Naming

- The public product name is **Unbiased Bob Engine (UBE)**; the current release label is **nerd 0.1**.
- Internal crates retain the `engine_` prefix and Cargo retains semver package versions.
- The public facade crate is named `engine`.
- The first example executable is named `engine-sandbox`.
