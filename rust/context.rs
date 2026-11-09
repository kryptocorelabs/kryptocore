use std::fmt;

const PREFIX: &[u8] = b"KryptoCore/AAD\0\x01";
const MAX_METADATA: usize = 64 * 1024;

/// Canonically encoded public AAD for one application, format version and purpose.
pub struct Context(Vec<u8>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ContextError {
    InvalidLabel,
    InvalidVersion,
    MetadataTooLarge,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLabel => "Context labels must be 1 to 255 lowercase ASCII letters, digits, dots, hyphens or underscores.",
            Self::InvalidVersion => "Context version must be greater than zero.",
            Self::MetadataTooLarge => "Context metadata exceeds 64 KiB.",
        })
    }
}

impl std::error::Error for ContextError {}

impl Context {
    /// Use an application-owned namespace such as `org.example.vault`.
    /// Choose the version and purpose from the expected protocol, and supply
    /// the same public metadata bytes when opening. Metadata is limited to 64 KiB.
    pub fn new(
        application: &str,
        version: u32,
        purpose: &str,
        metadata: &[u8],
    ) -> Result<Self, ContextError> {
        if !valid_label(application) || !valid_label(purpose) {
            return Err(ContextError::InvalidLabel);
        }
        if version == 0 {
            return Err(ContextError::InvalidVersion);
        }
        if metadata.len() > MAX_METADATA {
            return Err(ContextError::MetadataTooLarge);
        }
        let mut aad = Vec::with_capacity(
            PREFIX.len() + application.len() + purpose.len() + 6 + metadata.len(),
        );
        aad.extend_from_slice(PREFIX);
        aad.push(application.len() as u8);
        aad.extend_from_slice(application.as_bytes());
        aad.extend_from_slice(&version.to_be_bytes());
        aad.push(purpose.len() as u8);
        aad.extend_from_slice(purpose.as_bytes());
        aad.extend_from_slice(metadata);
        Ok(Self(aad))
    }

    /// The public bytes to pass to ordinary `encrypt` and `decrypt`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

fn valid_label(label: &str) -> bool {
    (1..=255).contains(&label.len())
        && label.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_has_explicit_boundaries_and_version() {
        let context = Context::new("ab", 1, "c", b"\0d").unwrap();
        assert_eq!(
            context.as_bytes(),
            b"KryptoCore/AAD\0\x01\x02ab\0\0\0\x01\x01c\0d"
        );
        assert_ne!(
            context.as_bytes(),
            Context::new("a", 1, "bc", b"\0d").unwrap().as_bytes()
        );
        assert_ne!(
            context.as_bytes(),
            Context::new("ab", 2, "c", b"\0d").unwrap().as_bytes()
        );
    }

    #[test]
    fn rejects_invalid_labels_versions_and_overlarge_metadata() {
        for label in ["", "UPPER", "a/b", "é", &"a".repeat(256)] {
            assert!(matches!(
                Context::new(label, 1, "slot", b""),
                Err(ContextError::InvalidLabel)
            ));
        }
        assert!(matches!(
            Context::new("example", 0, "slot", b""),
            Err(ContextError::InvalidVersion)
        ));
        assert!(Context::new("example", 1, "slot", &vec![0; MAX_METADATA]).is_ok());
        assert!(matches!(
            Context::new("example", 1, "slot", &vec![0; MAX_METADATA + 1]),
            Err(ContextError::MetadataTooLarge)
        ));
    }
}
