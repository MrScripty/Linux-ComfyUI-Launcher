# Torch Inference Sidecar

The Torch sidecar is an optional Python service for loading compatible model
artifacts and exposing Torch inference through Pumas.

## HTTP Surface

- `/v1/*`: OpenAI-compatible inference routes
- `/api/*`: Pumas load, unload, status, and device controls
- `/health`: process health

`ModelManager` owns loaded model slots and eviction. `DeviceManager` owns
device discovery and selection. Loader modules own framework-specific loading;
route handlers must not duplicate those policies.

## Install and Run

Use the Python version pinned at the repository root. From the repository root:

```bash
python3 -m venv torch-server/.venv
torch-server/.venv/bin/pip install -r torch-server/requirements.txt
torch-server/.venv/bin/pip install -r torch-server/requirements-dev.txt
torch-server/.venv/bin/python torch-server/serve.py --host 127.0.0.1 --port 8400 --max-models 4
```

The managed Pumas runtime normally creates and launches its own environment;
the commands above are for direct development.

## Network Safety

Loopback is the default. Binding to a non-loopback address requires both:

```bash
PUMAS_TORCH_ALLOW_LAN=1
PUMAS_TORCH_API_TOKEN=<secret>
```

All non-health routes then require the token through
`X-Pumas-Torch-Token` or a bearer authorization header. Do not expose a local
model server to an untrusted network merely because it has a token; also apply
host firewall and network controls.

## Verification

```bash
python3 -m ruff check torch-server
python3 -m ruff format --check torch-server
python3 -m unittest discover -s torch-server/tests
```

The unit suite can install local fakes for missing Torch/runtime dependencies.
That is useful for control logic but does not prove ASGI middleware, production
dependency resolution, device discovery, model loading, or inference. Use the
real resolved environment for claims at those boundaries.

The production requirements currently use ranges rather than a reproducible
lock. Capture the exact resolved set for release, vulnerability, and license
evidence. See [Releasing](../RELEASING.md), [Security](../docs/SECURITY.md), and
the [current standards audit](../docs/audits/current-standards-2026-09-03/README.md).
