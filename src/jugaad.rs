pub struct JugaadResult {
    pub fixed_template: String,
    pub fixes: Vec<String>,
}

pub fn jugaad_fix(template: &str) -> JugaadResult {
    let mut result = template.to_string();
    let mut fixes = vec![];

    // Fix 1: Unclosed {{ tags
    let mut i = 0;
    while i < result.len() {
        if result[i..].starts_with("{{") {
            let next_open = result[i + 2..].find("{{").map(|idx| i + 2 + idx);
            let next_close = result[i + 2..].find("}}").map(|idx| i + 2 + idx);
            match (next_open, next_close) {
                (Some(open_idx), Some(close_idx)) if close_idx < open_idx => {
                    i = close_idx + 2;
                }
                (None, Some(close_idx)) => {
                    i = close_idx + 2;
                }
                _ => {
                    let insert_pos = next_open.unwrap_or(result.len());
                    result.insert_str(insert_pos, "}}");
                    fixes.push(format!("Unclosed {{ tag at position {}... theek kar diya (fixed it)", i));
                    i = insert_pos + 2;
                }
            }
        } else {
            i += 1;
        }
    }

    // Fix 4: Whitespace in tag names (E.g. {{ name }} -> {{name}})
    let mut i = 0;
    while i < result.len() {
        if result[i..].starts_with("{{") {
            if let Some(close_idx) = result[i..].find("}}") {
                let tag_content = &result[i + 2..i + close_idx];
                let trimmed = tag_content.trim();
                if trimmed.starts_with('=') && trimmed.ends_with('=') {
                    i += close_idx + 2;
                    continue;
                }
                let first_char = trimmed.chars().next();
                let needs_rewrite = match first_char {
                    Some('#') | Some('^') | Some('/') | Some('>') | Some('&') => {
                        let prefix = first_char.unwrap();
                        let name = trimmed[1..].trim();
                        let expected = format!("{}{}", prefix, name);
                        tag_content != expected
                    }
                    Some('!') => false,
                    _ => {
                        if trimmed.starts_with('{') && trimmed.ends_with('}') {
                            let name = trimmed[1..trimmed.len() - 1].trim();
                            let expected = format!("{{{}}}", name);
                            tag_content != expected
                        } else {
                            tag_content != trimmed
                        }
                    }
                };

                if needs_rewrite {
                    let rewritten = match first_char {
                        Some('#') | Some('^') | Some('/') | Some('>') | Some('&') => {
                            let prefix = first_char.unwrap();
                            let name = trimmed[1..].trim();
                            format!("{{{{{}{}}}}}", prefix, name)
                        }
                        _ => {
                            if trimmed.starts_with('{') && trimmed.ends_with('}') {
                                let name = trimmed[1..trimmed.len() - 1].trim();
                                format!("{{{{{{{}}}}}}}", name)
                            } else {
                                format!("{{{{{}}}}}", trimmed)
                            }
                        }
                    };
                    result.replace_range(i..i + close_idx + 2, &rewritten);
                    fixes.push("Whitespace in tag name... saaf kar diya (cleaned it up)".to_string());
                    i += rewritten.len();
                } else {
                    i += close_idx + 2;
                }
            } else {
                i += 2;
            }
        } else {
            i += 1;
        }
    }

    // Fix 2 & 3: Unclosed sections and Extra closing tags
    let mut open_sections: Vec<String> = vec![];
    let mut i = 0;
    while i < result.len() {
        if result[i..].starts_with("{{") {
            if let Some(close_idx) = result[i..].find("}}") {
                let tag_content = &result[i + 2..i + close_idx];
                let trimmed = tag_content.trim();
                if trimmed.starts_with('#') || trimmed.starts_with('^') {
                    let name = trimmed[1..].trim().to_string();
                    open_sections.push(name);
                    i += close_idx + 2;
                } else if trimmed.starts_with('/') {
                    let name = trimmed[1..].trim().to_string();
                    if let Some(pos) = open_sections.iter().rposition(|n| n == &name) {
                        open_sections.remove(pos);
                        i += close_idx + 2;
                    } else {
                        result.replace_range(i..i + close_idx + 2, "");
                        fixes.push(format!("Orphan closing tag '{}'... hata diya (removed it)", name));
                    }
                } else {
                    i += close_idx + 2;
                }
            } else {
                i += 2;
            }
        } else {
            i += 1;
        }
    }
    for name in open_sections.into_iter().rev() {
        result.push_str(&format!("{{{{/{}}}}}", name));
        fixes.push(format!("Unclosed section '{}'... band kar diya (closed it)", name));
    }

    JugaadResult {
        fixed_template: result,
        fixes,
    }
}

pub fn print_jugaad_header() {
    println!("⚡ Jugaad mode activated!");
}

pub fn print_jugaad_fixes(fixes: &[String]) {
    for fix in fixes {
        println!("🔧 {}", fix);
    }
    println!("😎 Kaam ho gaya. (Job done.)\n");
}
