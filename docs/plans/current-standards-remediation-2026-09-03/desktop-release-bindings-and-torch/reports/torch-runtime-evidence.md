# Torch Contract and Runtime Evidence

**Milestone:** 3 — Make the Torch Surface Truthful and Schedulable

**Status:** `active; first request-contract slice accepted`

## Investigation Boundary

This report inventories the actual Pumas Torch ASGI Interface, accepted and
ignored values, response/stream/usage behavior, consumers, dependency and
runtime/device facts, and lifecycle ownership. Any OpenAI compatibility claim
is compared only to current official OpenAI documentation. Source availability,
fakes, or a lower-bounded requirement is not treated as shipped support.

## Current Surface

The independently deployed Python process registers three inference routes and
six control routes plus health:

| Interface | Current accepted input/output | Observed defect or evidence limit |
| --- | --- | --- |
| `GET /v1/models` | Lists ready slot names as OpenAI-shaped model objects. | `created` is request time, not model creation time; duplicate model names are emitted and lookup later selects the first match. |
| `POST /v1/chat/completions` | `model`, string `messages`, `temperature`, `top_p`, `max_tokens`, `stream`, and list-only `stop`; returns one assistant choice or SSE. | Pydantic silently drops every unknown field. Roles are arbitrary strings, but formatting silently drops roles other than `system`, `user`, and `assistant`. |
| `POST /v1/completions` | Same fields with one string `prompt`; returns one choice or SSE. | It does not support the official legacy endpoint's array/token prompt forms, yet the unqualified compatibility claim does not say so. |
| `GET /api/slots`, `/status`, `/devices` | Pumas control responses. | Status projects configured next-start host/port as the current API URL. Internal paths and dependency text can cross failure responses. |
| `POST /api/load`, `/unload`, `/configure` | Pumas load/unload/configure operations. | Load alone uses an executor; unload and device/cache work can block the event loop. Configure cannot change the active listener but its response does not clearly separate effective and next-start state. |
| `GET /health` | Always `{ "status": "ok" }`. | It does not reflect closed admission, overload, shutdown, failed dependencies, or unavailable runtime capability. |

### Accepted but ignored or misrepresented values

- Every unknown request or nested message field is accepted and discarded by
  the default Pydantic extra-field policy.
- Any chat role string parses, but `_format_chat_prompt` drops unrecognized
  roles without a diagnostic.
- `stop` parses for chat and text requests but is never used by non-streaming
  or streaming generation.
- `top_p` is used by non-streaming `model.generate` but ignored by the manual
  streaming sampler.
- `max_tokens: 0` parses and is replaced by 256 through `value or 256`;
  negative and arbitrarily large integers cross the request boundary.
- Empty model identifiers, prompts, message arrays, and contents parse. No
  aggregate input or request-body budget is enforced at the direct sidecar.
- Temperatures and nucleus-sampling values have no domain bounds. Positive
  streaming temperatures are applied, while only non-streaming generation
  applies nucleus sampling.
- Non-streaming success always reports zero prompt, completion, and total
  tokens. Streaming has no usage option or usage event.
- Every terminal choice reports `finish_reason: "stop"`, including exhaustion
  of `max_tokens`, which the external contract distinguishes as `length`.
- Streaming work is accepted despite having no admission, request deadline,
  disconnect check, cancellation propagation, or shutdown owner.

These are boundary failures, not harmless forward compatibility: values can
authorize expensive inference while their requested semantics are discarded.

## Consumer and Deployment Inventory

The concrete first-party consumer is Pumas control UI through
`pumas-app-manager::TorchClient` and the RPC Torch handlers. It calls health,
slots, load, unload, status, devices, and configure. Frontend slot and
configuration panels call those RPC operations. No first-party call to the
sidecar's chat or text completion route was found.

The canonical Pumas serving-provider registry currently contains Ollama,
llama.cpp, and ONNX Runtime only. Torch is not a provider, so the Pumas
OpenAI-shaped gateway cannot select or proxy a served model to this sidecar.
Pantograph contains generic llama.cpp/OpenAI clients, and puma-bot contains a
generic configurable OpenAI client, but the bounded search found no explicit
Torch port, Torch provider, or Pumas Torch sidecar configuration in either.
Generic protocol capability is not an actual consumer contract.

The current managed deployment is internally contradictory:

- the Torch plugin identifies GitHub `pytorch/pytorch`, version-management type
  `python-venv`, Python 3.10, and Pumas entry point `serve.py`;
- the generic installer downloads the upstream PyTorch release source, creates
  a virtual environment inside it, and installs that tree's requirements when
  present;
- the launcher then expects `serve.py` in that installed upstream tree and a
  POSIX `venv/bin/python`; and
- this repository's `.python-version` is 3.12.3.

No installed metadata or source presence turns this into a supported Pumas
sidecar deployment. The accepted release plan therefore correctly keeps Torch
non-shipped.

## OpenAI Contract Comparison

The comparison was made on 2026-09-03 against current official OpenAI primary
documentation, not another compatible server or a local client assumption:

- The official [chat-completion request](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)
  has role-specific message variants, several content representations, a
  current `max_completion_tokens` field, deprecated `max_tokens`, streaming
  options, and a much broader parameter set. Pumas accepts only arbitrary
  string role/content pairs and silently drops unknown fields; that is neither
  full compatibility nor an explicit subset.
- Official chat `temperature` is bounded from 0 through 2 and `top_p` from 0
  through 1. The Pumas models currently accept unbounded values.
- Official chat `stop` accepts a string or up to four strings and excludes the
  matched sequence from output. Pumas accepts only a list and ignores it.
- Official streamed chat returns chunk objects; when usage is requested, an
  additional final usage chunk precedes stream termination. Pumas has no
  `stream_options`, usage event, or cancellation/disconnect contract.
- Official response `finish_reason` distinguishes natural/provided stop from
  token-limit `length`. Pumas always returns `stop`.
- The official [legacy completion request](https://developers.openai.com/api/reference/resources/completions/methods/create)
  accepts string, string-array, token-array, and token-array-array prompts,
  defines numeric ranges, accepts string-or-array stop sequences, and documents
  data-only SSE termination. Pumas implements only one string prompt and does
  not declare that narrower contract.
- The official [model-list response](https://developers.openai.com/api/reference/resources/models/methods/list)
  describes `created` as the model's creation timestamp. Pumas manufactures the
  current request time for each listing.

The compatible surface selected by current evidence is therefore described as
**OpenAI-shaped text generation**, not general OpenAI compatibility. Compatibility
is limited to endpoint names and the explicitly retained JSON shapes. Broader
fields and variants are rejected rather than accepted and ignored.

## Work Ownership and Schedulability

No composition-owned inference work Interface exists today:

- non-streaming route handlers call tokenizer and `model.generate`
  synchronously on the ASGI event-loop thread;
- streaming performs a full synchronous Torch forward pass on that same thread
  for each token and yields to asyncio only after the blocking step;
- model-load work uses the process-global default executor, but its future has
  no lifecycle registry and cancellation of the awaiting request does not stop
  the worker;
- per-device locks serialize model loading only, not inference, unload, or
  access to a loaded object;
- unload can remove a model/tokenizer while an untracked inference still holds
  the loaded object;
- there is no finite inference admission capacity, overload result, fairness
  rule, deadline, disconnect observation, drain order, or incomplete-shutdown
  outcome; and
- the FastAPI app has no lifespan/shutdown owner, while health always reports
  success.

Selecting a thread, dedicated executor, queue, or worker process would be a
runtime-composition decision. The current fake-only environment cannot prove
Transformers/tokenizer/model/device concurrency or interruption behavior, so
that decision remains `unavailable`; it is not replaced by `asyncio.to_thread`
or another convenient mechanism.

## Dependency, Runtime, Model, and Device Facts

Observed local environment at code revision
`a89b6c129a85d3c0a34bd848cafe223002a01b3f`:

| Fact | Result |
| --- | --- |
| Host | Linux x64 |
| Python | 3.12.3 |
| Torch, Transformers, safetensors | unavailable |
| FastAPI, Uvicorn, Accelerate | unavailable |
| psutil, Pydantic, httpx | 5.9.8, 2.12.5, 0.28.1 |
| Unit suite | 13 pre-slice and 18 post-slice passed using dependency fakes |
| Real ASGI/model/device inference | unavailable |

`requirements.txt` declares lower bounds only (`torch>=2.1`,
`transformers>=4.36`, `safetensors>=0.4`, `fastapi>=0.104`,
`uvicorn[standard]>=0.24`, `accelerate>=0.25`, `psutil>=5.9`). It has no
resolved identities, indexes/wheel variants, integrity material, or
platform/device matrix. Runtime code imports Pydantic directly, but only the
development requirements declare it. The TestClient's httpx dependency is not
declared. No dependency mutation is authorized in this slice.

Device discovery prefers CUDA, then MPS, then CPU for `auto`; explicit device
selection constructs a Torch device without proving backend availability or
CUDA index existence. MPS memory reporting uses system memory as a proxy.
Loaders use Transformers with `trust_remote_code=True`, and the names DLLM and
Sherry do not correspond to independently exercised loader behavior. Explicit
unknown `model_type` values fall through to the generic loader. No model
architecture, tokenizer, remote-code policy, CPU/CUDA/MPS class, or offline
fixture has required-real acceptance evidence.

## Decisions and First Slice

The bounded investigation stopping condition is met because every current
surface has a supported, rejected, or `unavailable` disposition:

1. Retain `/v1/models`, non-streaming `/v1/chat/completions`, and
   `/v1/completions` only as an OpenAI-shaped, text-only subset.
2. Close inbound request models over exact known fields. Retain only
   `system`/`user`/`assistant` string messages, one string prompt, implemented
   temperature/nucleus-sampling values, and positive bounded output tokens.
3. Reject non-null `stop` and `stream: true` as outside the supported subset;
   delete the manual streaming implementation because it then has no consumer
   or reachable contract.
4. Bound model identifiers to 256 characters, at most 256 chat messages,
   aggregate prompt/message text to 1,000,000 characters, and requested output
   to 4,096 tokens. These values belong to this direct sidecar request contract
   and do not expand the separate Pumas gateway's body budget.
5. Preserve response/usage repair, work admission and ownership, redacted
   outcomes, effective-versus-next configuration, shutdown, dependency
   resolution, and required-real system evidence for later M3 slices.
6. Keep Torch non-shipped until those later claims and one real tuple pass.

The first implementation slice changes only `openai_api.py` and focused public
request-model tests. It is accepted as incremental DRBT-A5 contract evidence,
not Milestone 3 acceptance. This is reversible and does not choose unproved
runtime mechanics. The Pydantic request types remain the single decoder
Interface; route and generation code receive only its validated representation.

The accepted decoder rejects unknown fields and coercion; limits model
identifiers to nonblank values of at most 256 characters; admits only bounded
string prompts and `system`/`user`/`assistant` text messages; bounds
temperature, nucleus sampling, and requested output; requires `top_p=1` when
`temperature=0`; and admits only `stream=false` plus `stop=null`. A separate
assistant response type permits the valid empty generated-output case instead
of leaking inbound content requirements into response construction. Removing
the now-unreachable stream helpers passes the deletion test without moving
their complexity elsewhere.

## Evidence

- `python3 --version` -> `Python 3.12.3`.
- `importlib.metadata` inventory -> missing production stack as tabulated
  above.
- `python3 -m unittest discover -s torch-server/tests -v` -> 13/13 passing,
  explicitly fake-backed and not accepted as runtime evidence.
- Repository-wide bounded `rg` consumer search -> control consumers found;
  no sidecar inference consumer found.
- Direct source inspection -> route, field, ignored-value, task, loader,
  installer, launch, and device findings above.
- Official OpenAI API reference links above -> external comparison authority.
- First request-contract red -> 14/15 focused public-model tests failed; review
  reds additionally exposed the response-type, blank-model, and deterministic
  sampling gaps.
- Accepted request-contract green -> 16/16 focused validation/application
  tests, including five new public request/response regressions, and 18/18 full
  fake-backed Torch unit tests passed; Python byte compilation and Ruff
  check/format checks passed for the two changed Python files.
- Evidence boundary -> the full unit suite still uses dependency fakes. No real
  ASGI, model, device, inference, scheduling, cancellation, usage, or shutdown
  claim is inferred from it.

## Module Review

- **Module:** the eventual Torch request-execution Module must own admission,
  validated execution, one terminal result, cancellation, and shutdown. Its
  stable Interface is one admitted request to one terminal response/stream
  outcome. That composition cannot yet be selected without real runtime facts.
- **Current Interface:** Pydantic request construction is the inbound ASGI
  decoder. Closing it is deeper than scattering checks through route and
  generation helpers because callers can rely on supported roles, ranges,
  extra-field policy, and cross-field behavior once construction succeeds.
- **Adapters:** Transformers, Torch device mechanisms, ASGI delivery, and any
  future worker/executor are infrastructure Adapters. None receives scheduling
  or compatibility policy ownership from its framework API.
- **Deletion test:** deleting the request decoder would force every route and
  helper to reinterpret raw untrusted input, so it contains necessary contract
  complexity. Deleting the manual stream helpers after `stream: true` is
  rejected makes complexity disappear without moving to callers; they have no
  retained owner and must be removed. A queue/executor/process Seam is not
  admitted until real model/device evidence shows which necessary complexity it
  contains.
- **Cumulative machinery:** the first slice adds no package, registry, worker,
  queue, executor, schema artifact, or compatibility shim. It strengthens the
  existing decoder and removes unsupported code.
