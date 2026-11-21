use crate::{Result, bundle::Manifest};
use std::path::Path;

pub(crate) fn read_fixture(root: &Path) -> Result<(Manifest, Vec<u8>, Vec<u8>)> {
    let (manifest, game) = Manifest::read_game(root)?;
    let handler = manifest.payload(root, "retained_mod.bc")?;
    Ok((manifest, game, handler))
}
