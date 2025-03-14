use include_dir::{Dir, include_dir};

static SPRITE_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../assets");

mod sheet;
pub use sheet::{SpriteSheet, SpriteSheetUniforms};

mod billboard;
pub use billboard::Billboard;
