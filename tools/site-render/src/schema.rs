//! Strict JSON boundary: validate types once, then use named Rust fields.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CatalogEntry {
    pub id: String,
    pub slug: String,
    pub path: String,
    pub title: String,
    pub description: String,
    pub group: String,
    pub source: Source,
    #[serde(rename = "review")]
    pub _review: Review,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Review {
    Expanded,
    Migrated,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub id: String,
    pub modified: String,
    #[serde(rename = "paragraphs")]
    pub _paragraphs: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct SiteData {
    pub icons: BTreeMap<String, String>,
    pub stages: Vec<Stage>,
    pub ownership_steps: [OwnershipStep; 4],
    pub flows: BTreeMap<String, Flow>,
    pub quizzes: BTreeMap<String, Quiz>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Stage {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub description: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnershipStep {
    pub code: String,
    pub source: String,
    pub target: String,
    pub owner: Option<String>,
    pub borrowed: Option<bool>,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Flow {
    pub title: String,
    pub nodes: Vec<String>,
    pub caption: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Quiz {
    pub prompt: String,
    pub choices: Vec<String>,
    pub answer: usize,
    pub explanation: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SearchEntry {
    pub title: String,
    pub id: String,
    pub description: String,
    pub href: String,
    pub text: String,
    pub headings: Vec<SearchHeading>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SearchHeading {
    pub id: String,
    pub label: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RenderReport {
    pub layout: String,
    pub pages: usize,
    pub markdown: Vec<MarkdownReport>,
    pub highlight: HighlightReport,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MarkdownReport {
    pub slug: String,
    pub renderer: String,
    pub reason: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HighlightReport {
    pub renderer: String,
    pub native: usize,
    pub total: usize,
    pub fallback: usize,
    pub by_language: BTreeMap<String, LanguageCount>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LanguageCount {
    pub native: usize,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_rejects_wrong_types_and_unknown_fields() {
        assert!(
            serde_json::from_str::<Quiz>(
                r#"{"prompt":"Q","choices":["A"],"answer":"0","explanation":"E"}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<Quiz>(
                r#"{"prompt":"Q","choices":["A"],"answer":0,"explanation":"E","typo":true}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<Quiz>(
                r#"{"prompt":"Q","choices":["A"],"answer":0,"explanation":"E"}"#
            )
            .is_ok()
        );
    }
}
