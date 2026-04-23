# shared/

Cross-layer shared standards and fixtures.

## Canonical Rule
- Keep shared contracts here only when both Nostra and Cortex require them.
- Layer-specific domain logic must stay inside its owning workspace.

## A2UI Fixtures
- `shared/a2ui/fixtures/render_surface_golden.json`: baseline parity fixture.
- `shared/a2ui/fixtures/render_surface_spatial_plane_golden.json`: additive `SpatialPlane` fixture for contract-first canvas integration.
- `shared/a2ui/fixtures/spatial_plane_replay_deterministic_case.json`: deterministic command replay fixture for spatial plane state projection.
