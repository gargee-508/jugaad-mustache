use crate::parser::Node;
use serde_json::Value;

pub fn explain(nodes: &[Node], data: &Value) {
    println!("🔍 Template breakdown:");
    explain_nodes(nodes, data, 0);
    println!();
}

fn explain_nodes(nodes: &[Node], data: &Value, depth: usize) {
    let indent = " ".repeat(depth + 1);
    for node in nodes {
        match node {
            Node::Text(t) => {
                if !t.trim().is_empty() {
                    println!("{}\"{}\" -> literal text, copied as-is", indent, t.trim());
                }
            }
            Node::Variable(name) => {
                let val = lookup_explain(data, name);
                println!("{}{{{{{}}}}} -> variable lookup -> {}", indent, name, val);
            }
            Node::UnescapedVariable(name) => {
                let val = lookup_explain(data, name);
                println!("{}{{{{{{{}}}}}}} -> raw variable (no HTML escape) -> {}", indent, name, val);
            }
            Node::Section { name, children, inverted: false } => {
                println!("{}{{{{#{}}}}} -> section -> checking if truthy...", indent, name);
                explain_nodes(children, data, depth + 1);
                println!("{}{{{{/{}}}}} -> end section", indent, name);
            }
            Node::Section { name, children, inverted: true } => {
                println!("{}{{{{^{}}}}} -> inverted section -> checking if falsy...", indent, name);
                explain_nodes(children, data, depth + 1);
                println!("{}{{{{/{}}}}} -> end section", indent, name);
            }
            Node::Comment(c) => {
                println!("{}{{{{! {} }}}} -> comment, not rendered", indent, c.trim());
            }
            Node::Partial(name) => {
                println!("{}{{{{> {}}}}} -> partial template '{}'", indent, name, name);
            }
        }
    }
}

fn lookup_explain(data: &Value, name: &str) -> String {
    match crate::renderer::lookup(&[data], name) {
        Some(Value::String(s)) => format!("found \"{}\"", s),
        Some(val) => format!("found {}", val),
        None => "not found".to_string(),
    }
}
