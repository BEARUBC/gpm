//! Common utils

pub mod string {
    /// Converts snake_case string to Title Case
    /// Eg. grasp_primary_module -> Grasp Primary Module
    pub fn snake_to_title_case(s: &str) -> String {
        let mut capitalize_next = true;
        s.chars()
            .fold(String::with_capacity(s.len()), |mut acc, c| {
                match c {
                    '_' => {
                        acc.push(' ');
                        capitalize_next = true;
                    },
                    // spaces or non-ASCII characters are added as-is.
                    c if c == ' ' || !c.is_ascii() => {
                        acc.push(c);
                    },
                    c if capitalize_next => {
                        acc.push(c.to_ascii_uppercase());
                        capitalize_next = false;
                    },
                    c => {
                        acc.push(c.to_ascii_lowercase());
                    },
                }
                acc
            })
    }
}
