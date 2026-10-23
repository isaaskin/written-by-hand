pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    #[test]
    fn counts() {
        assert_eq!(super::count_words("a b  c"), 3);
    }
}
