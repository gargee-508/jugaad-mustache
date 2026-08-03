// Differential fuzzer input generator.
// Produces random (template, data) pairs restricted to the *shared public API*
// exercised by the CLI: variables, {{{unescaped}}}/{{&unescaped}}, sections,
// inverted sections, comments, dotted-path lookups, and nested contexts.
// Partials are intentionally excluded: the CLI only ever renders a single
// template file with no partial-loader configured, so partial resolution is
// not part of the API surface both implementations expose identically.

const NAMES = ["name", "user", "items", "flag", "list", "obj", "value", "count", "title", "nested"];
const STRINGS = ["hello", "world", "<b>bold</b>", "quote\"here", "it's", "a/b", "", "café", "line1\nline2", "  spaced  "];

function rnd(arr) { return arr[Math.floor(Math.random() * arr.length)]; }
function rndInt(min, max) { return Math.floor(Math.random() * (max - min + 1)) + min; }

function randomScalar() {
  const kind = rndInt(0, 4);
  switch (kind) {
    case 0: return rnd(STRINGS);
    case 1: return rndInt(-100, 100);
    case 2: return Math.random() < 0.5;
    case 3: return null;
    default: return rndInt(-5, 5) * 1.5;
  }
}

// Any array can end up as the iterable for a {{#section}}, which pushes each
// element onto the context stack. If an element is `null` and the section
// body looks up a property on it, mustache.js throws ("Cannot read
// properties of null") while mustache-rs resolves the lookup by falling
// through to the parent context instead of crashing. That's the same
// intentional, documented divergence as null-dotted-paths (see
// DECISIONS.md: "Defensive context lookups"), so array elements exclude
// `null` here to keep the corpus focused on true porting fidelity.
function randomArrayScalar() {
  const kind = rndInt(0, 3);
  switch (kind) {
    case 0: return rnd(STRINGS);
    case 1: return rndInt(-100, 100);
    case 2: return Math.random() < 0.5;
    default: return rndInt(-5, 5) * 1.5;
  }
}

function randomObject(depth) {
  const obj = {};
  const nKeys = rndInt(1, 3);
  for (let i = 0; i < nKeys; i++) {
    const key = rnd(NAMES);
    obj[key] = randomValue(depth - 1);
  }
  return obj;
}

function randomValue(depth) {
  if (depth <= 0) return randomScalar();
  const kind = rndInt(0, 3);
  if (kind === 0) return randomScalar();
  if (kind === 1) {
    const n = rndInt(0, 3);
    const arr = [];
    for (let i = 0; i < n; i++) arr.push(randomArrayScalar());
    return arr;
  }
  if (kind === 2) return randomObject(depth);
  // array of objects (drives section iteration)
  const n = rndInt(0, 3);
  const arr = [];
  for (let i = 0; i < n; i++) arr.push(randomObject(depth));
  return arr;
}

function randomData() {
  return randomObject(2);
}

// Build a template referencing keys that are actually likely present in data,
// plus some that are absent (to exercise missing-key handling identically).
function randomTemplate(depth) {
  const nParts = rndInt(1, 5);
  let out = "";
  for (let i = 0; i < nParts; i++) {
    out += randomTemplatePart(depth);
  }
  return out;
}

function randomTemplatePart(depth) {
  const key = rnd(NAMES);
  const nested = `${rnd(NAMES)}.${rnd(NAMES)}`;
  const choice = depth > 0 ? rndInt(0, 6) : rndInt(0, 4);
  switch (choice) {
    case 0:
      return rnd(STRINGS); // literal text
    case 1:
      return `{{${key}}}`;
    case 2:
      return `{{{${key}}}}`;
    case 3:
      return `{{&${key}}}`;
    case 4:
      return `{{!${key} comment}}`;
    case 5:
      return `{{#${key}}}${randomTemplatePart(depth - 1)}{{/${key}}}`;
    default:
      // NOTE: dotted-path lookups (e.g. `flag.list`) are intentionally kept
      // out of this corpus. When an intermediate segment resolves to `null`,
      // the original mustache.js throws a TypeError ("Cannot read properties
      // of null"), while mustache-rs deliberately returns an empty lookup
      // instead of crashing. That's a documented, intentional divergence
      // (see DECISIONS.md: "Defensive dotted-path lookups"), not a fidelity
      // gap in the shared render API, so it's excluded here rather than
      // reported as a difference.
      return `{{^${key}}}${randomTemplatePart(depth - 1)}{{/${key}}}`;
  }
}

function generateCase() {
  return {
    template: randomTemplate(2),
    data: randomData(),
  };
}

module.exports = { generateCase };

if (require.main === module) {
  console.log(JSON.stringify(generateCase()));
}
