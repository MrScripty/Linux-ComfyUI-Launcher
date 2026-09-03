# Torch Inference Sidecar

The Torch sidecar is an experimental, source-only Python service for loading
compatible model artifacts. It is not shipped or supported as a Pumas runtime.
Promotion requires a resolved production dependency set and real ASGI,
model-loader, generation, device, responsiveness, and shutdown evidence.

## HTTP Surface

- `GET /v1/models`: OpenAI-shaped model-list response
- `POST /v1/chat/completions`: text-only, non-streaming chat request subset
- `POST /v1/completions`: single-text, non-streaming completion request subset
- `/api/*`: Pumas load, unload, status, and device controls
- `/health`: process health

The inference routes are OpenAI-shaped, not generally OpenAI-compatible. The
request decoder accepts only:

- an exact set of known fields;
- a nonblank model identifier of at most 256 characters;
- `system`, `user`, and `assistant` chat messages with nonempty string content,
  at most 256 messages, and at most 1,000,000 aggregate content characters;
- one nonempty completion prompt of at most 1,000,000 characters;
- `temperature` from 0 through 2 and `top_p` above 0 through 1, with `top_p=1`
  required when `temperature=0`;
- `max_tokens` from 1 through 4,096; and
- `stream=false` and `stop=null` only.

Unknown fields, streaming, stop sequences, other chat roles or content forms,
multiple completion prompts, and values outside those bounds are rejected.
Current response usage remains placeholder data, terminal reasons are not yet
fully truthful, and synchronous inference does not yet have an accepted work,
cancellation, overload, or shutdown owner.

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

These commands are for direct development only. The requirements are ranges,
not a reproducible resolution, and the current managed installer does not
install this sidecar: its source identity, Python version, entry point, and
cross-platform interpreter path are inconsistent. Do not treat a successful
source checkout or fake-backed test run as a supported deployment.

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
Those tests prove focused request and control logic only. They do not prove
ASGI middleware, production dependency resolution, device discovery, model
loading, inference, control responsiveness during inference, disconnect or
cancellation behavior, or bounded shutdown. No real runtime/platform/device
tuple is currently accepted.

The production requirements currently use ranges rather than a reproducible
lock. Capture the exact resolved set for release, vulnerability, and license
evidence. See [Releasing](../RELEASING.md), [Security](../docs/SECURITY.md), and
the [current standards audit](../docs/audits/current-standards-2026-09-03/README.md).
