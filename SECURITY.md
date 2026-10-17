# Security policy

## Supported versions

khagole-engine is experimental (0.0.x). Security fixes go into a new patch release of
the latest version only; older releases are not patched. This applies to the Rust crate,
the Python wheels and the npm package alike, which share one version number.

| Version | Supported |
|---|---|
| latest 0.0.x | yes |
| anything older | no |

## Reporting a vulnerability

Please **do not open a public issue**. Report it privately through GitHub:
[Report a vulnerability](https://github.com/PriyankGahtori/Khagole/security/advisories/new)
(repository *Security* tab → *Report a vulnerability*).

Please include:

- the affected version and how you use it (Rust crate, Python, WebAssembly; OS or browser);
- what an attacker can achieve, and under which conditions;
- a reproducer: for example a crafted kernel file, or the request that triggers the issue.

## What to expect

This is a small project maintained on a best-effort basis:

- an acknowledgement within 7 days;
- an assessment, and a fix or mitigation plan, as soon as it is understood;
- coordinated disclosure: the fix is released first, then a GitHub Security Advisory is
  published (plus a [RustSec](https://rustsec.org) advisory for the crate), crediting you
  unless you prefer otherwise.

## Scope

The engine is often fed data that the host application does not control, so the main
attack surface is untrusted input:

- **SPK kernel bytes** passed to `Spk::from_bytes` / `Spk::open`, the WebAssembly
  `addKernel` and `excerptKernel`, or the Python `Engine` constructor. A malformed kernel
  must be rejected with an error. It must never cause memory corruption, a panic that
  crosses the bindings, unbounded memory allocation or an endless loop.
- **Chart requests** (dates, coordinates, orb settings, stored chart payloads passed to
  `synastry` / `derived_chart`) coming through the Python and WebAssembly bindings.
- **The release pipeline**: GitHub workflows, trusted publishing and the published
  artifacts on crates.io, PyPI and npm.

Out of scope:

- values that you believe are astronomically or astrologically wrong. Please open a
  normal issue with the request and the source of the expected value;
- vulnerabilities in dependencies that cannot be reached through the engine. Please report
  them upstream. If you are not sure whether one can be reached, report it here.

## Hardening in place

- Pure Rust with no `unsafe` code in the engine or its bindings, and no C dependencies.
- Few dependencies (only `serde` and `serde_json` in the core), all under permissive
  licenses. [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) checks them in CI
  for advisories, licenses and sources.
- Every GitHub Action is pinned to a commit SHA, which CI verifies, and checked against
  OSV.
- Releases are published through Trusted Publishing (OIDC), so no long-lived registry
  token is stored. Only maintainers can push release tags.
