# mustache-rs 🦀

[![CI](https://github.com/gargee-508/Jugaad_Mustache/actions/workflows/ci.yml/badge.svg)](https://github.com/gargee-508/Jugaad_Mustache/actions/workflows/ci.yml)
[![Tests](https://img.shields.io/badge/mustache%20spec-100%25-brightgreen)](tests/)
[![unsafe](https://img.shields.io/badge/unsafe%20blocks-0-brightgreen)](src/)
[![YouTube Demo](https://img.shields.io/badge/YouTube-Video%20Demo-red?logo=youtube)](https://youtu.be/sOZluEz885s)
[![Track F](https://img.shields.io/badge/Port%20Mortem%202026-Track%20F-orange)](https://portmortem.dev)

> A complete port of `mustache.js` from JavaScript to Rust.
> 100% spec compliance. Zero unsafe blocks. No Node.js required.
> Built for [Port Mortem 2026](https://portmortem.dev), Track F.

**[🌐 Live Verification Portal →](https://gargee-508.github.io/Jugaad_Mustache/)** | **[📺 YouTube Video Demo →](https://youtu.be/sOZluEz885s)** | **[📖 Developer Journey Blog →](BLOG.md)**
Try mustache templates live in your browser. Side-by-side JS vs Rust output. Real diffs.

---

## Results at a glance

| Metric | Value |
|--------|-------|
| Mustache spec compliance | **100%** — all spec tests passing |
| unsafe blocks | **0** — compiler-enforced via `#![forbid(unsafe_code)]` |
| Fuzz runs | **30,567** — zero divergences, zero panics |
| Binary size | **~2 MB** vs 47 MB (node_modules) |
| Startup time | **~5ms** vs ~150ms Node.js (30× faster) |
| Node.js required | **No** — single static binary |
| Bonus claims | Zero Unsafe +5, Fuzz Survivor +5, Decision Log +3 |

---

## Why this exists

`mustache.js` is one of the most widely used templating libraries in the
JavaScript ecosystem — but it requires Node.js, npm, and node_modules just
to render a template.

`mustache-rs` ships as a **single static binary**. Clone, build, run.
No runtime. No package manager. No 47MB of dependencies.

```bash
# Before (mustache.js)
npm install mustache         # downloads node_modules
node -e "require('mustache').render(...)"

# After (mustache-rs)
./mustache data.json template.mustache   # that's it
```

---

## Performance vs original

For a CLI tool, **startup time is the real metric**.
A template renderer that starts in 5ms vs 150ms is a completely
different user experience in CI pipelines and shell scripts.

| Metric | mustache.js (Node.js) | mustache-rs (Rust) | Speedup |
|--------|-----------------------|--------------------|---------|
| Startup (cold) | ~150ms | ~5ms | **30×** |
| Small template (1 KB) | ~160ms | ~5.5ms | **29×** |
| Medium template (10 KB) | ~175ms | ~8ms | **22×** |
| Large template (100 KB) | ~280ms | ~25ms | **11×** |
| Peak RSS | ~45 MB | ~4 MB | **11× less** |
| Binary + runtime | 47 MB (node_modules) | ~2 MB | **23× smaller** |

Full methodology: [`docs/benchmark.md`](docs/benchmark.md)

---

## Quick start

```bash
# Build (one command)
cargo build --release

# Render a template
echo '{"name":"World"}' > data.json
echo 'Hello, {{name}}!' > template.mustache
./target/release/mustache data.json template.mustache
# → Hello, World!

# Or via stdin
echo '{"name":"World"}' | ./target/release/mustache - template.mustache

# Test
cargo test

# Differential fuzz (60 seconds)
cd fuzz && npm install && node run_fuzz.js 60
```

---

## Mustache spec compliance

**100% — all spec tests passing unmodified.**

| Section | Tests | Status |
|---------|-------|--------|
| Comments | 10 | ✅ 100% |
| Delimiters | 14 | ✅ 100% |
| Interpolation | 28 | ✅ 100% |
| Inverted sections | 11 | ✅ 100% |
| Partials | 12 | ✅ 100% |
| Sections | 24 | ✅ 100% |
| **Total** | **99** | **✅ 100%** |

The original test suite is hashed at kickoff and never modified.

---

## Architecture

Two-stage pipeline matching mustache.js internals exactly:

```
TEMPLATE STRING + DATA (JSON)
         │
         ▼
┌─────────────────────┐
│      PARSER         │  src/parser.rs
│                     │
│  Tokenize template  │
│  into tagged spans: │
│  Text, Variable,    │
│  Section, Partial,  │
│  Comment, Delim     │
└──────────┬──────────┘
           │ Vec<Token>
           ▼
┌─────────────────────┐
│      RENDERER       │  src/renderer.rs
│                     │
│  Walk token tree    │
│  Resolve variables  │
│  against JSON data  │
│  Handle sections    │
│  (truthy/falsy/list)│
└──────────┬──────────┘
           │ String (rendered output)
           ▼
        OUTPUT
```

See [`DECISIONS.md`](DECISIONS.md) for all architectural decisions.

---

## Zero unsafe — how and why

```rust
// src/main.rs — first line
#![forbid(unsafe_code)]
```

This is a **compiler error**, not a lint. Any `unsafe` block makes the
build fail. It cannot be bypassed.

How we achieved zero unsafe in a template renderer:
- All string building via `String::push_str()` — no raw pointer writes
- All JSON traversal via `serde_json` safe API
- Zero pointer arithmetic, zero `transmute`, zero unchecked indexing
- Every index into a Vec uses `.get()` with explicit `None` handling

---

## CLI flags

```
mustache [FLAGS] <data.json> <template.mustache>

FLAGS:
  --jugaad    Auto-fix common template errors instead of crashing
              (unclosed tags, missing variables, malformed sections)
  --explain   Show a step-by-step parse trace of the template
  --version   Print version
  --help      Print help
```

### --jugaad mode (error recovery)

```bash
$ cat broken.mustache
Hello, {{name}!   ← unclosed tag

$ ./mustache --jugaad data.json broken.mustache
⚡ Auto-fixed: unclosed tag '{{name}' → '{{name}}'
Hello, World!
```

### --explain mode (parse trace)

```bash
$ ./mustache --explain data.json template.mustache
[TEXT]     "Hello, "
[VARIABLE] {{name}} → "World"
[TEXT]     "!"
→ "Hello, World!"
```

---

## Differential fuzzing

```bash
cd fuzz && npm install && node run_fuzz.js 60
```

**Results: 30,567 iterations over 65 seconds, 0 divergences**

Inputs tested:
- Random variable names and data values
- Nested sections (truthy, falsy, list)
- Inverted sections with edge case data
- Dotted-path lookups (`{{a.b.c}}`)
- Partial templates
- Custom delimiters (`{{= | | =}}`)
- Empty string, null, undefined context values

One documented intentional divergence (defensive null context handling)
is explained in [DECISIONS.md §9](DECISIONS.md#9-defensive-context-lookups).

Full fuzz log: [`fuzz/fuzz_log.md`](fuzz/fuzz_log.md)

---

## Bonus claims

| Bonus | Evidence |
|-------|----------|
| ✅ Zero Unsafe (+5) | `#![forbid(unsafe_code)]` in `src/main.rs` — compiler verified |
| ✅ Differential Fuzz Survivor (+5) | `fuzz/fuzz_log.md` — 30,567 runs, 0 divergences |
| ✅ Decision Log (+3) | `DECISIONS.md` — 11 non-trivial architectural entries |

---

## Decisions

See [`DECISIONS.md`](DECISIONS.md) for all architectural decisions.

Key entries judges should read:
- **§3** — Why we parse to an AST instead of single-pass rendering
- **§7** — How we handle the falsy/truthy section ambiguity in mustache spec
- **§9** — The one documented divergence found via fuzzing (null contexts)

---

## Project files

```
mustache-rs/
├── src/
│   ├── main.rs         ← #![forbid(unsafe_code)], CLI entrypoint
│   ├── scanner.rs      ← Template tokenizer
│   ├── parser.rs       ← Stack-based parser & AST
│   ├── renderer.rs     ← Context-aware scope walker
│   ├── jugaad.rs       ← Fault-tolerant pre-processor
│   └── explainer.rs    ← Trace breakdown generator
├── tests/
│   └── run_tests.sh    ← Verification test harness
├── fuzz/
│   ├── run_fuzz.js     ← Differential fuzzer
│   └── fuzz_log.md     ← 65s run, 0 divergences
├── docs/
│   ├── index.html      ← Live comparative web portal
│   └── benchmark.md    ← Measurement methodology
├── DECISIONS.md        ← 11 architectural decisions
├── COMPARISON.md       ← Side-by-side with mustache.js
├── SAFETY.md           ← unsafe audit
├── Cargo.toml
└── Cargo.lock
```

---

## Comparison with mustache.js

See [`COMPARISON.md`](COMPARISON.md) for a side-by-side analysis of
architectural differences between mustache.js and mustache-rs.
