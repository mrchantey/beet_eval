//! Reading a suite's sources into markdown, the `ingest-source` skill's
//! mechanical half: beet's store view dumps a source's words, and
//! [`DeckTriage`] sorts a deck's slides by what carries them, so the reader
//! knows which pictures must be read out by hand.
mod deck_triage;
pub use deck_triage::*;
