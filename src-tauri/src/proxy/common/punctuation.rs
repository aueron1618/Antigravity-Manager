use std::collections::HashSet;

const DEFAULT_EXCLUDE_TAGS: [&str; 4] = ["code", "pre", "script", "style"];

fn parse_exclude_tags(exclude_tags_raw: &str) -> Vec<String> {
    let tags: Vec<String> = exclude_tags_raw
        .split(',')
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();

    if tags.is_empty() {
        DEFAULT_EXCLUDE_TAGS
            .iter()
            .map(|tag| (*tag).to_string())
            .collect()
    } else {
        tags
    }
}

fn find_exclude_ranges_by_tags(text: &str, tags: &[String]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();

    for tag in tags {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);
        let mut search_pos = 0usize;

        while search_pos < text.len() {
            let Some(start_rel) = text[search_pos..].find(&open_tag) else {
                break;
            };
            let start_idx = search_pos + start_rel;
            let close_search_start = start_idx + open_tag.len();

            let Some(close_rel) = text[close_search_start..].find(&close_tag) else {
                break;
            };
            let close_idx = close_search_start + close_rel;
            let end_idx = close_idx + close_tag.len();

            ranges.push((start_idx, end_idx));
            search_pos = end_idx;
        }
    }

    ranges.sort_by_key(|(start, _)| *start);

    let mut merged = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut() {
            if range.0 <= last.1 {
                if range.1 > last.1 {
                    last.1 = range.1;
                }
                continue;
            }
        }
        merged.push(range);
    }

    merged
}

fn is_chinese(ch: char) -> bool {
    let code = ch as u32;
    (0x4E00..=0x9FFF).contains(&code)
        || (0x3400..=0x4DBF).contains(&code)
        || (0x20000..=0x2A6DF).contains(&code)
        || (0x2A700..=0x2B73F).contains(&code)
        || (0x2B740..=0x2B81F).contains(&code)
        || (0x2B820..=0x2CEAF).contains(&code)
        || (0xF900..=0xFAFF).contains(&code)
        || (0x2F800..=0x2FA1F).contains(&code)
}

fn is_cjk_punctuation_or_fullwidth(ch: char) -> bool {
    let code = ch as u32;
    (0x3000..=0x303F).contains(&code) || (0xFF00..=0xFFEF).contains(&code)
}

fn is_apostrophe(runes: &[char], pos: usize) -> bool {
    if pos == 0 || pos + 1 >= runes.len() {
        return false;
    }

    runes[pos - 1].is_ascii_alphabetic() && runes[pos + 1].is_ascii_alphabetic()
}

fn is_in_url(runes: &[char], pos: usize) -> bool {
    let start = pos.saturating_sub(20);

    for i in start..pos {
        if i + 6 >= runes.len() {
            break;
        }

        if runes[i] == 'h' && runes[i + 1] == 't' && runes[i + 2] == 't' && runes[i + 3] == 'p' {
            let is_http = runes[i + 4] == ':' && runes[i + 5] == '/' && runes[i + 6] == '/';
            let is_https = i + 7 < runes.len()
                && runes[i + 4] == 's'
                && runes[i + 5] == ':'
                && runes[i + 6] == '/'
                && runes[i + 7] == '/';

            if is_http || is_https {
                for j in i..=pos {
                    if j >= runes.len() {
                        break;
                    }
                    if runes[j].is_whitespace() {
                        return false;
                    }
                }
                return true;
            }
        }
    }

    false
}

fn has_chinese_context(runes: &[char], pos: usize) -> bool {
    let start = pos.saturating_sub(5);
    let end = (pos + 6).min(runes.len());

    let mut chinese_count = 0usize;
    let mut total_chars = 0usize;

    for i in start..end {
        if i == pos {
            continue;
        }

        let ch = runes[i];
        if ch != '\n' && ch != '\r' && !ch.is_whitespace() {
            total_chars += 1;
            if is_chinese(ch) || is_cjk_punctuation_or_fullwidth(ch) {
                chinese_count += 1;
            }
        }
    }

    total_chars > 0 && (chinese_count as f64 / total_chars as f64) > 0.2
}

fn collect_quote_positions(runes: &[char], quote_char: char, skip_apostrophe: bool) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut in_code_block = false;
    let mut i = 0usize;

    while i < runes.len() {
        let ch = runes[i];

        if ch == '`' && i + 2 < runes.len() && runes[i + 1] == '`' && runes[i + 2] == '`' {
            in_code_block = !in_code_block;
            i += 3;
            continue;
        }

        if in_code_block || is_in_url(runes, i) {
            i += 1;
            continue;
        }

        if ch == quote_char {
            if !(skip_apostrophe && is_apostrophe(runes, i)) {
                positions.push(i);
            }
        }

        i += 1;
    }

    positions
}

fn decide_convert_quote_positions(runes: &[char], positions: &[usize]) -> HashSet<usize> {
    let mut to_convert = HashSet::new();
    let mut i = 0usize;

    while i < positions.len() {
        let open_pos = positions[i];

        if let Some(close_pos) = positions.get(i + 1).copied() {
            let mut convert =
                has_chinese_context(runes, open_pos) || has_chinese_context(runes, close_pos);

            if !convert {
                for ch in runes.iter().take(close_pos).skip(open_pos + 1) {
                    if is_chinese(*ch) || is_cjk_punctuation_or_fullwidth(*ch) {
                        convert = true;
                        break;
                    }
                }
            }

            if convert {
                to_convert.insert(open_pos);
                to_convert.insert(close_pos);
            }
        } else if has_chinese_context(runes, open_pos) {
            to_convert.insert(open_pos);
        }

        i += 2;
    }

    to_convert
}

fn normalize_punctuation_internal(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }

    let runes: Vec<char> = text.chars().collect();
    let double_quote_positions = collect_quote_positions(&runes, '"', false);
    let single_quote_positions = collect_quote_positions(&runes, '\'', true);

    let double_quotes_to_convert = decide_convert_quote_positions(&runes, &double_quote_positions);
    let single_quotes_to_convert = decide_convert_quote_positions(&runes, &single_quote_positions);

    let mut out = String::with_capacity(text.len());
    let mut in_code_block = false;
    let mut double_is_open = true;
    let mut single_is_open = true;
    let mut i = 0usize;

    while i < runes.len() {
        let ch = runes[i];

        if ch == '`' && i + 2 < runes.len() && runes[i + 1] == '`' && runes[i + 2] == '`' {
            in_code_block = !in_code_block;
            out.push('`');
            out.push('`');
            out.push('`');
            i += 3;
            continue;
        }

        if in_code_block || is_in_url(&runes, i) {
            out.push(ch);
            i += 1;
            continue;
        }

        if ch == '"' {
            if double_quotes_to_convert.contains(&i) {
                out.push(if double_is_open {
                    '\u{201C}'
                } else {
                    '\u{201D}'
                });
                double_is_open = !double_is_open;
            } else {
                out.push(ch);
            }
            i += 1;
            continue;
        }

        if ch == '\'' {
            if single_quotes_to_convert.contains(&i) {
                out.push(if single_is_open {
                    '\u{2018}'
                } else {
                    '\u{2019}'
                });
                single_is_open = !single_is_open;
            } else {
                out.push(ch);
            }
            i += 1;
            continue;
        }

        if ch == ',' {
            if has_chinese_context(&runes, i) {
                out.push('\u{FF0C}');
            } else {
                out.push(ch);
            }
            i += 1;
            continue;
        }

        out.push(ch);
        i += 1;
    }

    out
}

pub fn normalize_punctuation_with_tags(text: &str, exclude_tags_raw: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }

    let tags = parse_exclude_tags(exclude_tags_raw);
    if tags.is_empty() {
        return normalize_punctuation_internal(text);
    }

    let ranges = find_exclude_ranges_by_tags(text, &tags);
    if ranges.is_empty() {
        return normalize_punctuation_internal(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut last_end = 0usize;

    for (start, end) in ranges {
        if start > last_end {
            result.push_str(&normalize_punctuation_internal(&text[last_end..start]));
        }
        result.push_str(&text[start..end]);
        last_end = end;
    }

    if last_end < text.len() {
        result.push_str(&normalize_punctuation_internal(&text[last_end..]));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::normalize_punctuation_with_tags;

    #[test]
    fn converts_comma_in_chinese_context() {
        let text = "你好,世界";
        let normalized = normalize_punctuation_with_tags(text, "code,pre");
        assert_eq!(normalized, "你好，世界");
    }

    #[test]
    fn keeps_english_comma_unchanged() {
        let text = "hello, world";
        let normalized = normalize_punctuation_with_tags(text, "code,pre");
        assert_eq!(normalized, text);
    }

    #[test]
    fn skips_excluded_html_blocks() {
        let text = "你好,<code>a,b</code>,世界";
        let normalized = normalize_punctuation_with_tags(text, "code");
        assert_eq!(normalized, "你好，<code>a,b</code>，世界");
    }
}
