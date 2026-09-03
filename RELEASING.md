# Releasing Pumas Library

## Current Status

Tagged CI builds desktop installers, a source crate, generated UniFFI packages,
and checksums, then opens a draft GitHub release. The current standards audit
identifies unresolved release-contract, SBOM, dependency-lock, binding-host, and
cross-platform runtime evidence gaps. Do not publish a draft based only on a
green build.

## Prepare the Version

1. Update `CHANGELOG.md` and create a new empty `Unreleased` section.
2. Set the same SemVer value in:
   - root `package.json`;
   - `frontend/package.json`;
   - `electron/package.json`; and
   - `rust/Cargo.toml` under `[workspace.package]`.
3. Run `npm run check:release-versions`.
4. Commit with `chore(release): prepare vX.Y.Z`.

## Required Local Evidence

```bash
corepack pnpm install --frozen-lockfile
./scripts/rust/check.sh
npm run -w frontend lint
npm run -w frontend check:types
npm run -w frontend test:run
npm run -w frontend build
npm run -w electron lint
npm run -w electron test
npm run -w electron validate
npm run test:launcher
python3 -m ruff check torch-server
python3 -m ruff format --check torch-server
python3 -m unittest discover -s torch-server/tests
./launcher.sh --build-release
./launcher.sh --release-smoke
```

The Torch unit suite can substitute local fakes when runtime packages are not
installed. If Torch is shipped, also verify the resolved production dependency
set and a real load/inference/control flow.

Run current vulnerability and license checks for Cargo, pnpm, and the resolved
Torch environment. Record the tool versions and results; do not rely on old
checked-in scan snapshots.

## Tag and Draft

```bash
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

The `Build` workflow creates a draft release. It currently assembles:

- the packaged `pumas-library` source crate;
- Electron installers from Linux x64, Windows x64, and macOS arm64 jobs;
- generated Python, Kotlin, Swift, Ruby, and C# binding archives containing
  native libraries collected from the Rust build matrix; and
- `checksums-sha256.txt` over the files copied into the final staging directory.

## Review the Draft

Before publication, independently verify:

- the tag version equals every manifest version;
- every expected platform artifact exists and unexpected files are absent;
- each desktop package contains the matching `pumas-rpc` binary and renderer;
- binding source and native libraries come from the same build cohort;
- checksums cover the exact final bytes being published;
- current Rust, Node/Electron, and Torch SBOMs cover the resolved shipped
  dependency closure;
- required licenses/notices are present in the distributable; and
- supported runtime workflows were exercised on every platform claimed by the
  release notes.

The workflow does not yet provide all of those proofs. See the
[current standards audit](docs/audits/current-standards-2026-09-03/README.md)
before changing or approving release automation.

## Local Binding Checks

```bash
./scripts/check-uniffi-surface.sh
./scripts/check-uniffi-csharp-smoke.sh
./scripts/package-uniffi-csharp-artifacts.sh
```

These are local generation/packaging helpers, not evidence that every advertised
host and target combination works. Keep generated sources and native libraries
version-matched. See [Native bindings](docs/native-bindings.md).

`pumas_rustler` requires an Erlang/OTP host and is excluded from the default
workspace/release checks. It is not currently a release-supported core binding.
