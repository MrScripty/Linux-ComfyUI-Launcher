# Release And Host Contract Decision

**Status:** `accepted`

**Observed at:** 2026-09-03

**Decision owner:** Pumas product/release owner

**Implementation owner:** Desktop, release, bindings, and Torch plan

## Decision Boundary

Milestone 0 asks which artifacts, consumers, channels, targets, and host
bindings Pumas actually promises. Getting that wrong would either withdraw a
real consumer's artifact or continue publishing unverified native software.
Repository and public-release inventory is cheaper and safer than implementing
more generators or release machinery first.

The investigation stops when every current output and public claim has an
observed consumer or an explicit proposed disposition, and the remaining
product choices are precise enough for the owner to accept or reject. That
stopping condition is met. The product/release owner accepted the decisions
below, so encoding `scripts/release/artifact-plan.json` and
`bindings/support-matrix.json` is admitted under serial ownership.

## Evidence Sources

- Current repository at `453105780b1e5181d27dd1f20b234591bb6ead86`:
  `.github/workflows/build.yml`, `electron/package.json`, the root and Rust
  manifests, release/binding/Torch documentation, binding scripts and host
  smoke, and current generated-output ignores.
- [GitHub release history](https://github.com/MrScripty/Pumas-Library/releases),
  queried through the authenticated GitHub API on 2026-09-03. Six public
  releases, `v0.1.0` through `v0.6.0`, were all marked prerelease.
- [Pumas v0.6.0](https://github.com/MrScripty/Pumas-Library/releases/tag/v0.6.0),
  including release notes, exact asset identities, sizes, and download counts.
- GitHub code search across public repositories on 2026-09-03 for the literal
  `pumas-library` package/repository identity.
- [Pantograph](https://github.com/MrScripty/Pantograph) revision
  `2ba2efb1cd1a06b657f7227bf74caae99f275dfc`, including its root/workflow-node
  manifests, current dependency/release plan, and quality workflow.
- [Pixapillars](https://github.com/MrScripty/Pixapillars) launcher and README,
  which use Pumas transitively through a local Pantograph checkout.
- A local historical `v0.3.0` Debian package, used only to inspect prior
  package composition. It is not evidence about current final bytes.

Release download counts show that assets were acquired, but they do not
identify a consumer or establish a support contract.

## Observed Consumers And Channels

| Consumer | Consumed interface | Acquisition/channel | Compatibility owner | Evidence-backed classification |
| --- | --- | --- | --- | --- |
| Pumas desktop user | Electron renderer, preload/main, and bundled `pumas-rpc` | Public GitHub prerelease assets | Pumas release | Real independent consumer; exact supported OS/package tuples still need owner acceptance and packaged-runtime evidence. |
| Pantograph | `pumas-library` Rust crate | Cargo Git dependency pinned to exact revision `f87c3da8276a914a54c6f4f36d617bef9d9f424e` | Pantograph dependency owner | Real distributed-independent source consumer; it does not consume the published `.crate` asset. Pantograph's current plan intends one Cargo-owned Pumas revision and removal of a redundant CI checkout identity. |
| Pixapillars | Pumas library root through Pantograph | Local sibling checkouts selected by its launcher | Pixapillars/Pantograph | Real transitive development consumer, not a Pumas binding or release-asset consumer. |
| Studio Whip | Project-level mention | None observed | Unavailable | Mention only; not a release consumer. |
| Python, Kotlin, Swift, Ruby, or C# host application | Generated UniFFI material | GitHub binding ZIPs or local generator scripts | Unavailable | No real application consumer found. Generation and downloads do not establish support. |
| Elixir/Erlang host application | `pumas_rustler` NIF | Historical GitHub ZIPs and local build instructions | Unavailable | No real host consumer found; current crate does not expose the Pumas core interface. |
| Go host application | Documentation claim only | None | Unavailable | No generator path, artifact, host consumer, or oracle found. |
| Torch deployment operator | Python sidecar source | Direct source checkout only | Unavailable | The sidecar is not included in current desktop, Rust, or binding artifacts. No deployment channel or accepted runtime/device tuple exists. |

GitHub Releases is the only observed public publication destination. Current
automation creates a draft on a `v*` tag with repository write permission; a
maintainer later made each observed release public. No registry publication,
stable channel, promotion role, support window, withdrawal procedure, or
retention contract was found.

## Current Release Output Inventory

The latest public release contains these roles:

| Current output | v0.6.0 identity/composition | Current evidence | Contract result |
| --- | --- | --- | --- |
| Linux desktop, AppImage | `Pumas.Library-0.6.0.AppImage`; renderer, Electron, target RPC | Linux build and an unpackaged Xvfb launcher smoke | Candidate target; packaged install/load/start and exact-content evidence unavailable. |
| Linux desktop, Debian | `pumas-library-electron_0.6.0_amd64.deb`; same product cohort | Linux package build only | Candidate target; package install/start and final license/notices evidence unavailable. |
| Windows desktop, NSIS | `Pumas.Library.Setup.0.6.0.exe`; renderer, Electron, target RPC | Windows build/package only | Candidate target; packaged install/start, launcher, recovery, and termination evidence unavailable. |
| Windows desktop, portable | `Pumas.Library.0.6.0.exe`; renderer, Electron, target RPC | Windows build/package only | Candidate target; packaged start and runtime evidence unavailable. |
| macOS desktop, DMG | `Pumas.Library-0.6.0-arm64.dmg`; renderer, Electron, target RPC | macOS build/package only | Candidate target; install/start, launcher, recovery, and termination evidence unavailable. |
| Rust source crate | `pumas-library-0.6.0.crate` | `cargo package --allow-dirty`; no registry or artifact consumer | Remove from GitHub release; the real consumer uses an immutable Git revision. |
| Five UniFFI bundles | One ZIP per host language, each combining generated sources with Linux x64, Windows x64, and macOS arm64 native libraries | Source generation plus native compilation; no packaged host run | Remove from release. These bundle unrelated host/native tuples without a host loading contract. |
| Checksum manifest | `checksums-sha256.txt` over collected top-level files | Final-name SHA-256 generation | Retain only if the accepted channel uses downloadable-byte verification; it does not replace artifact, provenance, or usability evidence. |

No current release output includes an accepted per-artifact SBOM, provenance,
project license/notice inventory, signature, or dependency vulnerability result.
The local historical Debian package includes Electron and Chromium license
files but no separately identifiable Pumas MIT license or final dependency
notice inventory.

## Accepted Desktop Matrix

These tuples are release candidates whose promotion remains blocked until their
required-real evidence passes.

| OS/architecture | Candidate packages | Candidate tier | Required-real promotion oracle |
| --- | --- | --- | --- |
| Linux x86_64 GNU | AppImage and Debian | Preview | Extract/inspect exact packages; install or mount as appropriate; start packaged Electron with its same-cohort RPC; exercise launcher-root cold recovery and launcher process termination; verify final metadata. |
| Windows x86_64 MSVC | NSIS installer and portable executable | Preview | Install/start exact packages on Windows; exercise same-cohort RPC, root persistence/recovery, wrapper equivalence, and process-tree deadlines; verify final metadata. |
| macOS arm64 | DMG | Preview | Install/start exact DMG application on macOS arm64; exercise same-cohort RPC, root persistence/recovery, launcher actions, shutdown, and final metadata. |

No x86_64 macOS, arm64 Windows/Linux, Flatpak, Snap, registry, package-manager,
or standalone RPC distribution is admitted. A missing required-real runner or
failed package path blocks only its tuple; it does not fall back to build-only
evidence.

## Accepted Binding Host Matrix

This table records current implementation facts and the accepted removal
disposition. No host tuple remains supported or experimental.

| Surface | Current generator/adapter | Host/runtime and operations | Current native tuples | Real-host oracle | Accepted disposition |
| --- | --- | --- | --- | --- | --- |
| Python | UniFFI 0.28 metadata generator | Runtime version and supported operations unavailable; no host call | Linux x64, Windows x64, macOS arm64 are bundled together | None | Remove generator, Adapter, docs, tests, and release paths. |
| Kotlin | UniFFI 0.28 metadata generator | JVM/Android runtime, package form, async/error/event contract unavailable | All three native tuples bundled | None | Remove generator, Adapter, docs, tests, and release paths. |
| Swift | UniFFI 0.28 metadata generator | Swift/Xcode runtime and supported platform contract unavailable | All three native tuples bundled despite no cross-platform host contract | None | Remove generator, Adapter, docs, tests, and release paths. |
| Ruby | UniFFI 0.28 metadata generator | Runtime version and supported operations unavailable | All three native tuples bundled | None | Remove generator, Adapter, docs, tests, and release paths. |
| C# | `uniffi-bindgen-cs` tag `v0.9.0+v0.28.3` | Local .NET smoke loads Linux debug native code, calls only synchronous `version()`, and constructs records; async/error/event behavior is unproved | All three native tuples bundled | No release CI host run | Remove generator, Adapter, smoke/package, docs, and release paths. Pixapillars uses Pantograph's C# binding, not this one. |
| Elixir/Erlang | Rustler 0.34 | Host versions unavailable; local parsers/constructors only; no core Pumas interface | Host/target package contract unavailable | No BEAM host run in this repository | Remove generation/docs/release paths and remove the Adapter crate through the Rust-owned source seam. |
| Go | Comment claims `uniffi-bindgen-go` | No generator, runtime, operation, async/error/event, package, or consumer contract | None | None | Remove the false advertised surface. |

Generator availability is inventory only. Deleting the entire non-consumed
binding surface is simpler than maintaining five hypothetical seams. Any later
host promotion is a new matrix decision with an exact real host/native tuple
and cohort oracle.

## Artifact Evidence Obligations

| Accepted artifact role | Resolved dependency input | Final-byte evidence required before promotion |
| --- | --- | --- |
| Desktop package | `pnpm-lock.yaml`, Rust `Cargo.lock`, pinned Node/Electron/Rust toolchains, target system inputs | Exact extracted renderer/RPC cohort; target start/workflow evidence; final-byte SHA-256; per-target resolved dependency/SBOM and provenance; Pumas MIT license plus reviewed required third-party notices; current vulnerability result. |
| Rust Git source consumed by Pantograph | Immutable Pumas Git revision and Rust lock resolution in the consuming repository | Pumas contract/change evidence plus Pantograph's locked build/tests at the exact revision. No duplicate `.crate` artifact is required unless a separate consumer is accepted. |
| Binding cohort, if later accepted | Exact Rust revision, Cargo lock, target/profile, generator product/version, generated source and native checksums | Clean generation; cohort manifest; extracted package verification; real host load/call/async/error/cancellation evidence for the exact tuple; applicable licenses/notices/SBOM/provenance. |
| Torch deployment, if later accepted | A reproducible Python resolution for each selected Python/platform/device class | Real ASGI and production model load/generation; responsiveness/cancellation/shutdown; environment-specific dependency graph, SBOM, provenance, license/notice, and vulnerability evidence. |

The repository's MIT project license is clear. The repository maintainer owns
third-party notice interpretation and acceptance for each final artifact.

## Accepted Product Contract

The product/release owner accepted the smallest contract consistent with
observed consumers and current product direction:

1. Treat Pumas as one versioned product with two consumed interfaces: the
   desktop release and the source-level Rust contract consumed by exact Git
   revision. Root/frontend/electron/Cargo versions remain lockstep product
   versions unless the owner selects independent release units.
2. Use one named GitHub `preview` channel for the three current desktop target
   tuples. A SemVer `0.x` version does not choose this channel; the accepted
   release plan does. Only a designated repository maintainer may promote an
   evidence-complete draft.
3. Remove the `.crate`, Python, Kotlin, Swift, Ruby, and C# ZIPs from public
   release assembly. Pantograph uses Git, and no host binding consumer was
   found.
4. Remove UniFFI, Rustler/Elixir, and Go generation, Adapter, registration,
   packaging, test, and advertising surfaces. A later real consumer requires a
   new host/target contract rather than dormant machinery.
5. Keep Torch source as an optional, non-shipped capability while Milestone 3
   defines and proves its smallest real runtime/device contract. Do not attach
   it to a desktop or standalone release until that contract is accepted.
6. Require exact final-byte contents, checksums, resolved dependency closure,
   SBOM/provenance, reviewed project and third-party license material, current
   vulnerability evidence, and target workflow evidence before any draft is
   promoted. Publication remains a separate manual transition.

For this standards-remediation program, draft promotion requires `PRG-A1`
through `PRG-A7` and `DRBT-A1` through `DRBT-A9`. Security and governance are
therefore explicit release prerequisites rather than being inferred from a
successful package job. Torch remains absent from the shipped bytes, but its
source contract must still satisfy the program before the remediation release
is promoted.

The top-level versioned `THIRD-PARTY-NOTICES-{version}.txt` is the accepted
review artifact. Each desktop package embeds byte-identical content under the
stable internal name `THIRD-PARTY-NOTICES.txt`; the two names serve publication
discovery and installed-artifact discovery without creating two notice
authorities.

## Owner Decision

The owner explicitly accepted the following bundle on 2026-09-03:

1. GitHub `preview` is the only public release channel; no stable channel or
   maintenance window exists yet.
2. Linux x64 AppImage/Debian, Windows x64 NSIS/portable, and macOS arm64 DMG are
   the accepted desktop tuples, each blocked from promotion until its real
   package evidence passes.
3. Remove the unused `.crate` and five UniFFI binding ZIPs from GitHub releases.
4. Remove UniFFI until a real host consumer and tuple are accepted.
5. Remove the non-core Rustler/Elixir surface and false Go claim.
6. Torch is intentionally non-shipped until Milestone 3 earns a selected
   runtime/device contract.
7. A repository maintainer owns manual draft promotion and third-party
   license/notice interpretation.

## Program Claim Impact

- `PRG-A6` now has an accepted consumer/support decision. It remains pending
  until target and final-byte release evidence passes.
- `PRG-A3` is affected by the desktop tuple decision because launcher-root
  interruption and recovery must run on every accepted filesystem/OS target.
- `PRG-A4` is affected by both the desktop tuple decision and the Torch
  disposition because launcher/process/stream lifecycle needs each accepted OS,
  while Torch scheduling needs a resolved real stack and selected device class.
- `DRBT-A7` is satisfied by complete removal only after every binding surface is
  absent. `DRBT-A8` and `DRBT-A9` remain pending. `DRBT-A3`, `DRBT-A4`, and
  `DRBT-A6` now have the accepted OS environments needed for their evidence.

## Module And Machinery Review

The release artifact-plan Module earns depth only if one small Interface hides
target naming, collection, version comparison, cohort checks, metadata, and
missing/unexpected-output rules from CI and local callers. No parser or generic
release framework is admitted before its input contract exists.

Generated binding sources are Adapters at a host seam, not products or semantic
owners. With no real host consumer, five publication paths are hypothetical
seams. Removing them reduces caller knowledge and maintenance more than adding
matrix parsers around them. The Rust source contract remains owned by the Rust
plan, and its exact-Git consumer must not be reinterpreted as evidence for a
host package or GitHub `.crate` asset.

## Handoffs

- Rust plan: preserve Pantograph's source consumer; provide binding conversion,
  error, async, and core/Adapter ownership before any UniFFI host promotion;
  remove Rustler source through the Rust-owned seam if the owner accepts that
  disposition.
- Frontend plan: no release-support claim follows from renderer build success;
  provide the accepted built-renderer workflow for each desktop package path.
- Governance plan: serialize shared workflow and package-script changes and
  attach retained gates only to the accepted claims above.
- Program/release owner: serialize the accepted artifact plan, host matrix,
  CI/package projections, and durable release documentation.
