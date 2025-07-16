pub fn split_answer(answer: &str) -> Vec<String> {
    let parts: Vec<_> = answer
        .split("\n\n")
        .map(str::trim).filter(|s| !s.is_empty())
        .collect();

    if parts.len() > 1 { 
        parts.into_iter().map(|s| s.to_string()).collect() 
    } else {
        vec![answer.to_string()]
    }
}