// digit_format.rs

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DecimalDigitSet {
    #[default]
    Latin,
    ArabicIndic,
    EasternArabicIndic,
}

impl DecimalDigitSet {
    pub fn for_locale(locale: &str) -> Self {
        let subtags = locale
            .split(['-', '_'])
            .filter(|subtag| !subtag.is_empty())
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>();

        if let Some(numbering_system) = unicode_numbering_system(&subtags) {
            return match numbering_system {
                "arab" => Self::ArabicIndic,
                "arabext" => Self::EasternArabicIndic,
                "latn" => Self::Latin,
                _ => locale_default(&subtags),
            };
        }

        locale_default(&subtags)
    }

    pub fn format(self, input: &str) -> String {
        input
            .chars()
            .map(|character| {
                decimal_value(character)
                    .map(|digit| digit_character(self, digit))
                    .unwrap_or(character)
            })
            .collect()
    }
}

fn unicode_numbering_system(subtags: &[String]) -> Option<&str> {
    let unicode_extension = subtags.iter().position(|subtag| subtag == "u")?;
    subtags[unicode_extension + 1..]
        .windows(2)
        .find(|pair| pair[0] == "nu")
        .map(|pair| pair[1].as_str())
}

fn locale_default(subtags: &[String]) -> DecimalDigitSet {
    match subtags.first().map(String::as_str) {
        Some("ar") => DecimalDigitSet::ArabicIndic,
        Some("fa" | "ps" | "ur") => DecimalDigitSet::EasternArabicIndic,
        _ => DecimalDigitSet::Latin,
    }
}

fn decimal_value(character: char) -> Option<u32> {
    match character {
        '0'..='9' => Some(character as u32 - '0' as u32),
        '\u{0660}'..='\u{0669}' => Some(character as u32 - '\u{0660}' as u32),
        '\u{06f0}'..='\u{06f9}' => Some(character as u32 - '\u{06f0}' as u32),
        _ => None,
    }
}

fn digit_character(digit_set: DecimalDigitSet, digit: u32) -> char {
    let zero = match digit_set {
        DecimalDigitSet::Latin => '0',
        DecimalDigitSet::ArabicIndic => '\u{0660}',
        DecimalDigitSet::EasternArabicIndic => '\u{06f0}',
    };
    char::from_u32(zero as u32 + digit).unwrap()
}

#[cfg(test)]
mod tests {
    use super::DecimalDigitSet;

    #[test]
    fn selects_digit_sets_from_language_subtags() {
        assert_eq!(
            DecimalDigitSet::for_locale("ar-EG"),
            DecimalDigitSet::ArabicIndic
        );
        assert_eq!(
            DecimalDigitSet::for_locale("fa_IR"),
            DecimalDigitSet::EasternArabicIndic
        );
        assert_eq!(
            DecimalDigitSet::for_locale("ur-PK"),
            DecimalDigitSet::EasternArabicIndic
        );
        assert_eq!(DecimalDigitSet::for_locale("en-US"), DecimalDigitSet::Latin);
    }

    #[test]
    fn unicode_numbering_system_overrides_language_default() {
        assert_eq!(
            DecimalDigitSet::for_locale("fa-IR-u-nu-latn"),
            DecimalDigitSet::Latin
        );
        assert_eq!(
            DecimalDigitSet::for_locale("en-u-nu-arab"),
            DecimalDigitSet::ArabicIndic
        );
        assert_eq!(
            DecimalDigitSet::for_locale("ar-u-ca-persian-nu-arabext"),
            DecimalDigitSet::EasternArabicIndic
        );
    }

    #[test]
    fn formats_only_decimal_digits() {
        let formatted = DecimalDigitSet::EasternArabicIndic.format("نسخه 2.10 (v3)");

        assert_eq!(formatted, "نسخه ۲.۱۰ (v۳)");
    }

    #[test]
    fn normalizes_supported_digit_sets() {
        assert_eq!(DecimalDigitSet::Latin.format("١٢۳۴"), "1234");
        assert_eq!(DecimalDigitSet::ArabicIndic.format("12۳۴"), "١٢٣٤");
        assert_eq!(DecimalDigitSet::EasternArabicIndic.format("12٣٤"), "۱۲۳۴");
    }

    #[test]
    fn unknown_locale_and_numbering_system_fall_back_to_language() {
        assert_eq!(
            DecimalDigitSet::for_locale("fa-u-nu-unknown"),
            DecimalDigitSet::EasternArabicIndic
        );
        assert_eq!(DecimalDigitSet::for_locale("und"), DecimalDigitSet::Latin);
        assert_eq!(DecimalDigitSet::for_locale(""), DecimalDigitSet::Latin);
    }
}
