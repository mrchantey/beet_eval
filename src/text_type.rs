//! The validated text newtypes the rows are keyed and cross-referenced by:
//! an id, a tag, an address, a column, a cell.

/// Declares a validated text newtype over [`SmolStr`](beet::prelude::SmolStr).
///
/// Serde goes through the string (`try_from`/`into`), so a deserialized value
/// holds the invariant `$check` enforces, and reflect sees a one-field tuple
/// struct, which the derived schema unwraps to a text field: the stored form
/// and the schema agree. `$check` is a closure body over the text returning
/// [`Result`](beet::prelude::Result).
macro_rules! text_type {
	($(#[$meta:meta])* $name:ident, |$text:ident| $check:expr) => {
		$(#[$meta])*
		#[derive(
			Debug,
			Clone,
			PartialEq,
			Eq,
			PartialOrd,
			Ord,
			Hash,
			Reflect,
			Serialize,
			Deserialize,
		)]
		#[serde(try_from = "SmolStr", into = "SmolStr")]
		pub struct $name(SmolStr);

		impl $name {
			/// Validates `text` as this type.
			pub fn parse(text: impl Into<SmolStr>) -> Result<Self> {
				let $text: SmolStr = text.into();
				let checked: Result = $check;
				checked.map(|_| Self($text))
			}

			/// The text as written.
			pub fn as_str(&self) -> &str { &self.0 }

			/// Teaches every authoring seam, request params included, to read
			/// this type, validated, from its text or from a greedy route
			/// capture's segments joined by `/`; `hint` is how `--help` names
			/// it.
			pub fn literal_parser(hint: &'static str) -> LiteralParser {
				LiteralParser::new(|value: &Value| match value {
					Value::Str(text) => Self::parse(text.as_str()).map(Some),
					Value::List(segments) => segments
						.iter()
						.map(|segment| match segment {
							Value::Str(segment) => Some(segment.as_str()),
							_ => None,
						})
						.collect::<Option<Vec<_>>>()
						.map(|segments| Self::parse(segments.join("/")))
						.transpose(),
					_ => Ok(None),
				})
				.with_hint(hint)
			}
		}

		impl TryFrom<SmolStr> for $name {
			type Error = BevyError;
			fn try_from(text: SmolStr) -> Result<Self> { Self::parse(text) }
		}

		impl From<$name> for SmolStr {
			fn from(value: $name) -> SmolStr { value.0 }
		}

		impl core::fmt::Display for $name {
			fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
				formatter.write_str(&self.0)
			}
		}

		impl core::ops::Deref for $name {
			type Target = str;
			fn deref(&self) -> &str { &self.0 }
		}
	};
}
pub(crate) use text_type;
