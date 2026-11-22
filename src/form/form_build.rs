use crate::prelude::*;
use beet::prelude::*;

/// A form's build in the build directory: the blank form unpacked into
/// `<package>/<rubric>/`, one file per entry of its zip, which the builder
/// edits as the form's own XML with the store's text tools, and the files
/// zipped back into the built form.
///
/// Every entry is kept as the blank form stored it, its content types and
/// relationships included, so what a builder leaves untouched comes back
/// byte for byte, and Word opens the result looking as its author made it.
pub struct FormBuild;

impl FormBuild {
	/// The directory below the build directory the form of `rubric` is
	/// unpacked into, ie `acme_course/01-business-plan`.
	pub fn parts_dir(rubric: &RubricRef) -> RelPath {
		RelPath::new(rubric.package()).join(rubric.rubric())
	}

	/// The path below the build directory a build writes the built form's
	/// [`CellsReport`] to, beside the form, which the triage reads as built.
	pub fn cells_path(rubric: &RubricRef) -> RelPath {
		RelPath::new(rubric.package())
			.join(format!("{}.cells.md", rubric.rubric()))
	}

	/// The path below the build directory a form's [`AnswerSheet`] is
	/// written to, beside the form, ie `acme_course/01-business-plan.answers.md`.
	pub fn answers_path(rubric: &RubricRef) -> RelPath {
		RelPath::new(rubric.package())
			.join(format!("{}.answers.md", rubric.rubric()))
	}

	/// Whether `parts` holds an unpacked form.
	pub async fn is_unpacked(parts: &BlobStore) -> Result<bool> {
		parts
			.exists(&RelPath::new(OpcPackage::CONTENT_TYPES_PATH))
			.await
	}

	/// Unpacks `form` into `parts`, first removing whatever `parts` held,
	/// answering the count of files written.
	pub async fn unpack(parts: &BlobStore, form: &MediaBytes) -> Result<usize> {
		let entries = OpcPackage::read_entries(form.bytes())?;
		if !entries
			.iter()
			.any(|(path, _)| path == OpcPackage::CONTENT_TYPES_PATH)
		{
			bevybail!("the form is no Office file, so it cannot be unpacked");
		}
		for path in parts.list().await? {
			parts.remove(&path).await?;
		}
		let count = entries.len();
		for (path, bytes) in entries {
			parts.insert(&RelPath::new(path.as_str()), bytes).await?;
		}
		count.xok()
	}

	/// The files of `parts` zipped into a form of `media_type`, the content
	/// types first as an Office file lists them.
	pub async fn pack(
		parts: &BlobStore,
		media_type: MediaType,
	) -> Result<MediaBytes> {
		let mut paths = parts.list().await?;
		paths.sort_by_key(|path| {
			(
				path.to_string() != OpcPackage::CONTENT_TYPES_PATH,
				path.to_string(),
			)
		});
		let mut entries = Vec::with_capacity(paths.len());
		for path in paths {
			let bytes = parts.get(&path).await?;
			entries.push((path.to_string(), bytes.to_vec()));
		}
		MediaBytes::new(media_type, OpcPackage::write_entries(entries)?).xok()
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// A form unpacked, one of its parts edited as text and packed again
	/// reads with the edit, and an untouched form packs to the same entries.
	#[beet::test]
	async fn edits_a_form_through_its_parts() {
		let parts = BlobStore::temp();
		let blank = Fixture::blank_form();
		FormBuild::unpack(&parts, &blank).await.unwrap();
		FormBuild::is_unpacked(&parts).await.unwrap().xpect_true();
		let packed = FormBuild::pack(&parts, MediaType::Docx).await.unwrap();
		let mut entries = OpcPackage::read_entries(packed.bytes()).unwrap();
		entries[0]
			.0
			.as_str()
			.xpect_eq(OpcPackage::CONTENT_TYPES_PATH);
		entries.sort();
		let mut original = OpcPackage::read_entries(blank.bytes()).unwrap();
		original.sort();
		entries.xpect_eq(original);

		let document = RelPath::new("word/document.xml");
		let xml =
			String::from_utf8(parts.get(&document).await.unwrap().to_vec())
				.unwrap()
				.replacen(
					"<w:tc><w:p/></w:tc>",
					"<w:tc><w:p><w:r><w:t>Acme Stalls</w:t></w:r></w:p></w:tc>",
					1,
				);
		parts.insert(&document, xml).await.unwrap();
		let built = FormBuild::pack(&parts, MediaType::Docx).await.unwrap();
		Fixture::cells_of(&built)[1]
			.to_string()
			.xpect_eq("| t0/1,0 | Acme Stalls |");
	}
}
