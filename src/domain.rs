use unicode_segmentation::UnicodeSegmentation;
use validator::validate_email;

#[derive(Debug)]
pub struct Subscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|c| forbidden_chars.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            Err(format!("Invalid subscriber name: {s}"))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("Invalid email address: {s}"))
        }
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{SubscriberEmail, SubscriberName};
    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck::Gen;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert!(SubscriberName::parse(name).is_ok());
    }

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert!(SubscriberName::parse(name).is_ok());
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_invalid() {
        let name = "a".repeat(257);
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn name_empty_string_is_rejected() {
        let name = "".to_string();
        assert!(SubscriberName::parse(name).is_err());
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert!(SubscriberName::parse(name).is_err());
        }
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_correctly(valid_email: ValidEmailFixture) {
        assert!(SubscriberEmail::parse(valid_email.0).is_ok());
    }

    #[test]
    fn email_empty_string_is_rejected() {
        let email = "".to_string();
        assert!(SubscriberEmail::parse(email).is_err());
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert!(SubscriberEmail::parse(email).is_err());
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert!(SubscriberEmail::parse(email).is_err());
    }
}
