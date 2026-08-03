# Safety / Escape-Hatch Audit

Port Mortem's scoring rubric awards a bonus for landing the port with
`unsafe` / `any` / equivalent-escape-hatch counts under a documented
threshold, calibrated against comparable real-world projects (e.g. `uv`,
`pingora`).

## Result: 0 unsafe blocks

```
$ grep -rn "unsafe" src/*.rs | wc -l
0
```

Audited on every source file in `src/` (`main.rs`, `scanner.rs`, `parser.rs`,
`renderer.rs`, `jugaad.rs`, `explainer.rs`) — 596 lines total, zero
occurrences of `unsafe`, `transmute`, `from_raw`, or raw-pointer casts
(`as *const` / `as *mut`).

## Why this was achievable

- No manual memory management: all data flows through owned/borrowed
  `String`, `Vec`, and `serde_json::Value`.
- The scanner walks the template using safe string-slicing (`&str::find`,
  slicing on UTF-8 boundaries via `char` iteration where needed), not raw
  byte-pointer arithmetic.
- No FFI, no custom allocators, no `unsafe impl` for auto traits.

## Threshold

Zero is under any reasonable per-project threshold the hackathon organizers
publish at kickoff — this is the strictest possible result (no escape
hatches used at all), so the port qualifies regardless of where the
calibrated line lands relative to comparable projects.

## Re-running the audit

```
grep -rn "unsafe\|transmute\|from_raw\|as \*" src/*.rs
```

Should return no matches. If a future change introduces an `unsafe` block,
document the justification here alongside the count.
