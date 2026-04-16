use std::collections::HashMap;

use crate::types::Document;

/// Computes IDF values for each term in the vocabulary.
///
/// Uses the formula: `idf(t) = ln(N / (1 + df(t)))` where `df(t)` is the
/// number of documents containing term `t` and `N` is the total number of
/// documents.
pub fn compute_idf(documents: &[Document], vocabulary: &HashMap<String, usize>) -> Vec<f64> {
	let vocab_size = vocabulary.len();
	// Precision loss is acceptable for document counts within practical corpus sizes
	#[allow(clippy::cast_precision_loss)]
	let n = documents.len() as f64;
	let mut df = vec![0u64; vocab_size];

	for doc in documents {
		// Count each term at most once per document
		let mut seen = std::collections::HashSet::new();
		for token in &doc.tokens {
			if let Some(&idx) = vocabulary.get(token.as_str())
				&& seen.insert(idx)
			{
				df[idx] += 1;
			}
		}
	}

	df.iter()
		.map(|&count| {
			// Precision loss is acceptable for term frequency counts
			#[allow(clippy::cast_precision_loss)]
			let df_f64 = count as f64;
			(n / (1.0 + df_f64)).ln()
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use crate::types::ItemId;

	use super::*;

	fn make_vocab(terms: &[&str]) -> HashMap<String, usize> {
		terms
			.iter()
			.enumerate()
			.map(|(i, &t)| (t.to_string(), i))
			.collect()
	}

	fn make_doc(id: u64, tokens: &[&str]) -> Document {
		Document {
			id: ItemId(id),
			tokens: tokens.iter().map(|s| s.to_string()).collect(),
		}
	}

	#[rstest]
	fn idf_basic() {
		// Arrange
		let vocab = make_vocab(&["cat", "dog"]);
		let docs = vec![
			make_doc(1, &["cat", "dog"]),
			make_doc(2, &["cat"]),
			make_doc(3, &["dog"]),
		];

		// Act
		let idf = compute_idf(&docs, &vocab);

		// Assert
		// cat appears in 2 docs: ln(3 / (1+2)) = ln(1) = 0
		assert!((idf[vocab["cat"]] - (3.0_f64 / 3.0).ln()).abs() < 1e-10);
		// dog appears in 2 docs: ln(3 / (1+2)) = ln(1) = 0
		assert!((idf[vocab["dog"]] - (3.0_f64 / 3.0).ln()).abs() < 1e-10);
	}

	#[rstest]
	fn idf_rare_term_higher() {
		// Arrange
		let vocab = make_vocab(&["common", "rare"]);
		let docs = vec![
			make_doc(1, &["common", "rare"]),
			make_doc(2, &["common"]),
			make_doc(3, &["common"]),
		];

		// Act
		let idf = compute_idf(&docs, &vocab);

		// Assert
		// common: ln(3/4), rare: ln(3/2) — rare has higher IDF
		assert!(idf[vocab["rare"]] > idf[vocab["common"]]);
	}

	#[rstest]
	fn idf_term_not_in_any_doc() {
		// Arrange
		let vocab = make_vocab(&["present", "absent"]);
		let docs = vec![make_doc(1, &["present"])];

		// Act
		let idf = compute_idf(&docs, &vocab);

		// Assert
		// absent: ln(1 / (1+0)) = ln(1) = 0
		assert!((idf[vocab["absent"]] - 0.0).abs() < 1e-10);
		// present: ln(1 / (1+1)) = ln(0.5) < 0
		assert!((idf[vocab["present"]] - (0.5_f64).ln()).abs() < 1e-10);
	}

	#[rstest]
	fn idf_duplicate_tokens_counted_once_per_doc() {
		// Arrange
		let vocab = make_vocab(&["word"]);
		let docs = vec![make_doc(1, &["word", "word", "word"])];

		// Act
		let idf = compute_idf(&docs, &vocab);

		// Assert
		// word appears in 1 doc: ln(1 / (1+1)) = ln(0.5)
		assert!((idf[vocab["word"]] - (1.0_f64 / 2.0).ln()).abs() < 1e-10);
	}
}
