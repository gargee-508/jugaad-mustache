// Differential fuzzer: mustache-rs (this port) vs. mustache.js (original).
// Runs continuously for MIN_SECONDS, feeding both implementations identical
// (template, data) pairs from generate.js and diffing their stdout.
//
// Usage: node run_fuzz.js [minSeconds]
// Writes a full log to fuzz/fuzz_log.md

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const Mustache = require("mustache");
const { generateCase } = require("./generate");

const MIN_SECONDS = parseInt(process.argv[2] || "60", 10);
const BINARY = path.join(__dirname, "..", "target", "release", "mustache");
const TMP_DATA = path.join(__dirname, "case_data.json");
const TMP_TPL = path.join(__dirname, "case_template.mustache");
const LOG_PATH = path.join(__dirname, "fuzz_log.md");

function runRustPort(template, data) {
  fs.writeFileSync(TMP_DATA, JSON.stringify(data));
  fs.writeFileSync(TMP_TPL, template);
  try {
    const out = execFileSync(BINARY, [TMP_DATA, TMP_TPL], { encoding: "utf8" });
    return { ok: true, out };
  } catch (e) {
    return { ok: false, out: (e.stdout || "") + (e.stderr || "") };
  }
}

function runOriginal(template, data) {
  try {
    const out = Mustache.render(template, data);
    return { ok: true, out };
  } catch (e) {
    return { ok: false, out: "ORACLE_ERROR: " + e.message };
  }
}

const start = Date.now();
let iterations = 0;
let divergences = [];

while ((Date.now() - start) / 1000 < MIN_SECONDS) {
  const { template, data } = generateCase();
  const rust = runRustPort(template, data);
  const orig = runOriginal(template, data);
  iterations++;

  if (rust.ok !== orig.ok || (rust.ok && orig.ok && rust.out !== orig.out)) {
    divergences.push({
      iteration: iterations,
      template,
      data,
      rust,
      orig,
    });
  }
}

const durationSec = ((Date.now() - start) / 1000).toFixed(1);

let log = `# Differential Fuzz Log — mustache-rs vs. mustache.js\n\n`;
log += `- Duration: ${durationSec}s (continuous, minimum ${MIN_SECONDS}s required)\n`;
log += `- Iterations run: ${iterations}\n`;
log += `- Shared API surface tested: variables, {{{unescaped}}} / {{&unescaped}}, ` +
       `sections (objects/arrays/booleans), inverted sections, comments, ` +
       `dotted-path lookups, nested contexts\n`;
log += `- Excluded from scope: partials (CLI has no partial-loader, so it is not ` +
       `part of the shared API), custom delimiter changes (documented separately)\n`;
log += `- Divergences found: ${divergences.length}\n\n`;

if (divergences.length === 0) {
  log += `## Result: PASS — zero divergences across ${iterations} iterations.\n`;
} else {
  log += `## Result: ${divergences.length} divergence(s) found\n\n`;
  divergences.slice(0, 20).forEach((d) => {
    log += `### Iteration ${d.iteration}\n`;
    log += "```\ntemplate: " + JSON.stringify(d.template) + "\n";
    log += "data:     " + JSON.stringify(d.data) + "\n";
    log += "rust:     " + JSON.stringify(d.rust.out) + (d.rust.ok ? "" : " (error)") + "\n";
    log += "original: " + JSON.stringify(d.orig.out) + (d.orig.ok ? "" : " (error)") + "\n";
    log += "```\n\n";
  });
  if (divergences.length > 20) {
    log += `_...and ${divergences.length - 20} more (truncated)._\n`;
  }
}

fs.writeFileSync(LOG_PATH, log);
try { fs.unlinkSync(TMP_DATA); } catch (_) {}
try { fs.unlinkSync(TMP_TPL); } catch (_) {}

console.log(log);
console.log(`\nLog written to ${LOG_PATH}`);
process.exit(divergences.length === 0 ? 0 : 1);
