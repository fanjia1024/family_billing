//! Value object for monetary amount. Validates non-NaN, finite, and optional sign.

use crate::domain::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Money(f64);

impl Money {
    /// Create Money if value is valid (finite, not NaN). Does not require positive.
    pub fn try_new(value: f64) -> Result<Self, DomainError> {
        if value.is_nan() || value.is_infinite() {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Money(value))
    }

    pub fn as_f64(&self) -> f64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }
}
