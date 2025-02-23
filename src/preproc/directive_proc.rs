use crate::*;
pub fn process_start(
    toks: &mut Vec<(String, TokenKind, std::ops::Range<usize>)>,
    error_count: &mut i32,
) {
    use crate::TokenKind::*;
    let mut toks_iter = toks.clone().into_iter().peekable();
    let mut start_addr = 100;
    let mut seen_start = false;
    while let Some((fname, tok, span)) = toks_iter.next() {
        if let Directive(data) = tok {
            if data.as_str() == "start" {
                if seen_start {
                    handle_include_error(
                        &fname,
                        &span,
                        error_count,
                        ".start directive can only be declared once",
                        None,
                    );
                    break;
                }
                if let Some((_, TokenKind::IntLit(val), _)) = toks_iter.peek() {
                    start_addr = *val;
                    seen_start = true;
                } else {
                    handle_include_error(
                        &fname,
                        &span,
                        error_count,
                        ".start directive must be succeeded by integer literal",
                        None,
                    );

                    break;
                }
            }
        }
    }
    process_directives(toks, error_count, start_addr);
    let mut new_toks = Vec::new();
    {
        for (fname, tok, span) in &mut *toks {
            if let TokenKind::Label(_) = tok {
            } else {
                new_toks.push((fname.to_string(), tok.clone(), span.clone()));
            }
        }
    }
    *toks = new_toks;
    print_errc!(*error_count);
}

fn process_directives(
    toks: &mut [(String, TokenKind, std::ops::Range<usize>)],
    error_count: &mut i32,
    start_addr: i64,
) {
    use crate::TokenKind::*;
    let mut toks_iter = toks.iter().cloned().peekable();
    let mut l_map = LABEL_MAP.lock().unwrap();
    let mut loc_counter = start_addr;
    while let Some((fname, tok, span)) = toks_iter.next() {
        match tok {
            Label(name) => {
                l_map.insert(
                    name,
                    (fname.to_string(), span.clone(), loc_counter as usize),
                );
            }
            Directive(data) => match data.trim() {
                "start" => {
                    if let Some((_, TokenKind::IntLit(_), _)) = toks_iter.peek() {
                        toks_iter.next();
                    }
                }
                "pad" => {
                    if let Some((_, TokenKind::IntLit(v), _)) = toks_iter.peek() {
                        loc_counter += v;
                        toks_iter.next();
                    } else {
                        handle_include_error(
                            &fname,
                            &span,
                            error_count,
                            ".pad directive must be succeeded by literal",
                            None,
                        );
                        break;
                    }
                }
                "word" => {
                    if toks_iter
                        .peek()
                        .is_some_and(|v| matches!(v.1, TokenKind::Ident(_) | TokenKind::IntLit(_)))
                    {
                        loc_counter += 1;
                        toks_iter.next();
                    } else {
                        handle_include_error(
                            &fname,
                            &span,
                            error_count,
                            ".word directive must be succeeded by literal",
                            None,
                        );
                        break;
                    }
                }
                "asciiz" => {
                    if let Some((_, TokenKind::StringLit(val), _)) = toks_iter.peek() {
                        loc_counter += val.len() as i64;
                        toks_iter.next();
                    } else {
                        handle_include_error(
                            &fname,
                            &span,
                            error_count,
                            ".asciiz directive must be succeeded by string",
                            None,
                        );
                        break;
                    }
                }
                _ => {
                    handle_include_error(
                        &fname,
                        &span,
                        error_count,
                        &format!("unrecognized directive {data}"),
                        None,
                    );
                    break;
                }
            },
            Instruction(_) => loc_counter += 1,
            Newline => (),
            _ => {
                handle_include_error(
                    &fname,
                    &span,
                    error_count,
                    &format!("unrecognized {tok}"),
                    None,
                );
                break;
            }
        }
    }
    print_errc!(*error_count);
}
