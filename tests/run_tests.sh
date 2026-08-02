#!/bin/bash
# Run original mustache.js test cases against our Rust binary

BINARY="./target/release/mustache"
PASS=0
FAIL=0

# Ensure the temp directory exists
mkdir -p /tmp

run_test() {
    local name="$1"
    local template="$2"
    local data="$3"
    local expected="$4"

    echo "$data" > /tmp/test_data.json
    echo "$template" > /tmp/test_template.mustache
    actual=$("$BINARY" /tmp/test_data.json /tmp/test_template.mustache 2>&1)

    if [ "$actual" = "$expected" ]; then
        echo "✅ PASS: $name"
        ((PASS++))
    else
        echo "❌ FAIL: $name"
        echo "  Expected: $expected"
        echo "  Got:      $actual"
        ((FAIL++))
    fi
}

# Core tests from mustache.js spec
run_test "basic variable" "Hello, {{name}}!" '{"name":"World"}' "Hello, World!"
run_test "html escape" "{{name}}" '{"name":"<b>bold</b>"}' "&lt;b&gt;bold&lt;&#x2F;b&gt;"
run_test "unescaped variable" "{{{name}}}" '{"name":"<b>bold</b>"}' "<b>bold</b>"
run_test "section truthy" "{{#show}}yes{{/show}}" '{"show":true}' "yes"
run_test "section falsy" "{{#show}}yes{{/show}}" '{"show":false}' ""
run_test "section list" "{{#items}}{{.}} {{/items}}" '{"items":["a","b","c"]}' "a b c "
run_test "inverted section" "{{^empty}}not empty{{/empty}}" '{"empty":false}' "not empty"
run_test "comment" "before{{! comment }}after" '{}' "beforeafter"
run_test "dotted path" "{{person.name}}" '{"person":{"name":"Rahul"}}' "Rahul"
run_test "empty string" "{{name}}" '{"name":""}' ""
run_test "missing key" "{{missing}}" '{}' ""
run_test "nested sections" "{{#a}}{{#b}}{{{c}}}{{/b}}{{/a}}" '{"a":{"b":{"c":"deep"}}}' "deep"

echo ""
echo "Results: $PASS passed, $FAIL failed"
if [ "$FAIL" -eq 0 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    exit 1
fi
