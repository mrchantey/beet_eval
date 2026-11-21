use beet::prelude::*;

/// The triage a reader of a deck reads before transcribing it: each slide
/// sorted by what carries it, words or a picture, every picture's size and
/// cover, and what to watch for, hidden slides, recurring pictures,
/// same-size pictures and exact duplicates.
///
/// It reads the deck as beet reads one, an `article data-card` per slide, its
/// front before its first `hr` and its speaker notes after, so a slide is
/// counted from 0 as the card stack counts it, and a picture's cover is its
/// `width` and `height` against the 1280 by 720 px slide a deck reads in.
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DeckTriage {
	/// Every slide, in presentation order.
	pub slides: Vec<SlideTriage>,
	/// Every picture of every slide's front, in order.
	pub pictures: Vec<PictureTriage>,
	/// What to watch for while transcribing.
	pub findings: Vec<String>,
}

/// One slide as the triage reads it.
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SlideTriage {
	/// Its place in the deck, from 0.
	pub slide: u32,
	/// Its title, its first `h1`, empty for an untitled slide.
	pub title: String,
	/// Whether a slideshow skips it.
	pub hidden: bool,
	/// The words of its front.
	pub words: usize,
	/// Its front's pictures.
	pub pictures: usize,
	/// The largest share of the slide a picture covers, in percent.
	pub largest: u32,
	/// Its front's tables.
	pub tables: usize,
	/// The words of its speaker notes.
	pub notes_words: usize,
	/// What carries it, ie `text`, `picture` or `duplicate of slide 3`.
	pub carried_by: String,
}

/// One picture of a slide's front.
#[derive(Debug, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct PictureTriage {
	/// The slide it is on.
	pub slide: u32,
	/// Its name across the deck, the same bytes the same name, ie
	/// `picture 2`, counted from 0 in order of first appearance.
	pub picture: String,
	/// Its pixel size as its bytes declare it, ie `1920x1080`, empty for a
	/// linked picture or a format whose header is not read.
	pub native: String,
	/// The share of the slide it covers, in percent.
	pub cover: u32,
}

impl DeckTriage {
	/// The slide size a deck reads in, in CSS px.
	pub const SLIDE: UVec2 = UVec2::new(1280, 720);
	/// A picture covering at least this share of a slide, in percent, carries
	/// content rather than decoration, so a recurrence of it is reported.
	const RECURRING_FLOOR: u32 = 25;

	/// The triage of the deck under `root`.
	pub fn of(world: &mut World, root: Entity) -> Self {
		let slides =
			world.with_state::<DeckQuery, _>(|query| query.slides(root));
		Self::from_slides(slides)
	}

	fn from_slides(slides: Vec<ReadSlide>) -> Self {
		// each distinct picture named in order of first appearance
		let mut names = Vec::<&str>::new();
		let mut pictures = Vec::new();
		for slide in &slides {
			for picture in &slide.pictures {
				let index =
					match names.iter().position(|src| *src == picture.src) {
						Some(index) => index,
						None => {
							names.push(&picture.src);
							names.len() - 1
						}
					};
				pictures.push(PictureTriage {
					slide: slide.index,
					picture: format!("picture {index}"),
					native: picture
						.native
						.map(|size| format!("{}x{}", size.x, size.y))
						.unwrap_or_default(),
					cover: picture.cover,
				});
			}
		}
		let mut seen = HashMap::<&str, u32>::default();
		let rows = slides
			.iter()
			.map(|slide| {
				let duplicate_of = match slide.fingerprint.is_empty() {
					true => None,
					false => {
						let first =
							seen.get(slide.fingerprint.as_str()).copied();
						seen.entry(&slide.fingerprint).or_insert(slide.index);
						first
					}
				};
				SlideTriage {
					slide: slide.index,
					title: slide.title.clone(),
					hidden: slide.hidden,
					words: slide.words,
					pictures: slide.pictures.len(),
					largest: slide.largest(),
					tables: slide.tables,
					notes_words: slide.notes_words,
					carried_by: match duplicate_of {
						Some(first) => format!("duplicate of slide {first}"),
						None => slide.carried_by().into(),
					},
				}
			})
			.collect::<Vec<_>>();
		let findings = Self::findings(&rows, &pictures);
		Self {
			slides: rows,
			pictures,
			findings,
		}
	}

	/// What to watch for: hidden slides, recurring pictures, same-size
	/// pictures, slides carried by a picture and exact duplicates.
	fn findings(
		slides: &[SlideTriage],
		pictures: &[PictureTriage],
	) -> Vec<String> {
		let joined = |numbers: &[u32]| {
			numbers
				.iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>()
				.join(", ")
		};
		let mut findings = Vec::new();
		let hidden = slides
			.iter()
			.filter(|slide| slide.hidden)
			.map(|slide| slide.slide)
			.collect::<Vec<_>>();
		if !hidden.is_empty() {
			findings.push(format!(
				"hidden slide(s) {}: in the deck, not shown in a slideshow and dropped from a render. \
				 Transcribe from the dump and say it is hidden.",
				joined(&hidden)
			));
		}
		let large = pictures
			.iter()
			.filter(|picture| picture.cover >= Self::RECURRING_FLOOR)
			.collect::<Vec<_>>();
		// a picture big enough to carry content, shown on several slides
		let mut recurring = BTreeMap::<&str, Vec<u32>>::new();
		for picture in &large {
			let on = recurring.entry(&picture.picture).or_default();
			if on.last() != Some(&picture.slide) {
				on.push(picture.slide);
			}
		}
		for (picture, on) in recurring.iter().filter(|(_, on)| on.len() >= 2) {
			let span = match on.windows(2).all(|pair| pair[1] == pair[0] + 1) {
				true => format!("{}-{}", on[0], on[on.len() - 1]),
				false => joined(on),
			};
			findings.push(format!(
				"{picture} is on {} slides ({span}): the same picture twice is one artefact re-shown, \
				 often with a different box drawn on it. Transcribe it once and cross-reference.",
				on.len()
			));
		}
		// one capture saved twice: different bytes of identical dimensions
		let mut by_size = BTreeMap::<&str, BTreeMap<&str, Vec<u32>>>::new();
		for picture in large
			.iter()
			.filter(|picture| picture.cover < 90 && !picture.native.is_empty())
		{
			let on = by_size
				.entry(&picture.native)
				.or_default()
				.entry(&picture.picture)
				.or_default();
			if on.last() != Some(&picture.slide) {
				on.push(picture.slide);
			}
		}
		for (size, names) in by_size
			.iter()
			.filter(|(_, names)| (2..=3).contains(&names.len()))
		{
			let at = names
				.iter()
				.map(|(name, on)| format!("{name} on {}", joined(on)))
				.collect::<Vec<_>>()
				.join("; ");
			findings.push(format!(
				"{} pictures of the same size, {size} ({at}): often one capture saved twice. \
				 Compare them before transcribing both.",
				names.len()
			));
		}
		let pictured = slides
			.iter()
			.filter(|slide| slide.carried_by == "picture")
			.map(|slide| slide.slide)
			.collect::<Vec<_>>();
		if !pictured.is_empty() {
			findings.push(format!(
				"read and transcribe these slides' pictures: {}",
				joined(&pictured)
			));
		}
		let duplicates = slides
			.iter()
			.filter_map(|slide| {
				slide
					.carried_by
					.strip_prefix("duplicate of slide ")
					.map(|first| format!("{}=={first}", slide.slide))
			})
			.collect::<Vec<_>>();
		if !duplicates.is_empty() {
			findings.push(format!(
				"duplicate slides (say so once, do not transcribe twice): {}",
				duplicates.join(", ")
			));
		}
		findings
	}
}

/// The triage as a scene: a line of the deck's size, the slides, the
/// pictures and the findings.
#[template]
pub fn DeckTriageReport(
	/// The deck's path in the store.
	file: String,
	#[prop(required)] triage: DeckTriage,
) -> impl Bundle {
	let blank_zero = |count: usize| match count {
		0 => String::new(),
		count => count.to_string(),
	};
	let slides = triage
		.slides
		.iter()
		.map(|slide| {
			let hidden = match slide.hidden {
				true => "hidden",
				false => "",
			};
			rsx! {
				<tr>
					<td>{slide.slide.to_string()}</td>
					<td>{slide.title.clone()}</td>
					<td>{hidden}</td>
					<td>{slide.words.to_string()}</td>
					<td>{blank_zero(slide.pictures)}</td>
					<td>{format!("{}%", slide.largest)}</td>
					<td>{blank_zero(slide.tables)}</td>
					<td>{blank_zero(slide.notes_words)}</td>
					<td>{slide.carried_by.clone()}</td>
				</tr>
			}
		})
		.collect::<Vec<_>>();
	let pictures = triage
		.pictures
		.iter()
		.map(|picture| {
			rsx! {
				<tr>
					<td>{picture.slide.to_string()}</td>
					<td>{picture.picture.clone()}</td>
					<td>{picture.native.clone()}</td>
					<td>{format!("{}%", picture.cover)}</td>
				</tr>
			}
		})
		.collect::<Vec<_>>();
	let findings = triage
		.findings
		.iter()
		.map(|finding| rsx! { <p>{finding.clone()}</p> })
		.collect::<Vec<_>>();
	rsx! {
		<h1>{format!("Triage of {file}")}</h1>
		<p>{format!(
			"{} slides, counted from 0; a picture's cover is its share of the {} by {} px slide.",
			triage.slides.len(),
			DeckTriage::SLIDE.x,
			DeckTriage::SLIDE.y
		)}</p>
		<h2>"Slides"</h2>
		<table>
			<tr>
				<th>"Slide"</th><th>"Title"</th><th>"Hidden"</th><th>"Words"</th><th>"Pictures"</th>
				<th>"Largest"</th><th>"Tables"</th><th>"Notes"</th><th>"Carried by"</th>
			</tr>
			{slides}
		</table>
		<h2>"Pictures"</h2>
		<table>
			<tr><th>"Slide"</th><th>"Picture"</th><th>"Native px"</th><th>"Covers"</th></tr>
			{pictures}
		</table>
		<h2>"Findings"</h2>
		{findings}
	}
}

/// One slide as read off its article, before it is sorted.
struct ReadSlide {
	index: u32,
	title: String,
	hidden: bool,
	words: usize,
	tables: usize,
	notes_words: usize,
	pictures: Vec<ReadPicture>,
	/// Its front's text and pictures, empty when it has neither, so two
	/// slides alike are exact duplicates.
	fingerprint: String,
}

/// One picture of a slide's front.
struct ReadPicture {
	src: String,
	native: Option<UVec2>,
	cover: u32,
}

impl ReadSlide {
	fn largest(&self) -> u32 {
		self.pictures
			.iter()
			.map(|picture| picture.cover)
			.max()
			.unwrap_or(0)
	}

	/// Which half of the slide holds the content: a picture covering the
	/// whole slide under real text is a background photo, one covering half
	/// of it is a diagram or a screenshot to be read.
	fn carried_by(&self) -> &'static str {
		let total = self
			.pictures
			.iter()
			.map(|picture| picture.cover)
			.sum::<u32>();
		match (self.words, self.pictures.len(), self.largest()) {
			(0, 0, _) if self.tables == 0 => "blank",
			(words, _, _) if words < 25 && self.tables == 0 => "picture",
			(_, _, largest) if largest >= 90 => "text over photo",
			(words, _, largest)
				if largest >= 40 || (total >= 25 && words < 40) =>
			{
				"picture"
			}
			_ => "text",
		}
	}
}

/// The slides of a deck read into the world.
#[derive(SystemParam)]
struct DeckQuery<'w, 's> {
	tree: RenderTreeQuery<'w, 's>,
	elements: Query<'w, 's, &'static Element>,
	views: ElementQuery<'w, 's>,
	text: DocumentText<'w, 's>,
}

impl DeckQuery<'_, '_> {
	/// Every `article data-card` under `root`, in order.
	fn slides(&self, root: Entity) -> Vec<ReadSlide> {
		self.tree
			.iter_descendants_inclusive_depth_first(root)
			.filter(|entity| {
				self.views.get(*entity).is_ok_and(|view| {
					view.tag() == "article"
						&& view.attribute("data-card").is_some()
				})
			})
			.collect::<Vec<_>>()
			.into_iter()
			.enumerate()
			.map(|(index, article)| self.slide(index as u32, article))
			.collect()
	}

	fn slide(&self, index: u32, article: Entity) -> ReadSlide {
		// the front before the article's first `hr`, the notes after it
		let children = self.tree.children(article).collect::<Vec<_>>();
		let split = children
			.iter()
			.position(|child| self.tag(*child) == Some("hr"))
			.unwrap_or(children.len());
		let (front, notes) = children.split_at(split);
		let notes = notes.get(1..).unwrap_or_default();
		let words = |entities: &[Entity]| -> String {
			entities
				.iter()
				.map(|entity| self.text.text(*entity))
				.collect::<Vec<_>>()
				.join(" ")
		};
		let front_text = words(front);
		let below = front
			.iter()
			.flat_map(|child| {
				self.tree.iter_descendants_inclusive_depth_first(*child)
			})
			.collect::<Vec<_>>();
		let pictures = below
			.iter()
			.filter(|entity| self.tag(**entity) == Some("img"))
			.map(|image| self.picture(*image))
			.collect::<Vec<_>>();
		let title = below
			.iter()
			.find(|entity| self.tag(**entity) == Some("h1"))
			.map(|heading| {
				self.text
					.text(*heading)
					.split_whitespace()
					.collect::<Vec<_>>()
					.join(" ")
			})
			.unwrap_or_default();
		let fingerprint = match front_text.trim().is_empty()
			&& pictures.is_empty()
		{
			true => String::new(),
			false => {
				let mut sources = pictures
					.iter()
					.map(|picture| picture.src.as_str())
					.collect::<Vec<_>>();
				sources.sort();
				format!(
					"{}\n{}",
					front_text.split_whitespace().collect::<Vec<_>>().join(" "),
					sources.join("\n")
				)
			}
		};
		ReadSlide {
			index,
			title,
			hidden: self
				.views
				.get(article)
				.is_ok_and(|view| view.attribute("hidden").is_some()),
			words: front_text.split_whitespace().count(),
			tables: below
				.iter()
				.filter(|entity| self.tag(**entity) == Some("table"))
				.count(),
			notes_words: words(notes).split_whitespace().count(),
			pictures,
			fingerprint,
		}
	}

	fn picture(&self, image: Entity) -> ReadPicture {
		let attribute = |key: &str| {
			self.views
				.get(image)
				.map(|view| view.attribute_string(key))
				.unwrap_or_default()
		};
		let side =
			|key: &str| attribute(key).trim().parse::<f32>().unwrap_or(0.);
		let src = attribute("src");
		let area = side("width") * side("height");
		let slide = (DeckTriage::SLIDE.x * DeckTriage::SLIDE.y) as f32;
		ReadPicture {
			native: Url::parse(&src)
				.and_then(|url| url.media_bytes())
				.ok()
				.and_then(|bytes| raster_ext::size(bytes.bytes())),
			cover: (area * 100. / slide).round().min(100.) as u32,
			src,
		}
	}

	fn tag(&self, entity: Entity) -> Option<&str> {
		self.elements.get(entity).ok().map(Element::tag)
	}
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use beet::prelude::*;

	/// A deck written by beet from cards and read back: a title slide with
	/// notes, a slide carried by a picture shown again on the next, an exact
	/// duplicate and a hidden slide.
	#[beet::test]
	fn triages_a_deck() {
		let png = Url::from_media_bytes(&MediaBytes::new(
			MediaType::Png,
			b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR\0\0\x07\x80\0\0\x04\x38".to_vec(),
		))
		.to_string();
		let picture = format!(
			"<img src=\"{png}\" alt=\"a form\" width=\"800\" height=\"450\">"
		);
		let cards = format!(
			"<article data-card><h1>Field notes</h1><p>A talk</p><hr><p>Welcome everyone to the talk.</p></article>\
			 <article data-card><h1>The form</h1>{picture}</article>\
			 <article data-card><h1>The form again</h1>{picture}</article>\
			 <article data-card><h1>The form again</h1>{picture}</article>\
			 <article data-card hidden><h1>Hidden</h1></article>"
		);
		let deck =
			Fixture::written(&MediaBytes::new_html(cards), MediaType::Pptx);
		let triage = Fixture::read_with(&deck, DeckTriage::of);
		triage
			.slides
			.iter()
			.map(|slide| {
				format!(
					"{} {} {} {} {}",
					slide.slide,
					slide.title,
					slide.words,
					slide.notes_words,
					slide.carried_by
				)
			})
			.collect::<Vec<_>>()
			.xpect_eq(vec![
				"0 Field notes 4 5 picture".to_string(),
				"1 The form 2 0 picture".into(),
				"2 The form again 3 0 picture".into(),
				"3 The form again 3 0 duplicate of slide 2".into(),
				"4 Hidden 1 0 picture".into(),
			]);
		triage.pictures[0].native.as_str().xpect_eq("1920x1080");
		triage.pictures[0].cover.xpect_eq(39);
		triage
			.findings
			.join("\n")
			.xpect_contains("hidden slide(s) 4")
			.xpect_contains("picture 0 is on 3 slides (1-3)")
			.xpect_contains(
				"duplicate slides (say so once, do not transcribe twice): 3==2",
			);
	}
}
