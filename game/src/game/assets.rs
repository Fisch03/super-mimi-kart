use crate::engine::{CreateContext, sprite::Skybox, ui::*};

pub struct SharedAssets {
    pub skybox: Skybox,

    pub game_logo: UiSprite,
    pub item_frame: UiSprite,
}
impl SharedAssets {
    pub fn load(ctx: &CreateContext) -> Self {
        let mut game_logo = UiSprite::load_single(&ctx, "logo.png", UiVec::new(Px(0), Pct(-20.0)));
        game_logo.width = Pct(50.0).into();
        game_logo.local_anchor = Anchor::CENTER;
        game_logo.global_anchor = Anchor::CENTER;

        let mut item_frame =
            UiSprite::load_single(&ctx, "item_frame.png", UiVec::new(Px(0), Px(1)));
        item_frame.local_anchor = Anchor::TOP_CENTER;
        item_frame.global_anchor = Anchor::TOP_CENTER;

        Self {
            skybox: Skybox::load(&ctx, "skybox"),

            game_logo,
            item_frame,
        }
    }
}
