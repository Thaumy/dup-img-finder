use terminal_size::Width;

pub fn fmt_path_for_display(path: &str, prefix_len: usize) -> String {
    let (Width(width), _) = terminal_size::terminal_size().expect("Can not get terminal size");
    let diff = width as i32 - path.len() as i32 - prefix_len as i32;

    if diff >= 0 {
        path.to_owned()
    } else {
        path.chars().skip(prefix_len - diff as usize).collect()
    }
}
