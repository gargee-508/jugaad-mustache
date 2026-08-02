#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Variable(String),
    UnescapedVariable(String),
    SectionOpen(String),
    SectionClose(String),
    InvertedOpen(String),
    Comment(String),
    Partial(String),
}

pub struct Scanner {
    pub tokens: Vec<Token>,
}

impl Scanner {
    pub fn scan(template: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut open_del = "{{".to_string();
        let mut close_del = "}}".to_string();
        let mut remaining = template;

        while !remaining.is_empty() {
            if let Some(open_idx) = remaining.find(&open_del) {
                // Add text before open delimiter
                if open_idx > 0 {
                    tokens.push(Token::Text(remaining[..open_idx].to_string()));
                }
                
                // Advance past the open delimiter
                remaining = &remaining[open_idx + open_del.len()..];

                // Check for triple mustache {{{
                let is_triple = if open_del == "{{" && remaining.starts_with('{') {
                    remaining = &remaining[1..];
                    true
                } else {
                    false
                };

                let search_close = if is_triple {
                    format!("}}{}", close_del)
                } else {
                    close_del.clone()
                };

                if let Some(close_idx) = remaining.find(&search_close) {
                    let tag_content = &remaining[..close_idx];
                    remaining = &remaining[close_idx + search_close.len()..];

                    let trimmed = tag_content.trim();

                    // Check for delimiter change
                    if trimmed.starts_with('=') && trimmed.ends_with('=') {
                        let inner = &trimmed[1..trimmed.len() - 1];
                        let parts: Vec<&str> = inner.split_whitespace().collect();
                        if parts.len() == 2 {
                            open_del = parts[0].to_string();
                            close_del = parts[1].to_string();
                        }
                    } else if let Some(first_char) = trimmed.chars().next() {
                        match first_char {
                            '!' => tokens.push(Token::Comment(trimmed[1..].trim().to_string())),
                            '#' => tokens.push(Token::SectionOpen(trimmed[1..].trim().to_string())),
                            '^' => tokens.push(Token::InvertedOpen(trimmed[1..].trim().to_string())),
                            '/' => tokens.push(Token::SectionClose(trimmed[1..].trim().to_string())),
                            '>' => tokens.push(Token::Partial(trimmed[1..].trim().to_string())),
                            '&' => tokens.push(Token::UnescapedVariable(trimmed[1..].trim().to_string())),
                            _ => {
                                if is_triple {
                                    tokens.push(Token::UnescapedVariable(trimmed.to_string()));
                                } else {
                                    tokens.push(Token::Variable(trimmed.to_string()));
                                }
                            }
                        }
                    } else {
                        // Empty tag content
                        if is_triple {
                            tokens.push(Token::UnescapedVariable(String::new()));
                        } else {
                            tokens.push(Token::Variable(String::new()));
                        }
                    }
                } else {
                    // Mismatched open delimiter (no close found).
                    let unclosed_prefix = if is_triple {
                        format!("{}{}", open_del, "{")
                    } else {
                        open_del.clone()
                    };
                    tokens.push(Token::Text(format!("{}{}", unclosed_prefix, remaining)));
                    break;
                }
            } else {
                tokens.push(Token::Text(remaining.to_string()));
                break;
            }
        }

        tokens
    }
}
