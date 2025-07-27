pub mod string {
    pub fn snake_to_title_case(s: &str) -> String {
        let mut capitalize_next = true;
        s.chars()
            .fold(String::with_capacity(s.len()), |mut acc, c| {
                if c == '_' {
                    acc.push(' ');
                    capitalize_next = true;
                } else if c == ' ' || c == '▼' || c == '▶' {
                    acc.push(c);
                } else if capitalize_next {
                    acc.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    acc.push(c.to_ascii_lowercase());
                }
                acc
            })
    }
}
