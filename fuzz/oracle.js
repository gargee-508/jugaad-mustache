// Renders a template/data pair with the ORIGINAL mustache.js.
// Usage: node oracle.js <case.json>
// Prints the rendered output to stdout (nothing else), or exits 1 on error.
const fs = require("fs");
const Mustache = require("mustache");

const caseFile = process.argv[2];
const { template, data } = JSON.parse(fs.readFileSync(caseFile, "utf8"));

try {
  const out = Mustache.render(template, data);
  process.stdout.write(out);
} catch (e) {
  process.stderr.write("ORACLE_ERROR: " + e.message);
  process.exit(1);
}
