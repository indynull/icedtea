//! Fuzzy ranking for the command palette.

/// Score `candidate` against `query`. Higher is better. `None` is no match.
///
/// ```
/// use icedtea::fuzzy::score;
/// assert!(score("save", "Save file").is_some());
/// assert!(score("zz", "Save").is_none());
/// ```
pub fn score(query: &str, candidate: &str) -> Option<u32> {
    let q = query.trim();
    if q.is_empty() {
        return Some(0);
    }
    let query: Vec<char> = q.to_ascii_lowercase().chars().collect();
    let cand_raw = candidate.to_ascii_lowercase();
    let cand: Vec<char> = cand_raw.chars().collect();
    if query.len() > cand.len() {
        return None;
    }
    let mut qi = 0;
    let mut points: u32 = 0;
    let mut prev_match = false;
    for (i, ch) in cand.iter().enumerate() {
        if qi < query.len() && *ch == query[qi] {
            points += 8;
            if i == 0 || cand[i - 1] == ' ' || cand[i - 1] == '.' || cand[i - 1] == '/' {
                points += 12;
            }
            if prev_match {
                points += 6;
            }
            qi += 1;
            prev_match = true;
        } else {
            prev_match = false;
        }
    }
    if qi == query.len() {
        Some(points)
    } else {
        None
    }
}

/// Filter and rank strings; stable order for equal scores.
pub fn rank<'a>(query: &str, items: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut scored: Vec<(u32, usize, &'a str)> = items
        .into_iter()
        .enumerate()
        .filter_map(|(i, item)| score(query, item).map(|s| (s, i, item)))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, _, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_prefix_above_scattered() {
        assert_eq!(score("", "Anything"), Some(0));
        assert!(score("save", "Save").is_some());
        assert!(score("zzzz", "Save").is_none());
        assert!(score("toolongquery", "ab").is_none());
        let ranked = rank("sav", ["Close view", "Save file", "Server"]);
        assert_eq!(ranked[0], "Save file");
        let tied = rank("a", ["ab", "ac"]);
        assert_eq!(tied, vec!["ab", "ac"]);
        assert!(rank("q", Vec::<&str>::new()).is_empty());
    }
}
