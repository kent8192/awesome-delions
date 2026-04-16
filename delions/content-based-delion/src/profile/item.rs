use crate::error::ContentBasedError;
use crate::tfidf::{TfIdf, TfIdfConfig};
use crate::types::{Document, FeatureVector, ItemId};

/// A feature profile for an item, built from TF-IDF vectors.
#[derive(Debug, Clone)]
pub struct ItemProfile {
	pub id: ItemId,
	pub features: FeatureVector,
}

/// Builds item profiles from a document corpus using TF-IDF.
#[derive(Debug)]
pub struct ItemProfileBuilder {
	tfidf: TfIdf,
	profiles: Vec<ItemProfile>,
}

impl ItemProfileBuilder {
	/// Creates a new builder with the given TF-IDF configuration.
	#[must_use]
	pub fn new(config: TfIdfConfig) -> Self {
		Self {
			tfidf: TfIdf::new(config),
			profiles: Vec::new(),
		}
	}

	/// Builds item profiles from the given documents.
	///
	/// Fits the TF-IDF model on the corpus and transforms each document
	/// into a feature vector.
	///
	/// # Errors
	///
	/// Returns `ContentBasedError::EmptyCorpus` if the documents slice is empty.
	pub fn build_profiles(
		&mut self,
		documents: &[Document],
	) -> Result<Vec<ItemProfile>, ContentBasedError> {
		let vectors = self.tfidf.fit_transform(documents)?;

		self.profiles = documents
			.iter()
			.zip(vectors)
			.map(|(doc, features)| ItemProfile {
				id: doc.id,
				features,
			})
			.collect();

		Ok(self.profiles.clone())
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	fn make_doc(id: u64, tokens: &[&str]) -> Document {
		Document {
			id: ItemId(id),
			tokens: tokens.iter().map(|s| s.to_string()).collect(),
		}
	}

	#[rstest]
	fn build_profiles_creates_one_per_document() {
		// Arrange
		let mut builder = ItemProfileBuilder::new(TfIdfConfig::default());
		let docs = vec![
			make_doc(1, &["rust", "programming"]),
			make_doc(2, &["python", "programming"]),
			make_doc(3, &["rust", "systems"]),
		];

		// Act
		let profiles = builder.build_profiles(&docs).unwrap();

		// Assert
		assert_eq!(profiles.len(), 3);
		assert_eq!(profiles[0].id, ItemId(1));
		assert_eq!(profiles[1].id, ItemId(2));
		assert_eq!(profiles[2].id, ItemId(3));
	}

	#[rstest]
	fn build_profiles_vectors_have_consistent_dimensions() {
		// Arrange
		let mut builder = ItemProfileBuilder::new(TfIdfConfig::default());
		let docs = vec![make_doc(1, &["a", "b"]), make_doc(2, &["c", "d"])];

		// Act
		let profiles = builder.build_profiles(&docs).unwrap();

		// Assert
		assert_eq!(
			profiles[0].features.dimension(),
			profiles[1].features.dimension()
		);
	}

	#[rstest]
	fn build_profiles_empty_corpus_returns_error() {
		let mut builder = ItemProfileBuilder::new(TfIdfConfig::default());
		let result = builder.build_profiles(&[]);
		assert!(matches!(result, Err(ContentBasedError::EmptyCorpus)));
	}
}
