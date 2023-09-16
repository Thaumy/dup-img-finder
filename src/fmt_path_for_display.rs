pub fn fmt_path_for_display(path: &str, prefix_len: usize) -> String {
    let diff = term_size::dimensions()
        .unwrap()
        .0 as i32 -
        path.len() as i32 -
        prefix_len as i32;

    if diff >= 0 {
        path.to_owned()
    } else {
        path.chars()
            .skip(prefix_len - diff as usize)
            .collect()
    }
}
