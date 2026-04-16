mod inverse_document_frequency;
mod term_frequency;

use std::collections::HashMap;

use crate::error::ContentBasedError;
use crate::types::{Document, FeatureVector};

/// Configuration for TF-IDF computation.
#[derive(Debug, Clone)]
pub struct TfIdfConfig {
	/// Whether to apply sublinear TF scaling (`1 + ln(tf)`).
	pub sublinear_tf: bool,
	/// Minimum document frequency for a term to be included.
	pub min_df: usize,
	/// Maximum document frequency ratio (0.0..=1.0) for a term to be included.
	pub max_df_ratio: f64,
}

impl Default for TfIdfConfig {
	fn default() -> Self {
		Self {
			sublinear_tf: false,
			min_df: 1,
			max_df_ratio: 1.0,
		}
	}
}

/// TF-IDF feature extractor that builds a vocabulary from a corpus and
/// transforms documents into feature vectors.
#[derive(Debug)]
pub struct TfIdf {
	vocabulary: Option<HashMap<String, usize>>,
	idf_values: Option<Vec<f64>>,
	config: TfIdfConfig,
}

impl TfIdf {
	/// Creates a new TF-IDF extractor with the given configuration.
	#[must_use]
	pub fn new(config: TfIdfConfig) -> Self {
		Self {
			vocabulary: None,
			idf_values: None,
			config,
		}
	}

	/// Builds the vocabulary and computes IDF values from the given corpus.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::EmptyCorpus` if the documents slice is empty.
	pub fn fit(&mut self, documents: &[Document]) -> Result<(), ContentBasedError> {
		if documents.is_empty() {
			return Err(ContentBasedError::EmptyCorpus);
		}

		let n = documents.len();

		// Count document frequency for each term
		let mut df_counts: HashMap<String, usize> = HashMap::new();
		for doc in documents {
			let mut seen = std::collections::HashSet::new();
			for token in &doc.tokens {
				if seen.insert(token.clone()) {
					*df_counts.entry(token.clone()).or_insert(0) += 1;
				}
			}
		}

		// Filter terms by min_df and max_df_ratio, then build vocabulary
		let mut vocabulary = HashMap::new();
		let mut idx = 0;
		// Precision loss is acceptable for practical corpus sizes
		#[allow(clippy::cast_precision_loss)]
		// Sign loss cannot occur: ceil of a non-negative product is non-negative
		#[allow(clippy::cast_sign_loss)]
		// Truncation is intentional: we want the ceiling as an integer bound
		#[allow(clippy::cast_possible_truncation)]
		let max_df = (self.config.max_df_ratio * n as f64).ceil() as usize;

		// Sort keys for deterministic vocabulary ordering
		let mut terms: Vec<_> = df_counts.keys().cloned().collect();
		terms.sort();

		for term in terms {
			let df = df_counts[&term];
			if df >= self.config.min_df && df <= max_df {
				vocabulary.insert(term, idx);
				idx += 1;
			}
		}

		let idf_values = inverse_document_frequency::compute_idf(documents, &vocabulary);

		self.vocabulary = Some(vocabulary);
		self.idf_values = Some(idf_values);

		Ok(())
	}

	/// Transforms a single document into a TF-IDF feature vector.
	///
	/// The vocabulary must have been built by calling [`fit`](Self::fit) first.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::VocabularyNotBuilt` if [`fit`](Self::fit)
	/// has not been called.
	pub fn transform(&self, document: &Document) -> Result<FeatureVector, ContentBasedError> {
		let vocabulary = self
			.vocabulary
			.as_ref()
			.ok_or(ContentBasedError::VocabularyNotBuilt)?;
		let idf_values = self
			.idf_values
			.as_ref()
			.ok_or(ContentBasedError::VocabularyNotBuilt)?;

		let tf = term_frequency::compute_tf(&document.tokens, vocabulary, self.config.sublinear_tf);

		// Multiply TF by IDF
		let tfidf: Vec<f64> = tf
			.iter()
			.zip(idf_values.iter())
			.map(|(t, i)| t * i)
			.collect();

		Ok(FeatureVector::new(tfidf))
	}

	/// Fits the vocabulary on the corpus and transforms all documents.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::EmptyCorpus` if the documents slice is empty.
	pub fn fit_transform(
		&mut self,
		documents: &[Document],
	) -> Result<Vec<FeatureVector>, ContentBasedError> {
		self.fit(documents)?;
		documents.iter().map(|doc| self.transform(doc)).collect()
	}

	/// Returns the vocabulary size, or `None` if not yet fitted.
	#[must_use]
	pub fn vocabulary_size(&self) -> Option<usize> {
		self.vocabulary.as_ref().map(HashMap::len)
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use crate::types::ItemId;

	use super::*;

	fn make_doc(id: u64, tokens: &[&str]) -> Document {
		Document {
			id: ItemId(id),
			tokens: tokens.iter().map(|s| s.to_string()).collect(),
		}
	}

	#[rstest]
	fn fit_empty_corpus_returns_error() {
		let mut tfidf = TfIdf::new(TfIdfConfig::default());
		let result = tfidf.fit(&[]);
		assert!(matches!(result, Err(ContentBasedError::EmptyCorpus)));
	}

	#[rstest]
	fn transform_without_fit_returns_error() {
		let tfidf = TfIdf::new(TfIdfConfig::default());
		let doc = make_doc(1, &["hello"]);
		let result = tfidf.transform(&doc);
		assert!(matches!(result, Err(ContentBasedError::VocabularyNotBuilt)));
	}

	#[rstest]
	fn fit_builds_vocabulary() {
		// Arrange
		let mut tfidf = TfIdf::new(TfIdfConfig::default());
		let docs = vec![make_doc(1, &["cat", "dog"]), make_doc(2, &["cat", "fish"])];

		// Act
		tfidf.fit(&docs).unwrap();

		// Assert
		assert_eq!(tfidf.vocabulary_size(), Some(3));
	}

	#[rstest]
	fn fit_transform_produces_correct_dimensions() {
		// Arrange
		let mut tfidf = TfIdf::new(TfIdfConfig::default());
		let docs = vec![make_doc(1, &["a", "b", "c"]), make_doc(2, &["a", "d"])];

		// Act
		let vectors = tfidf.fit_transform(&docs).unwrap();

		// Assert
		assert_eq!(vectors.len(), 2);
		assert_eq!(vectors[0].dimension(), 4); // a, b, c, d
		assert_eq!(vectors[1].dimension(), 4);
	}

	#[rstest]
	fn fit_transform_common_term_has_lower_weight() {
		// Arrange
		let mut tfidf = TfIdf::new(TfIdfConfig::default());
		let docs = vec![
			make_doc(1, &["common", "rare_a"]),
			make_doc(2, &["common", "rare_b"]),
			make_doc(3, &["common", "rare_c"]),
		];

		// Act
		let vectors = tfidf.fit_transform(&docs).unwrap();

		// Assert
		let vocab = tfidf.vocabulary.as_ref().unwrap();
		let common_idx = vocab["common"];
		let rare_a_idx = vocab["rare_a"];

		// Common term (in all docs) should have lower TF-IDF than rare term
		// Both appear once in doc 0, but IDF differs
		assert!(vectors[0].values[common_idx].abs() < vectors[0].values[rare_a_idx].abs());
	}

	#[rstest]
	fn min_df_filters_rare_terms() {
		// Arrange
		let config = TfIdfConfig {
			min_df: 2,
			..TfIdfConfig::default()
		};
		let mut tfidf = TfIdf::new(config);
		let docs = vec![make_doc(1, &["common", "rare"]), make_doc(2, &["common"])];

		// Act
		tfidf.fit(&docs).unwrap();

		// Assert — "rare" appears in only 1 doc, filtered out
		assert_eq!(tfidf.vocabulary_size(), Some(1));
		assert!(tfidf.vocabulary.as_ref().unwrap().contains_key("common"));
	}

	#[rstest]
	fn max_df_ratio_filters_common_terms() {
		// Arrange
		let config = TfIdfConfig {
			max_df_ratio: 0.5,
			..TfIdfConfig::default()
		};
		let mut tfidf = TfIdf::new(config);
		let docs = vec![
			make_doc(1, &["everywhere", "unique_a"]),
			make_doc(2, &["everywhere", "unique_b"]),
		];

		// Act
		tfidf.fit(&docs).unwrap();

		// Assert — "everywhere" appears in 100% of docs (ratio 1.0 > 0.5), filtered out
		let vocab = tfidf.vocabulary.as_ref().unwrap();
		assert!(!vocab.contains_key("everywhere"));
		assert_eq!(tfidf.vocabulary_size(), Some(2));
	}

	#[rstest]
	fn transform_with_only_unknown_tokens_produces_zero_vector() {
		// Arrange
		let mut tfidf = TfIdf::new(TfIdfConfig::default());
		let docs = vec![make_doc(1, &["cat", "dog"]), make_doc(2, &["cat", "fish"])];
		tfidf.fit(&docs).unwrap();

		let unknown_doc = make_doc(3, &["zebra", "elephant"]);

		// Act
		let vector = tfidf.transform(&unknown_doc).unwrap();

		// Assert
		assert_eq!(vector.dimension(), 3); // cat, dog, fish
		assert!(vector.values.iter().all(|&v| v == 0.0));
	}

	#[rstest]
	fn fit_min_df_filters_all_vocabulary() {
		// Arrange — every term appears in only 1 doc, min_df=2 filters all
		let config = TfIdfConfig {
			min_df: 2,
			..TfIdfConfig::default()
		};
		let mut tfidf = TfIdf::new(config);
		let docs = vec![make_doc(1, &["alpha"]), make_doc(2, &["beta"])];

		// Act
		tfidf.fit(&docs).unwrap();

		// Assert
		assert_eq!(tfidf.vocabulary_size(), Some(0));
	}

	#[rstest]
	fn fit_with_empty_token_documents_mixed_in() {
		// Arrange
		let mut tfidf = TfIdf::new(TfIdfConfig::default());
		let docs = vec![
			make_doc(1, &["hello", "world"]),
			make_doc(2, &[]),
			make_doc(3, &["hello"]),
		];

		// Act
		tfidf.fit(&docs).unwrap();

		// Assert — "hello" (df=2), "world" (df=1) both pass default min_df=1
		assert_eq!(tfidf.vocabulary_size(), Some(2));
	}

	#[rstest]
	fn sublinear_tf_applied() {
		// Arrange — use 3 docs so "word" (df=1) has non-zero IDF: ln(3/2)
		let config = TfIdfConfig {
			sublinear_tf: true,
			..TfIdfConfig::default()
		};
		let mut tfidf_sub = TfIdf::new(config);
		let mut tfidf_raw = TfIdf::new(TfIdfConfig::default());

		let docs = vec![
			make_doc(1, &["word", "word", "word", "other"]),
			make_doc(2, &["other"]),
			make_doc(3, &["another"]),
		];

		// Act
		let vecs_sub = tfidf_sub.fit_transform(&docs).unwrap();
		let vecs_raw = tfidf_raw.fit_transform(&docs).unwrap();

		// Assert — sublinear TF for "word" (tf=3) should be less than raw TF
		let vocab = tfidf_sub.vocabulary.as_ref().unwrap();
		let word_idx = vocab["word"];
		// With sublinear: (1 + ln(3)) * idf, without: 3 * idf
		// Sublinear value should be smaller in absolute terms
		assert!(vecs_sub[0].values[word_idx].abs() < vecs_raw[0].values[word_idx].abs());
	}
}
