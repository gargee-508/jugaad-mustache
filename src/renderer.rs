use serde_json::Value;
use crate::parser::Node;

pub fn render(nodes: &[Node], data: &Value) -> String {
    let mut output = String::new();
    let context_stack = vec![data];
    render_nodes(nodes, &context_stack, &mut output);
    output
}

fn render_nodes(nodes: &[Node], context_stack: &[&Value], output: &mut String) {
    for node in nodes {
        match node {
            Node::Text(t) => {
                output.push_str(t);
            }
            Node::Comment(_) => {
                // Comments are ignored in rendering
            }
            Node::Variable(name) => {
                if let Some(val) = lookup(context_stack, name) {
                    let s = val_to_string(val);
                    output.push_str(&html_escape(&s));
                }
            }
            Node::UnescapedVariable(name) => {
                if let Some(val) = lookup(context_stack, name) {
                    let s = val_to_string(val);
                    output.push_str(&s);
                }
            }
            Node::Partial(name) => {
                render_partial(name, context_stack, output);
            }
            Node::Section { name, children, inverted } => {
                let val_opt = lookup(context_stack, name);
                let is_true = val_opt.map_or(false, is_truthy);

                if *inverted {
                    if !is_true {
                        render_nodes(children, context_stack, output);
                    }
                } else {
                    if is_true {
                        let val = val_opt.unwrap();
                        match val {
                            Value::Array(arr) => {
                                for item in arr {
                                    let mut new_stack = context_stack.to_vec();
                                    new_stack.push(item);
                                    render_nodes(children, &new_stack, output);
                                }
                            }
                            _ => {
                                let mut new_stack = context_stack.to_vec();
                                new_stack.push(val);
                                render_nodes(children, &new_stack, output);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_partial(name: &str, context_stack: &[&Value], output: &mut String) {
    let mut file_content = None;
    if let Ok(content) = std::fs::read_to_string(format!("{}.mustache", name)) {
        file_content = Some(content);
    } else if let Ok(content) = std::fs::read_to_string(name) {
        file_content = Some(content);
    }

    if let Some(content) = file_content {
        let tokens = crate::scanner::Scanner::scan(&content);
        if let Ok(nodes) = crate::parser::parse(tokens) {
            render_nodes(&nodes, context_stack, output);
        }
    }
}

pub fn lookup<'a>(context_stack: &[&'a Value], key: &str) -> Option<&'a Value> {
    if key == "." {
        return context_stack.last().copied();
    }
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    for ctx in context_stack.iter().rev() {
        if let Some(mut val) = ctx.get(parts[0]) {
            for &part in &parts[1..] {
                if let Some(next) = val.get(part) {
                    val = next;
                } else {
                    return None;
                }
            }
            return Some(val);
        }
    }
    None
}

pub fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                f != 0.0 && !f.is_nan()
            } else if let Some(i) = n.as_i64() {
                i != 0
            } else if let Some(u) = n.as_u64() {
                u != 0
            } else {
                true
            }
        }
        Value::Object(_) => true,
    }
}

pub fn val_to_string(val: &Value) -> String {
    match val {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let strs: Vec<String> = arr.iter().map(val_to_string).collect();
            strs.join(",")
        }
        Value::Object(_) => "[object Object]".to_string(),
    }
}

fn html_escape(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            '/' => result.push_str("&#x2F;"),
            _ => result.push(c),
        }
    }
    result
}
