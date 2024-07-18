
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YoctoNearError {
    InvalidTokensAmount(crate::utils::DecimalNumberParsingError),
    InvalidTokenUnit(String),
}

impl std::fmt::Display for YoctoNearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          YoctoNearError::InvalidTokensAmount(err) => write!(f, "invalid tokens amount: {}", err),
          YoctoNearError::InvalidTokenUnit(unit) => write!(f, "invalid token unit: {}", unit),
        }
    }
}

impl std::error::Error for YoctoNearError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
          YoctoNearError::InvalidTokensAmount(err) => Some(err),
          YoctoNearError::InvalidTokenUnit(_) => None,
        }
    }
}
