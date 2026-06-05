# LARGER-001 — Pre-built image release pipeline (reference: deployment epic)

## Why this came from documentation

Multiple places in the docs note:

> "Pre-built images and a standalone download are not yet published. You are building
> from source with Docker."

And `scripts/zealot-compose.yml` does reference images at
`registry.alexanderfarrell.net/zealotd:v0.1.0` — but it's not clear these are
actually published and up to date. The quickstart acknowledges: "the first build
takes a few minutes."

Building from source on first run is a meaningful adoption barrier: Rust compilation
can take 10+ minutes on a slow machine. A published Docker image at a registry eliminates
this.

## Pain points identified

1. **First-run friction:** New users must wait for full Rust compilation before seeing
   the app. This discourages exploration.
2. **Stale image tags in `zealot-compose.yml`:** If the registry images are out of date,
   operators using `zealot-compose.yml` are running old code without knowing it.
3. **No documented release process:** There is no documentation for cutting a release,
   tagging an image, or what version numbers mean.

## Belongs in

A dedicated **deployment / release engineering epic**, covering:
- CI/CD pipeline (GitHub Actions) to build and push images on tag/release
- Versioning scheme
- `scripts/zealot-compose.yml` pointing to a tagged `latest` or semver tag
- `docs/quickstart.md` updated to use `docker compose -f scripts/zealot-compose.yml up`
  as the primary install path once images are reliably published
