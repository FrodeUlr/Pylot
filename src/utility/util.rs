use crate::cmd::utils;

pub fn get_index(size: usize) -> usize {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|&i| (1..=size).contains(&i))
        .unwrap_or_else(|| utils::exit_with_error("Error, please provide a valid index"))
}
