use std::time::Duration;

use crate::error::PopularityError;

/// A function that computes a weight based on the age of an event.
///
/// Decay functions are used by trending scorers to give more recent events
/// higher weights than older ones.
pub trait DecayFunction: Send + Sync {
	/// Computes the weight for an event with the given age.
	///
	/// Returns a value in `[0.0, 1.0]` where 1.0 means no decay.
	fn weight(&self, age: Duration) -> f64;
}

/// Exponential decay: weight = `2^(-age / half_life)`.
///
/// Events lose half their weight after each half-life period.
pub struct ExponentialDecay {
	half_life: Duration,
}

impl ExponentialDecay {
	/// Creates a new `ExponentialDecay` with the given half-life.
	///
	/// # Errors
	///
	/// Returns [`PopularityError::InvalidDecayParameter`] if `half_life` is zero.
	pub fn new(half_life: Duration) -> Result<Self, PopularityError> {
		if half_life.is_zero() {
			return Err(PopularityError::InvalidDecayParameter(
				"half_life must be greater than zero".to_string(),
			));
		}
		Ok(Self { half_life })
	}
}

impl DecayFunction for ExponentialDecay {
	fn weight(&self, age: Duration) -> f64 {
		let ratio = age.as_secs_f64() / self.half_life.as_secs_f64();
		f64::powf(2.0, -ratio)
	}
}

/// Linear decay: weight = `max(0, 1 - age / max_age)`.
///
/// Events lose weight linearly, reaching zero at `max_age`.
pub struct LinearDecay {
	max_age: Duration,
}

impl LinearDecay {
	/// Creates a new `LinearDecay` with the given maximum age.
	///
	/// # Errors
	///
	/// Returns [`PopularityError::InvalidDecayParameter`] if `max_age` is zero.
	pub fn new(max_age: Duration) -> Result<Self, PopularityError> {
		if max_age.is_zero() {
			return Err(PopularityError::InvalidDecayParameter(
				"max_age must be greater than zero".to_string(),
			));
		}
		Ok(Self { max_age })
	}
}

impl DecayFunction for LinearDecay {
	fn weight(&self, age: Duration) -> f64 {
		let ratio = age.as_secs_f64() / self.max_age.as_secs_f64();
		f64::max(0.0, 1.0 - ratio)
	}
}

/// No decay: always returns weight 1.0 regardless of age.
pub struct NoDecay;

impl DecayFunction for NoDecay {
	fn weight(&self, _age: Duration) -> f64 {
		1.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use rstest::rstest;

	#[rstest]
	fn exponential_decay_zero_age() {
		// Arrange
		let decay = ExponentialDecay::new(Duration::from_secs(3600)).unwrap();

		// Act
		let w = decay.weight(Duration::ZERO);

		// Assert
		assert_eq!(w, 1.0);
	}

	#[rstest]
	fn exponential_decay_one_half_life() {
		// Arrange
		let decay = ExponentialDecay::new(Duration::from_secs(3600)).unwrap();

		// Act
		let w = decay.weight(Duration::from_secs(3600));

		// Assert
		assert!((w - 0.5).abs() < f64::EPSILON);
	}

	#[rstest]
	fn exponential_decay_two_half_lives() {
		// Arrange
		let decay = ExponentialDecay::new(Duration::from_secs(3600)).unwrap();

		// Act
		let w = decay.weight(Duration::from_secs(7200));

		// Assert
		assert!((w - 0.25).abs() < f64::EPSILON);
	}

	#[rstest]
	fn exponential_decay_invalid_zero_half_life() {
		// Act
		let result = ExponentialDecay::new(Duration::ZERO);

		// Assert
		assert!(matches!(
			result,
			Err(PopularityError::InvalidDecayParameter(_))
		));
	}

	#[rstest]
	fn linear_decay_zero_age() {
		// Arrange
		let decay = LinearDecay::new(Duration::from_secs(3600)).unwrap();

		// Act
		let w = decay.weight(Duration::ZERO);

		// Assert
		assert_eq!(w, 1.0);
	}

	#[rstest]
	fn linear_decay_half_max_age() {
		// Arrange
		let decay = LinearDecay::new(Duration::from_secs(3600)).unwrap();

		// Act
		let w = decay.weight(Duration::from_secs(1800));

		// Assert
		assert!((w - 0.5).abs() < f64::EPSILON);
	}

	#[rstest]
	fn linear_decay_at_max_age() {
		// Arrange
		let decay = LinearDecay::new(Duration::from_secs(3600)).unwrap();

		// Act
		let w = decay.weight(Duration::from_secs(3600));

		// Assert
		assert_eq!(w, 0.0);
	}

	#[rstest]
	fn linear_decay_beyond_max_age() {
		// Arrange
		let decay = LinearDecay::new(Duration::from_secs(3600)).unwrap();

		// Act
		let w = decay.weight(Duration::from_secs(7200));

		// Assert
		assert_eq!(w, 0.0);
	}

	#[rstest]
	fn linear_decay_invalid_zero_max_age() {
		// Act
		let result = LinearDecay::new(Duration::ZERO);

		// Assert
		assert!(matches!(
			result,
			Err(PopularityError::InvalidDecayParameter(_))
		));
	}

	#[rstest]
	fn no_decay_always_one() {
		// Arrange
		let decay = NoDecay;

		// Assert
		assert_eq!(decay.weight(Duration::ZERO), 1.0);
		assert_eq!(decay.weight(Duration::from_secs(999_999)), 1.0);
	}
}
