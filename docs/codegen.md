# Why Hand-Written Types

## Background

We initially built a full codegen pipeline (TypeScript → JSON Schema → `typify` → Rust types), plus a separate script to extract command/response pairings and generate a `PiAgent` trait. It worked end-to-end but was abandoned because:

1. **typify output was unusable** — 9500 lines with wrong serde strategies (`#[serde(untagged)]` instead of `#[serde(tag = "role")]`), meaningless variant names, and unnecessary builder patterns.
2. **The pipeline input required manual maintenance anyway** — `ts-json-schema-generator` couldn't follow cross-package `.d.ts` imports or resolve declaration merging, so we had to maintain a hand-assembled `pi-types.ts` consolidation file.
3. **Disproportionate infrastructure** — multiple TypeScript scripts, npm dependencies, intermediate JSON files, and post-processing steps to produce worse output than ~500 lines of hand-written Rust.

## Current approach

Rust types in `src/types/` are hand-written to closely mirror pi's TypeScript sources. Each `.rs` file corresponds to a specific `.ts` file, with comments marking the correspondence. See [src/types/README.md](../src/types/README.md) for the complete file mapping, version info, and update instructions.

When pi releases a new version, diff the TypeScript sources against the Rust types and update by hand. The file-by-file correspondence makes this straightforward.
