pub fn redact_user_paths(text: &str) -> String {
    let mut result = text.to_owned();
    if let Ok(profile) = std::env::var("USERPROFILE") {
        for variant in [
            profile.clone(),
            profile.replace('\\', "/"),
            profile.replace('\\', "\\\\"),
        ] {
            result = replace_ignore_case(&result, &variant, "%USERPROFILE%");
        }
    }
    if let Ok(user) = std::env::var("USERNAME")
        && user.chars().count() >= 3
    {
        result = replace_ignore_case(&result, &user, "<user>");
    }
    result
}

fn replace_ignore_case(text: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return text.to_owned();
    }
    let lower_text = text.to_lowercase();
    let lower_needle = needle.to_lowercase();
    if lower_text.len() != text.len() || lower_needle.len() != needle.len() {
        return text.replace(needle, replacement);
    }
    let mut result = String::with_capacity(text.len());
    let mut last = 0;
    for (index, _) in lower_text.match_indices(&lower_needle) {
        if index < last {
            continue;
        }
        result.push_str(&text[last..index]);
        result.push_str(replacement);
        last = index + needle.len();
    }
    result.push_str(&text[last..]);
    result
}

#[cfg(test)]
mod tests {
    use super::replace_ignore_case;

    #[test]
    fn replaces_regardless_of_case() {
        let text = r"C:\Users\Adria\AppData and c:\users\adria\x";
        assert_eq!(
            replace_ignore_case(text, r"C:\Users\adria", "%USERPROFILE%"),
            r"%USERPROFILE%\AppData and %USERPROFILE%\x"
        );
    }
}
