use crate::scanner::Token;

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Variable(String),
    UnescapedVariable(String),
    Section { name: String, children: Vec<Node>, inverted: bool },
    Comment(String),
    Partial(String),
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Node>, String> {
    let mut stack: Vec<(String, Vec<Node>, bool)> = Vec::new();
    let mut current_children: Vec<Node> = Vec::new();

    for token in tokens {
        match token {
            Token::Text(t) => {
                current_children.push(Node::Text(t));
            }
            Token::Variable(v) => {
                current_children.push(Node::Variable(v));
            }
            Token::UnescapedVariable(uv) => {
                current_children.push(Node::UnescapedVariable(uv));
            }
            Token::Comment(c) => {
                current_children.push(Node::Comment(c));
            }
            Token::Partial(p) => {
                current_children.push(Node::Partial(p));
            }
            Token::SectionOpen(name) => {
                stack.push((name, current_children, false));
                current_children = Vec::new();
            }
            Token::InvertedOpen(name) => {
                stack.push((name, current_children, true));
                current_children = Vec::new();
            }
            Token::SectionClose(name) => {
                if let Some((open_name, parent_children, inverted)) = stack.pop() {
                    if open_name == name {
                        let section_node = Node::Section {
                            name,
                            children: current_children,
                            inverted,
                        };
                        current_children = parent_children;
                        current_children.push(section_node);
                    } else {
                        return Err(format!(
                            "Mismatched closing tag: expected {}, found {}",
                            open_name, name
                        ));
                    }
                } else {
                    return Err(format!("Unopened section closing tag: {}", name));
                }
            }
        }
    }

    if let Some((unclosed_name, _, _)) = stack.pop() {
        return Err(format!("Unclosed section: {}", unclosed_name));
    }

    Ok(current_children)
}
