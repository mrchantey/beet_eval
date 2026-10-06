use beet::prelude::*;

/// A fenced block named in its info string, `csv <name>` for a table and
/// `json <name>` only where the data nests: anything tabular or numeric that
/// another document or a renderer consumes. A name is defined once across the
/// documents, in the document that owns the fact, and every other document
/// links to it.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DataBlock {
	/// The name after the format in the info string, ie `price-list`.
	pub name: SmolStr,
	/// The fence's format.
	pub format: BlockFormat,
	/// The opening fence's line in its file, from 1.
	pub line: u32,
	/// The slug of the section it sits in, absent above the first.
	pub section: Option<SmolStr>,
	/// A `csv` block's first row, the column names.
	pub header: Vec<String>,
	/// A `csv` block's other rows, cells trimmed and unquoted.
	pub rows: Vec<Vec<String>>,
	/// The fence's body as written.
	pub text: String,
}

/// The two formats a data block may take.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum BlockFormat {
	/// A table: a header row then data rows, double quotes grouping a cell.
	Csv,
	/// Nested data.
	Json,
}

/// One column of a block's schema, as a document package's
/// [`Outline`](crate::prelude::Outline) fixes it. Its flat
/// spelling, `name` or `name:kind`, is what a check's params and a reader
/// carry.
#[derive(Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub struct Column {
	/// Lowercase letters, digits and underscores, starting with a letter, ie
	/// `price_ex_gst`.
	pub name: SmolStr,
	/// What every cell of the column must hold.
	pub kind: ColumnKind,
}

impl Column {
	/// Parses the flat spelling, `name` or `name:kind`, ie `price_ex_gst:num`.
	pub fn parse(text: &str) -> Result<Self> {
		let (name, kind) = match text.split_once(':') {
			Some((name, kind)) => (name, ColumnKind::parse(kind)?),
			None => (text, ColumnKind::Text),
		};
		match name.starts_with(|char: char| char.is_ascii_lowercase())
			&& name.chars().all(|char| {
				char.is_ascii_lowercase()
					|| char.is_ascii_digit()
					|| char == '_'
			}) {
			true => Self {
				name: name.into(),
				kind,
			}
			.xok(),
			false => bevybail!(
				"`{name}` is not a column name, expected lowercase letters, \
				 digits and underscores"
			),
		}
	}
}

/// The flat spelling, the kind omitted for [`ColumnKind::Text`].
impl core::fmt::Display for Column {
	fn fmt(
		&self,
		formatter: &mut core::fmt::Formatter<'_>,
	) -> core::fmt::Result {
		match self.kind {
			ColumnKind::Text => formatter.write_str(&self.name),
			kind => write!(formatter, "{}:{}", self.name, kind.spelling()),
		}
	}
}

/// What a column's cells hold. Only the typed kinds are enforced: a cell that
/// does not apply is a 0 with the reason in the prose.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize,
)]
pub enum ColumnKind {
	/// Free text.
	Text,
	/// A number, `-?digits` with an optional decimal part; money is whole
	/// dollars unless the schema says otherwise.
	Num,
	/// A month, `2026-10`.
	Month,
	/// A date, `2026-10-02`.
	Date,
}

impl ColumnKind {
	/// The kind as a column's flat spelling writes it, ie `num`.
	pub fn spelling(&self) -> &'static str {
		match self {
			Self::Text => "text",
			Self::Num => "num",
			Self::Month => "month",
			Self::Date => "date",
		}
	}

	/// The typed kinds only: a text column is spelled by its name alone.
	fn parse(text: &str) -> Result<Self> {
		match text {
			"num" => Self::Num,
			"month" => Self::Month,
			"date" => Self::Date,
			other => bevybail!(
				"`{other}` is not a column kind, expected num, month or date"
			),
		}
		.xok()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	#[beet::test]
	fn columns_round_trip_their_spelling() {
		for text in ["line", "price_ex_gst:num", "month:month", "start:date"] {
			Column::parse(text).unwrap().to_string().xpect_eq(text);
		}
		Column::parse("price:money").xpect_err();
		Column::parse("Price").xpect_err();
	}
}
