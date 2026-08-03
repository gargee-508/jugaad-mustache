# mustache-rs 🦀

> mustache.js ported from JavaScript to Rust.
> Same behavior. No Node.js required. Single binary.

## Interactive Verification Portal
Try the comparative playground directly in your browser:
👉 **[Live Demo Playground](https://gargee-508.github.io/jugaad-mustache/)**

## Why
mustache.js requires Node.js + npm install + node_modules to run.
mustache-rs ships as a single static binary. Clone and run.

## Install
cargo build --release

## Usage
./mustache data.json template.mustache

## Flags
--jugaad    Fix broken templates instead of crashing
--explain   Show what each part of the template does

## Examples

### Basic
echo '{"name":"Rahul"}' > data.json
echo 'Hello, {{name}}!' > template.mustache
./mustache data.json template.mustache
# Hello, Rahul!

### Jugaad mode
./mustache --jugaad data.json broken_template.mustache
# ⚡ Jugaad mode activated!
# 🔧 Unclosed tag... theek kar diya (fixed it)
# Hello, Rahul!
# 😎 Kaam ho gaya. (Job done.)

### Explain mode
./mustache --explain data.json template.mustache
# 🔍 Template breakdown:
#  "Hello, " -> literal text, copied as-is
#  {{name}} -> variable lookup -> found "Rahul"
# ⚙️ Executing...
# Hello, Rahul!

## Test Results
All original mustache spec tests passing

## Repository Comparison
See [COMPARISON.md](COMPARISON.md) for a side-by-side comparison with the original `mustache.js` implementation.

## Engineering Decisions
See [DECISIONS.md](DECISIONS.md)

## Bonus Points

### Differential fuzzing (60+ continuous seconds, zero divergences)
`fuzz/run_fuzz.js` generates random templates/data and renders each one
through both mustache.js (the original) and this Rust port, byte-comparing
the output. Run it yourself:
```
cd fuzz && npm install && node run_fuzz.js 60
```
Latest run: **30,567 iterations over 65s, 0 divergences** on the shared
render API (variables, unescaped variables, sections, inverted sections,
comments, dotted-path lookups, nested contexts). Full log:
[fuzz/fuzz_log.md](fuzz/fuzz_log.md). Scope notes and one intentional,
documented divergence (defensive `null` context handling) are in
[DECISIONS.md](DECISIONS.md#9-defensive-context-lookups-on-null-found-via-differential-fuzzing).

### Escape-hatch / unsafe threshold
Zero `unsafe` blocks across all of `src/`. See [SAFETY.md](SAFETY.md) for
the full audit.
