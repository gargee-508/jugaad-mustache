# Engineering Decisions

This document lists the non-trivial architectural divergences between
`mustache-rs` and the original `mustache.js`, and the rationale for each.

## 1. Static binary distribution instead of a Node.js runtime
Track F allows both Rust and Go. Rust produces a fully static binary with
zero runtime dependencies — this directly satisfies the hackathon's goal of
eliminating the Node.js runtime requirement entirely (`cargo build --release`
→ single executable, no `node_modules`, no interpreter on the target
machine).

## 2. CLI parsing: `clap` instead of `commander.js`
mustache.js's CLI wrapper uses `commander.js`. `clap` (derive macros) is the
idiomatic Rust equivalent with identical flag-parsing capability, but it is a
different library with different error-formatting and help-text generation
than the original.

## 3. JSON handling: `serde_json` instead of native `JSON.parse`
JSON parsing is not the core problem this port solves, so `serde_json` — the
standard, battle-tested Rust JSON library — was used rather than
hand-rolling a parser. This means JSON edge cases (e.g. duplicate keys,
number precision) follow `serde_json`'s rules rather than V8's.

## 4. Scanner: direct string-index scanning instead of regex
mustache.js's `Scanner.scanUntil` is regex-driven (`this.tail.match(re)`).
`mustache-rs` reimplements the same two-pass architecture (split on
delimiters, then classify each token by sigil) but replaces every regex call
with direct substring search (`str::find`). This avoids pulling in a regex
engine and its associated unsafe/backtracking surface, at the cost of
needing to hand-roll delimiter-change handling that regex would do for free.

## 5. HTML escaping character set & order
We escape the same 6 characters as mustache.js: `& < > " ' /`, in the same
order (`&` first, to avoid double-escaping already-escaped entities). This
is an intentional match rather than a divergence, but is called out because
Rust has no built-in HTML-escaping primitive — the table was hand-written to
mirror mustache.js's `entityMap` exactly.

## 6. Truthy/falsy coercion for numbers is implemented explicitly
JavaScript's `!value` gives free truthiness coercion (`0` and `NaN` are
falsy). Rust has no implicit truthiness, so `is_truthy()` in `renderer.rs`
explicitly special-cases `0`, `0.0`, and `NaN` as falsy to match JS section
semantics. This is a deliberate divergence from idiomatic Rust (which would
normally require an explicit non-zero check at each call site) in favor of
byte-for-byte behavioral compatibility.

## 7. Object stringification mirrors JS's default `toString()`
When a JSON object is rendered directly as a variable (rather than entered
as a section context), mustache.js falls back to JS's default
`Object.prototype.toString`, producing the literal string `[object Object]`.
`mustache-rs` replicates this literal string rather than doing the more
"idiomatic Rust" thing of serializing the object back to JSON — an explicit
compatibility shim rather than the natural Rust behavior.

## 8. Array stringification via comma-join, not Rust's `Debug` format
Rendering an array directly (outside a section) uses JS's
`Array.prototype.join(',')` semantics — elements joined with `,`, no
brackets. `mustache-rs` implements this explicitly in `val_to_string`; the
idiomatic Rust default (`{:?}` debug formatting, e.g. `[1, 2, 3]`) was
rejected in favor of matching the original's output byte-for-byte.

## 9. Defensive context lookups on `null` (found via differential fuzzing)
When a dotted-path lookup or a section-context traversal hits a `null`
value, mustache.js throws (`TypeError: Cannot read properties of null`)
because JS property access on `null` is a runtime error. `mustache-rs`'s
`lookup()` instead falls through to the parent context and ultimately
returns `None`, rendering nothing rather than crashing. This divergence was
surfaced by the differential fuzzer (see `fuzz/fuzz_log.md`) and is a
deliberate choice: crashing the CLI over a null intermediate value is worse
UX than an empty render, so the corpus in `fuzz/generate.js` documents and
excludes this known/intentional difference rather than "fixing" it to match
a crash.

## 10. Partial resolution via filesystem read, not an explicit partials map
`Mustache.render(template, view, partials)` in the original takes partials
as an explicit JS object supplied by the caller. `mustache-rs` has no
equivalent CLI flag for a partials map, so `render_partial` instead reads
`{name}.mustache` or `{name}` directly off disk relative to the working
directory. This trades explicit-dependency-injection for CLI convenience,
and is out of scope for the differential fuzzer (see `fuzz/generate.js`)
since it isn't part of the API surface the CLI exposes identically on both
sides.

## 11. Jugaad mode (new feature, not present in the original)
Named after the Indian philosophy of frugal innovation. mustache.js crashes
on malformed templates. `--jugaad` pre-processes the template to fix common
errors (unclosed tags/sections, orphan closing tags, stray whitespace)
before parsing. It is strictly opt-in and does not affect default behavior
or any original test — a pure architectural addition, not a compatibility
requirement.

## Memory safety posture
Zero `unsafe` blocks anywhere in `src/`. See `SAFETY.md` for the full audit
and how it satisfies the hackathon's escape-hatch threshold.

## AI assistance
Claude was used to scaffold all modules. Every AI output was verified
against the test suite. This document was written by hand to explain every
choice.
