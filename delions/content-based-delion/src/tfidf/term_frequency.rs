use std::collections::HashMap;

/// Computes term frequency vector for the given tokens against the vocabulary.
///
/// If `sublinear` is true, applies sublinear TF scaling: `1 + ln(tf)` for `tf > 0`.
pub fn compute_tf(
	tokens: &[String],
	vocabulary: &HashMap<String, usize>,
	sublinear: bool,
) -> Vec<f64> {
	let vocab_size = vocabulary.len();
	let mut tf = vec![0.0_f64; vocab_size];

	// Count raw term frequencies
	let mut counts = HashMap::new();
	for token in tokens {
		if vocabulary.contains_key(token) {
			*counts.entry(token.as_str()).or_insert(0u64) += 1;
		}
	}

	for (term, &count) in &counts {
		if let Some(&idx) = vocabulary.get(*term) {
			// Precision loss is acceptable for term frequency counts
			#[allow(clippy::cast_precision_loss)]
			let raw_tf = count as f64;
			tf[idx] = if sublinear { 1.0 + raw_tf.ln() } else { raw_tf };
		}
	}

	tf
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	fn make_vocab(terms: &[&str]) -> HashMap<String, usize> {
		terms
			.iter()
			.enumerate()
			.map(|(i, &t)| (t.to_string(), i))
			.collect()
	}

	#[rstest]
	fn tf_basic_counts() {
		// Arrange
		let vocab = make_vocab(&["cat", "dog", "fish"]);
		let tokens: Vec<String> = ["cat", "dog", "cat"]
			.iter()
			.map(|s| s.to_string())
			.collect();

		// Act
		let tf = compute_tf(&tokens, &vocab, false);

		// Assert
		assert_eq!(tf[vocab["cat"]], 2.0);
		assert_eq!(tf[vocab["dog"]], 1.0);
		assert_eq!(tf[vocab["fish"]], 0.0);
	}

	#[rstest]
	fn tf_sublinear() {
		// Arrange
		let vocab = make_vocab(&["hello", "world"]);
		let tokens: Vec<String> = ["hello", "hello", "hello"]
			.iter()
			.map(|s| s.to_string())
			.collect();

		// Act
		let tf = compute_tf(&tokens, &vocab, true);

		// Assert
		let expected = 1.0 + (3.0_f64).ln();
		assert!((tf[vocab["hello"]] - expected).abs() < 1e-10);
		assert_eq!(tf[vocab["world"]], 0.0);
	}

	#[rstest]
	fn tf_empty_tokens() {
		let vocab = make_vocab(&["a", "b"]);
		let tokens: Vec<String> = vec![];
		let tf = compute_tf(&tokens, &vocab, false);
		assert_eq!(tf, vec![0.0, 0.0]);
	}

	#[rstest]
	fn tf_same_token_repeated_many_times() {
		// Arrange
		let vocab = make_vocab(&["word"]);
		let tokens: Vec<String> = std::iter::repeat_n("word".to_string(), 100).collect();

		// Act
		let tf = compute_tf(&tokens, &vocab, false);

		// Assert
		assert_eq!(tf[vocab["word"]], 100.0);
	}

	#[rstest]
	fn tf_unknown_tokens_ignored() {
		// Arrange
		let vocab = make_vocab(&["known"]);
		let tokens: Vec<String> = ["unknown", "known"]
			.iter()
			.map(|s| s.to_string())
			.collect();

		// Act
		let tf = compute_tf(&tokens, &vocab, false);

		// Assert
		assert_eq!(tf[vocab["known"]], 1.0);
	}
}
