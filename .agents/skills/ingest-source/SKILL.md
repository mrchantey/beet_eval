---
name: ingest-source
description: >
  Bring a source file into a package's assets as markdown: a Word file, a slide deck or a
  PDF dumped mechanically, through `just eval view` for Office files and poppler for a PDF,
  rendered to images where the page carries what text cannot, then written up by hand in the
  house style. Use when a .docx, .pptx or .pdf arrives in a package's assets, when asked to
  "extract", "convert" or "markdown" one, or when an asset has no markdown twin.
---

# ingest-source

Every source becomes markdown in passes: a mechanical dump, which gets the wording, the tables and the notes exactly right; for a deck or a PDF a render of every page, the only thing that shows what a reader saw; then a write-up by hand in the house style, and a spot check. The dump is the authority on wording, the render on layout and highlighting. A package that keeps its own conventions, where a write-up goes, what a source is checked against, what its sources are like, says so in its README or its own copy of this skill, and that wins.

## The tools

| Source | Dump | Render |
|---|---|---|
| `.docx` | `just eval view <path> --accept=text/markdown` | rarely needed: a form's look is in the dump |
| `.pptx` | `just eval view <path> --accept=text/markdown` | `soffice --headless --convert-to pdf`, then `pdftoppm` |
| `.pdf` | `pdfinfo`, `pdftotext -layout`, `pdfimages -list` | `pdftoppm` |

`just eval view` is beet's store view: it parses any file in the workspace store as its media type and renders it as markdown, the path relative to the workspace root. Scratch goes in a directory of the source's own, `/tmp/<slug>/`, never a bare `/tmp/big`: several sources are often converted at once, and a shared name means one conversion silently overwrites or deletes another's file. At the end delete your directory and nothing else.

## The house style

1. Start with `# Title`, an italic line naming the organisation, then a `Source:` paragraph: the original filename, the page or slide count, the author and the created and modified dates (the dump's first paragraph names them), what a typical page looks like, and a sentence saying how anything that carried meaning by its look was rendered. Note where the filename's version or date disagrees with what the source says.
2. Keep the source's own wording, numbering and typos; fix only what the dump broke, a hyphenated line, a lost space, a ligature. A defect kept on purpose gets a clause saying so.
3. Tables are GFM tables, keeping the empty fill-in rows so the shape of what is asked survives. A row-spanning label is repeated or left blank on the rows below; a column-spanning header becomes a bold line above the table; a checkbox grid becomes a `- [ ]` list; a table that is really two side by side is split.
4. A flowchart, pathway or process becomes a Mermaid diagram; a shape Mermaid cannot express, a pyramid or a wheel, a numbered table with a clause naming the shape, never inline SVG, which GitHub strips, and never an image file. A screenshot of a form, spreadsheet, timetable or matrix is transcribed cell by cell, never described. A graph is described in a sentence: series, axis range, shape, the labelled values. A photograph is at most a clause.
5. No extracted image is committed. Read it, transcribe what it says, delete it.
6. No em dashes and no line breaks inside a sentence. That is about prose: an address, a signature block or an email header transcribed in a blockquote keeps a line per line, `>` and all.

## A Word file

7. Dump with `just eval view`. It carries headings by paragraph style; every list as bullets nested by level; every table as GFM headed `<!-- t<n> -->`, the number `just eval ooxml/cells <path>` gives its cells, a merged cell as an empty neighbour and a nested table after its parent; a floating text box's paragraphs after the paragraph anchoring it; an image as `![name](part)`; and what a form signals by look as inline HTML: a coloured run as `<span style="color: #..">`, a highlight as `<mark>`, shading as `<mark style="background: #..">`, bold and italic as markdown.
8. Write the twin from the dump, reading all of it first. A contents page stays a nested list with its page numbers, so a reference to "page 18" still resolves, and the headings take the contents page's numbering even where the heading text lacks it. Red instruction text ("please delete this sentence once completed") goes in *italics inside parentheses*, a highlighted placeholder becomes `` `[Insert business name]` ``, and the Source paragraph says so.
9. The view writes every list as bullets: the real numbering is in `word/numbering.xml` (`unzip -p <file> word/numbering.xml`), and a document's hand-typed oddities, `2B`, `4.b`, a `15` that is really an item of 14, stay as typed with a note.
10. Spot-check against the original: the cells dump's table count against the twin's tables, the `☐` count against the dump's, the section count against the contents page, and every red sentence and placeholder in the dump present in the twin. Images listed by `unzip -l` under `word/media/` are described in place.

## A slide deck

11. Dump with `just eval view`. It carries a header with the slide count, the slide size and the core properties; one `## Slide N` per slide in presentation order, a hidden one marked, with its part and layout, each shape's text in reading order under its kind, nested bullets by level, links, tables (a cell's lines joined by `<br>`), SmartArt read out of its diagram part, the picture inventory (each picture's part, native pixel size and share of the slide) and the speaker notes, an empty `Script:` stub dropped; and a closing report.
12. Render every slide: `soffice -env:UserInstallation=file:///tmp/<slug>/profile --headless --convert-to pdf --outdir /tmp/<slug> <deck>`, the private profile letting several conversions run at once, then `pdftoppm -r 110 -png /tmp/<slug>/<deck>.pdf /tmp/<slug>/slide`. `soffice` is LibreOffice; where it is not installed, say so, and work from the dump and the pictures pulled from the zip, naming in the Source paragraph that no render was read. A hidden slide is dropped from the PDF, so slide n is page n less the hidden slides before it, and `slide-04.png` can be slide 5; the report names the hidden ones. One slide bigger is `pdftoppm -r 300 -f p -l p` off the same PDF, `p` its page by that count, and a picture at native resolution is its part out of the zip, `unzip -p <deck> ppt/media/image3.png > /tmp/<slug>/image3.png`, the size the inventory gives.
13. The report triages every slide by what carries it. Its labels are a floor and not a ceiling: it counts words and measures rectangles, and cannot tell a title slide's stock photo from a screenshot of a form. Judge every slide yourself from the render.
    - **text**: the words carry the slide, the pictures are decoration. **text over photo**: a picture covering the slide under a paragraph of real text. Describe the photo in half a sentence, if at all.
    - **blank**: no words and no pictures, an empty shape tree; say so on its heading.
    - **picture**: under 25 words, or a picture over 40% of the slide. The content is in the image and has to be read out of it. It over-fires on title slides and dividers as often as it under-fires.
    - **duplicate of slide N**: the same text and the same pictures, a claim to verify against the two renders. A deck that re-shows a blank form after its worked examples means it: keep the heading, name it a repeat and point back. A slide repeated with one line changed is not flagged at all; two consecutive headings that read alike are worth comparing anyway.
    - A **recurring picture**, one part down several slides at different coverages, is one screenshot with a different box drawn on it each time: transcribe it once where it first appears, and give the others a heading saying which part is boxed. **Two pictures of the same size** under different parts are often one capture saved twice: compare, transcribe once, cross-reference.
    - A **hidden slide** is in the deck but never shown: transcribe it from the dump and say it is hidden. A **slide part the presentation never shows** is a deleted slide's leftover: mention it in the Source paragraph and transcribe nothing.
14. Read every rendered slide, not only the flagged ones. The dump cannot say that a cell is highlighted, which of two columns a bullet belongs to, or what a picture shows. Where a screenshot is too small, compare its native size with the render: bigger, pull the part; smaller, upscale what you have, `magick <png> -filter Lanczos -resize 300% /tmp/<slug>/big.png`. Cropping usually matters more than resolution: three cards across one strip are legible one at a time, `magick <png> -crop 33%x100% +repage /tmp/<slug>/card.png`, minding that the crop's offset is in pixels even when its size is a percentage.
15. Pull a picture too when pictures overlap, when the render clips one at a slide edge, and whenever a short string sits neatly inside a form field on a screenshot: a deck can fake a filled-in cell with a text box over an empty form, and the picture underneath is the only way to tell. A slide can also be that composite on purpose; read both and describe what the audience saw.
16. Write the deck up. One `##` per slide, numbered as the deck numbers them, with the slide's own title (`## Slide 7: Live Training Schedule`); a slide with two titles takes the running one and adds the heading when that is what tells two slides apart; an untitled slide gets a short one in square brackets; a run of slides sharing one title gets what distinguishes each. Bullets stay bullets in the slide's order. Speaker notes go last on their slide under `### Speaker notes`, as written; where they contradict the slide keep both and say which is which; two slides sharing notes verbatim is said once. A two-column term-and-definition table with no header row becomes a list of bold terms; a table whose cells hold bullet lists becomes lists under bold headings, since GFM cannot nest a list in a cell; a merged header spanning two columns of bullets dumps as a phantom empty column, and the render says the reading order. A deck with no diagram, process or table says so rather than inventing one.
17. Check a screenshot against its source when the package holds it: it settles a cell you could not read and catches a screenshot cropped mid-sentence. Expect the source to have moved, a deck screenshotting an older revision of a form, and say which revision the slide shows. Where a slide and a markdown twin disagree, check the original file before believing either.
18. Spot-check: the `##` count against the slide count in the header, every slide the report flags `picture` carrying a table, a diagram or a real description, a distinctive phrase from each slide of the dump found in the write-up, and numbers transcribed from a screenshot adding up where the sheet shows a total, recomputed at full precision rather than off the sheet's displayed rounding, and agreeing across slides.

### What the deck's mechanical pass gets wrong

- **LibreOffice is not PowerPoint.** Text can overflow a slide that fits in PowerPoint, a heading can render over its own first line, a cropped picture can land in the wrong place. The render owns layout and highlighting; it does not own wording, where the dump wins: a clipped tag reading `Sample` is `Sample Data`, a round badge reading `3 min` is `30 mins`.
- **The render can lose text the dump has**, a body text box dropped whole, so check the shape geometry (`<a:off>` and `<a:ext>` in the slide part) before calling a slide sparse. And it is not the authority on outline depth: it flattens a third-level bullet onto its parent, where the dump's nesting is the deck's own.
- **A table cell can hold several rows of text** joined by line breaks, which the dump keeps as `<br>`: expand them into real rows. A heading and its paragraph can share one text box across a soft break; two sentences fused with no space are that.
- **A two-column slide is often one text box**, and the dump interleaves its columns; the render says which half a line belongs to.
- **The same logo listed twice**, or a full-width brand footer on every slide, is nothing. **A slide reporting zero words and one big picture** is usually one pasted object, often an `.emf` of a form, and is never skippable. **Coverage ignores cropping**: a picture at 100% may be cropped to a third of the slide.
- **A vector picture, `.emf` or `.wmf`, cannot be displayed out of the zip**: the render is its only readable form, so render bigger and crop, and settle a crowded digit by the arithmetic, not another zoom. A 3200 px render is downsampled again on the way in, so crop it into bands too.
- **A generated deck may double its words**, an author like `PptxGenJS` baking line breaks into overlapping runs ("helps you understand understand your rivals"). Confirm it in the dump, give each sentence once, say so in the Source paragraph with one verbatim specimen, and renumber a broken list. A whole sentence repeated under two headings is the deck's own copy-paste and stays.

## A PDF

19. Dump: `pdfinfo <pdf>` for the Source paragraph (Title, Author, Creator, CreationDate, Pages: a Creator of PowerPoint means an exported deck, Word a normal document, a scanner or no text a job for OCR, `tesseract <png> - --psm 6`); `pdftotext -layout <pdf> /tmp/<slug>/text.txt`, pages split by form feeds, the layout mode keeping columns and callouts apart; `pdfimages -list <pdf>` for the images on each page and their sizes.
20. Render every page, `pdftoppm -r 110 -png <pdf> /tmp/<slug>/p`, and read every one, since the text cannot say what an arrow points at, which cell is highlighted or what a graph shows. Too small to read, extract the image at native resolution, `pdfimages -png -f n -l n <pdf> /tmp/<slug>/x`, or render a crop, `pdftoppm -r 300 -f n -l n -x X -y Y -W W -H H -png <pdf> /tmp/<slug>/x`, coordinates in pixels at that resolution.
21. Write it up: one `##` per slide or section, with its title as written, a contents page kept as a nested list with its page numbers. A callout becomes a paragraph or bullets right after the thing it points at, in the order the arrows land; narration along the bottom of a slide follows the transcription as paragraphs. A highlight in a callout becomes **bold**, and the Source paragraph says so. Fix pdftotext's artefacts, soft hyphens and `‐` (U+2010) for `-`, words split across lines, stray column gaps mid-sentence.
22. Spot-check: the `##` count against the pages or the contents page, every page carrying a large image having a table or a description, a distinctive phrase from each page of the dump found in the write-up, and transcribed figures adding up where the page shows a total; where they do not, look at the crop again before deciding the source is wrong.

## Where it goes

23. The write-up sits beside the original in the package's assets, `<name>.md`, unless the package says otherwise; the dump, the renders and every pulled picture are scratch under `/tmp/<slug>/`. Then add the write-up to whatever index the package keeps of its assets.
