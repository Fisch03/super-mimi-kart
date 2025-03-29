use crate::engine::{CreateContext, RenderContext, sprite::Skybox, ui::*};

#[derive(Debug)]
pub struct SharedAssets {
    pub skybox: Skybox,

    pub game_logo: UiSprite,
    pub item_frame: UiSprite,

    pub countdown: UiSprite,
    pub pos_indicator: UiSprite,
    pub pos_indicator_suffix: UiSprite,

    pub join_waiting: UiSprite,
    pub load_waiting: UiSprite,
}
impl SharedAssets {
    pub fn load(ctx: &CreateContext) -> Self {
        let mut game_logo = UiSprite::load_single(&ctx, "logo.png", UiVec::new(Px(0), Pct(-20.0)))
            .anchor(Anchor::CENTER);
        game_logo.width = Pct(50.0).into();

        let item_frame = UiSprite::load_single(&ctx, "item_frame.png", UiVec::new(Px(-1), Px(1)))
            .anchor(Anchor::TOP_RIGHT);

        let countdown = UiSprite::load_multi(&ctx, "pos_indicator", UiVec::new(Px(0), Pct(20.0)))
            .global_anchor(Anchor::TOP_CENTER);

        let pos_indicator_suffix =
            UiSprite::load_multi(&ctx, "pos_suffix", UiVec::new(Px(-3), Px(-3)))
                .anchor(Anchor::BOTTOM_RIGHT);

        let pos_indicator = UiSprite::load_multi(&ctx, "pos_indicator", UiVec::new(Px(0), Px(-1)))
            .anchor(Anchor::BOTTOM_RIGHT);

        let mut join_waiting =
            UiSprite::load_single(&ctx, "join_wait.png", UiVec::new(Px(0), Pct(20.0)))
                .anchor(Anchor::CENTER);
        join_waiting.width = Ratio(0.5).into();
        let mut load_waiting =
            UiSprite::load_single(&ctx, "load_wait.png", UiVec::new(Px(0), Pct(20.0)))
                .anchor(Anchor::CENTER);
        load_waiting.width = Ratio(0.5).into();

        Self {
            skybox: Skybox::load(&ctx, "skybox"),

            game_logo,
            item_frame,

            countdown,
            pos_indicator,
            pos_indicator_suffix,

            join_waiting,
            load_waiting,
        }
    }

    pub fn render_countdown(&mut self, ctx: &RenderContext, countdown: u32) {
        self.countdown.sheet.get_mut().active_sprite = countdown;
        self.countdown.render(ctx);
    }

    pub fn render_pos(&mut self, ctx: &RenderContext, mut pos: u32) {
        let mut x = -(self.pos_indicator_suffix.sheet.get().sprite_dimensions().x as i32) * 2 - 2;
        let x_diff = self.pos_indicator.sheet.get().sprite_dimensions().x as i32;

        let suffix = match pos % 10 {
            1 if pos != 11 => 0,
            2 if pos != 12 => 1,
            3 if pos != 13 => 2,
            _ => 3,
        };
        self.pos_indicator_suffix.sheet.get_mut().active_sprite = suffix;
        self.pos_indicator_suffix.render(ctx);

        loop {
            let digit = pos % 10;
            pos /= 10;

            self.pos_indicator.sheet.get_mut().active_sprite = digit;
            self.pos_indicator.pos.x = Px(x).into();
            self.pos_indicator.render(ctx);

            x -= x_diff;

            if pos == 0 {
                break;
            }
        }
    }
}
