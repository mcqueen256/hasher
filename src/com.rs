pub fn count_nounce(hash: &String) -> usize {
    let mut length = 0;
    for c in hash.chars() {
        if c == '0' {
            length += 1;
        } else {
            break;
        }
    }
    length
}