use terminal_size::Width;

pub fn fmt_path_for_display(path: &str, prefix_len: usize) -> String {
    let width = terminal_size::terminal_size()
        .map(|(Width(w), _)| w)
        .unwrap_or(128);
    let diff = width as i32 - path.len() as i32 - prefix_len as i32;

    if diff >= 0 {
        path.to_owned()
    } else {
        let skip = prefix_len.saturating_sub(diff as usize);
        path.chars().skip(skip).collect()
    }
}
