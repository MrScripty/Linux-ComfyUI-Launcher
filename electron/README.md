# Electron Desktop Shell

Electron owns the privileged desktop boundary between the React renderer and
the local Rust sidecar.

## Responsibilities

- create and secure application windows;
- expose the narrow preload API available to the renderer;
- start, monitor, and stop `pumas-rpc`;
- validate renderer IPC requests before forwarding them;
- own native dialogs and other desktop-only operations; and
- package the renderer, RPC binary, and platform icons.

Electron does not own model-library rules or durable runtime truth.

```text
React renderer
  -> preload API
    -> validated Electron IPC
      -> loopback HTTP/JSON-RPC
        -> pumas-rpc
```

## Security Boundary

Renderer windows use context isolation and sandboxing. Do not enable direct
Node access or add a generic IPC/RPC escape hatch. The executable RPC method
and request registry is `src/rpc-method-registry.ts`; `src/ipc-validation.ts`
applies it before dispatch.

TypeScript interfaces are compile-time projections, not validation of runtime
messages. A contract change must update the receiver decoder, producer,
consumer, and negative tests together. Treat sidecar logs as sensitive: they
can contain backend diagnostics and must not expose credentials.

## Sidecar and Events

Electron owns process lifecycle and forwards backend event streams to the
renderer. It must make startup, ready, timeout, exit, and shutdown outcomes
distinct. Events coordinate or invalidate cached state; they do not transfer
durable ownership into Electron.

The RPC server is loopback-only by default. Pumas RPC LAN mode is currently
unauthenticated and must not be exposed to an untrusted network.

## Commands

From the repository root:

```bash
npm run -w electron lint
npm run -w electron validate
npm run -w electron test
npm run -w electron build
npm run -w electron dev
```

The direct development command expects a built renderer and an available
sidecar. For the supported end-to-end workflow, use:

```bash
./launcher.sh --build
./launcher.sh --run
```

Packaging requires `frontend/dist` plus a matching `pumas-rpc` staged under
`electron/resources/bin/`. Do not treat a successful package assembly as proof
that the application runs on that platform. See [Releasing](../RELEASING.md)
and the [current standards audit](../docs/audits/current-standards-2026-09-03/README.md).
