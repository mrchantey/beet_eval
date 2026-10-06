use beet::prelude::*;

/// `check/asks`: passes when no ask is open anywhere under the documents
/// directory, and lists where the open ones are.
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AsksCheckParams {}
