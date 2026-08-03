# Porting Analysis: JavaScript (mustache.js) vs Rust (mustache-rs)

This document provides a side-by-side comparison of the original JavaScript implementation ([janl/mustache.js](https://github.com/janl/mustache.js)) and this Rust port (**jugaad-mustache**) for track evaluation.

---

## 1. Architectural Mapping

| Component | original `mustache.js` (JavaScript) | `mustache-rs` (Rust Port) | Equivalency & Design Note |
| :--- | :--- | :--- | :--- |
| **CLI & Entry** | Uses `bin/mustache` & `commander.js` | [src/main.rs](src/main.rs) using `clap` | Handles file inputs and CLI flags (`--jugaad`, `--explain`) natively without Node.js. |
| **Token Scanner** | `Scanner` class in `mustache.js` | [src/scanner.rs](src/scanner.rs) | Two-pass parser-guided scanner. Retains delimiter changes (`{{=<% %>=}}`). |
| **AST Parser** | `parseTemplate()` in `mustache.js` | [src/parser.rs](src/parser.rs) | Translates flat tokens to nested blocks (`Section`). Mismatched tags return compile errors. |
| **Context Scope** | `Context` class in `mustache.js` | `context_stack: Vec<&Value>` | Scope hierarchy walker that supports parent inheritance and dotted path lookups. |
| **HTML Escaper** | `escapeHtml` (entity map replacement) | `html_escape` character matching | Escapes `& < > " ' /` in exact order to protect against XSS injection. |
| **Writer/Renderer** | `Writer` class in `mustache.js` | [src/renderer.rs](src/renderer.rs) | Traverses the AST recursively to construct the final output against JSON values. |

---

## 2. Core Implementation Comparison

### A. Scanning & Delimiters
In `mustache.js`, scanning is stateful, tracking the index and reading substrings using regular expressions:
```javascript
// mustache.js
Scanner.prototype.scanUntil = function (re) {
  var match = this.tail.match(re);
  // ... stateful pointer movement ...
};
```
In `mustache-rs`, we implemented a stateful Rust scanner that matches the delimiter strings index-by-index to eliminate regular expression overhead and enforce compile-time safety:
```rust
// src/scanner.rs
if let Some(open_idx) = remaining.find(&open_del) {
    // Stage preceding literal text
    remaining = &remaining[open_idx + open_del.len()..];
    // ... resolve delimiters and extract tag ...
}
```

### B. Context & Scoping (Dotted Path Lookups)
In `mustache.js`, contexts are nested recursively:
```javascript
// mustache.js Context lookup
Context.prototype.lookup = function (name) {
  var value = this._cache[name];
  // ... walk up context hierarchy ...
};
```
In `mustache-rs`, we model this using a vector of JSON references (`&Value`), walking backwards (from leaf to root) to search for variables, supporting nested structures (`person.name`) and the current element context (`.`):
```rust
// src/renderer.rs
pub fn lookup<'a>(context_stack: &[&'a Value], key: &str) -> Option<&'a Value> {
    if key == "." { return context_stack.last().copied(); }
    let parts: Vec<&str> = key.split('.').collect();
    for ctx in context_stack.iter().rev() {
        if let Some(mut val) = ctx.get(parts[0]) {
            for &part in &parts[1..] {
                val = val.get(part)?;
            }
            return Some(val);
        }
    }
    None
}
```

---

## 3. Added Innovation: Jugaad (Fault-Tolerance) Mode
While the original `mustache.js` crashes or produces broken HTML on malformed templates (e.g. unclosed sections or orphaned tags), `mustache-rs` introduces **Jugaad Mode** (`--jugaad`). 

This pre-processor resolves common templates faults safely before feeding them to the scanner:
1. **Unclosed variables (`{{name`)**: Automatically appends closing delimiters.
2. **Unclosed sections (`{{#section}}`)**: Appends the corresponding closing tags at EOF.
3. **Orphan tags (`{{/section}}`)**: Removes dangling closing tags.
4. **Whitespace cleanup (`{{ name }}`)**: Normalizes spaces around tag names to avoid lookup errors.
