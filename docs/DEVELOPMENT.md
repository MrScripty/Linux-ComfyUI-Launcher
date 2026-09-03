# Development

## Setup

Use the versions pinned by `rust-toolchain.toml`, `.node-version`,
`.python-version`, and the root `package.json`.

```bash
corepack enable
corepack pnpm install --frozen-lockfile
./launcher.sh --build
./launcher.sh --run
```

On Windows, use the same actions through `launcher.ps1`.

## Standards

The standards source is:

```text
/media/jeremy/OrangeCream/Linux Software/repos/owned/developer-tooling/Coding-Standards/
```

Read `CORE-STANDARDS.md`, then use `STANDARDS-ROUTER.md` to select the canonical
workflow, topic, language, application, and boundary owners for the change.
Repository docs explain Pumas-specific facts; they do not replace the standards.

File length, directory count, and the existence of a `src/` directory are
diagnostic facts, not design or documentation requirements.

## Common Commands

### Rust

```bash
./scripts/rust/check.sh
cargo test --manifest-path rust/Cargo.toml -p pumas-library <test-filter>
```

The aggregate script runs formatting, all-target/all-feature checks, Clippy,
workspace tests and docs, and no-default compilation. It excludes
`pumas_rustler`, which needs an Erlang/OTP host.

### Frontend

```bash
npm run -w frontend lint
npm run -w frontend check:types
npm run -w frontend test:run
npm run -w frontend build
npm run -w frontend build:library-only
```

`check:size` and `check:errors` are legacy policies under review. Neither is a
substitute for an ownership/design review or a typed error-contract check.

### Electron and Launcher

```bash
npm run -w electron lint
npm run -w electron test
npm run -w electron validate
npm run test:launcher
node scripts/launcher/cli.mjs --help
```

### Torch Sidecar

```bash
python3 -m ruff check torch-server
python3 -m ruff format --check torch-server
python3 -m unittest discover -s torch-server/tests
```

The unit suite can install local fakes for missing runtime packages. Use a real
resolved Torch environment for ASGI, middleware, loading, or inference claims.

## Frontend Conventions

- Backend responses and persisted browser data remain `unknown` until decoded.
- Hooks own async work, cancellation, current-invocation checks, and cleanup.
- Components render backend-owned state; they do not invent durable success.
- Prefer native semantic controls. When richer interaction is needed, use an
  established accessible pattern with keyboard, focus, naming, dismissal, and
  state behavior verified together.
- The ESLint policy requires React Aria `useHover` rather than raw mouse hover
  handlers.

Theme tokens live in `frontend/src/index.css`; programmatic mappings live in
`frontend/src/config/theme.ts`. Use semantic tokens rather than hard-coded
colors. Do not claim contrast or reduced-motion compliance without current
representative evidence.

## Verification Design

Every permanent gate needs a named claim, proof boundary, adequate oracle,
known overlap, and enough marginal value to justify its maintenance. Examples:

- a typecheck proves static type consistency, not runtime payload validity;
- jsdom component tests do not prove real focus or layout behavior;
- a cross-platform build does not prove runtime behavior on that platform;
- a startup smoke proves startup, not a user workflow; and
- generated bindings do not prove that a host can load and call them.

Choose targeted evidence first, then broaden where the changed boundary or
release risk requires it.

## Documentation Lifecycle

Update the nearest current owner document. Add a new document only when it has
a distinct durable audience or decision owner. Do not create per-directory
READMEs, duplicate command guides, or permanent implementation diaries.

Plans are temporary execution authority. Once complete or abandoned, preserve
durable decisions in an ADR/current guide and remove the plan. Audits are dated
snapshots and must name their code and standards baselines.
