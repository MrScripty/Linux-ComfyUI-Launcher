# Generated Bindings

This directory is the local output and test area for experimental Pumas host
bindings. The stable implementation boundary is the Rust adapter in
`rust/crates/pumas-uniffi`; generated language files are build artifacts.

Supported generators currently target Python, Kotlin, Swift, Ruby, and C#.
There is also an experimental Rustler crate for Elixir/Erlang, but it does not
currently expose the core library API. None of these surfaces has a complete
host/runtime support matrix yet.

## Generate

From the repository root:

```bash
./scripts/generate-bindings.sh python
./scripts/generate-bindings.sh csharp
./scripts/generate-bindings.sh all
```

Generation builds `pumas-uniffi` and may require separately installed,
version-compatible bindgen tools. Generated files and their native library are
one compatibility cohort: do not combine output from different commits,
versions, targets, or profiles.

## Check and Package

```bash
./scripts/check-uniffi-surface.sh
./scripts/check-uniffi-csharp-smoke.sh
./scripts/package-uniffi-csharp-artifacts.sh
```

The C# smoke proves that generated code compiles, the matching native library
loads, and a synchronous version call works. It does not prove async APIs or a
complete model-library workflow.

Generated output belongs in ignored directories and must not be hand-edited.
See [Native bindings](../docs/native-bindings.md) for the boundary, runtime
loading, packaging, and promotion requirements.
