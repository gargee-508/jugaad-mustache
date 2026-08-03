# Differential Fuzz Log — mustache-rs vs. mustache.js

- Duration: 60.0s (continuous, minimum 60s required)
- Iterations run: 1531
- Shared API surface tested: variables, {{{unescaped}}} / {{&unescaped}}, sections (objects/arrays/booleans), inverted sections, comments, dotted-path lookups, nested contexts
- Excluded from scope: partials (CLI has no partial-loader, so it is not part of the shared API), custom delimiter changes (documented separately)
- Divergences found: 0

## Result: PASS — zero divergences across 1531 iterations.
