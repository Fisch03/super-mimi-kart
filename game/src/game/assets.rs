use crate::engine::{CreateContext, RenderContext, sprite::Skybox, ui::*};

#[derive(Debug)]
pub struct SharedAssets {
    pub skybox: Skybox,

    pub game_logo: UiSprite,
    pub start_button: UiSprite,
    pub credits_button: UiSprite,
    pub credits: UiSprite,

    pub item_frame: UiSprite,

    pub countdown: UiSprite,
    pub pos_indicator: UiSprite,
    pub pos_indicator_suffix: UiSprite,

    pub join_waiting: UiSprite,
    pub load_waiting: UiSprite,
    pub download_waiting: UiSprite,

    pub cursor: UiSprite,
}
impl SharedAssets {
    pub fn load(ctx: &CreateContext) -> Self {
        let mut game_logo = UiSprite::load_single(&ctx, "logo.png", UiVec::new(Px(0), Pct(-25.0)))
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

        let start_button = UiSprite::load_multi(&ctx, "start_button", UiVec::new(Px(0), Pct(10.0)))
            .anchor(Anchor::CENTER);
        let credits_button =
            UiSprite::load_multi(&ctx, "credits_button", UiVec::new(Px(0), Pct(20.0)))
                .anchor(Anchor::CENTER);

        let credits = UiSprite::load_single(&ctx, "credits.png", UiVec::new(Px(0), Pct(20.0)))
            .anchor(Anchor::CENTER);

        let mut join_waiting =
            UiSprite::load_single(&ctx, "join_wait.png", UiVec::new(Px(0), Pct(20.0)))
                .anchor(Anchor::CENTER);
        join_waiting.width = Ratio(0.5).into();
        let mut download_waiting =
            UiSprite::load_single(&ctx, "download_wait.png", UiVec::new(Px(0), Pct(20.0)))
                .anchor(Anchor::CENTER);
        download_waiting.width = Ratio(0.5).into();
        let mut load_waiting =
            UiSprite::load_single(&ctx, "load_wait.png", UiVec::new(Px(0), Pct(20.0)))
                .anchor(Anchor::CENTER);
        load_waiting.width = Ratio(0.5).into();

        let mut cursor = UiSprite::load_single(&ctx, "cursor.png", UiVec::new(Px(0), Px(0)))
            .anchor(Anchor::TOP_LEFT);
        cursor.width = Ratio(0.5).into();

        Self {
            skybox: Skybox::load(&ctx, "skybox"),

            game_logo,

            start_button,
            credits_button,
            credits,

            item_frame,

            countdown,
            pos_indicator,
            pos_indicator_suffix,

            join_waiting,
            load_waiting,
            download_waiting,

            cursor,
        }
    }

    pub fn render_cursor(&mut self, ctx: &RenderContext) {
        self.cursor.pos = UiVec::new(Px(ctx.mouse_pos.x as i32), Px(ctx.mouse_pos.y as i32));
        self.cursor.render(ctx);
    }

    pub fn render_menu(&mut self, ctx: &RenderContext) {
        self.start_button.sheet.get_mut().active_sprite =
            if self.start_button.hovered(ctx.viewport, ctx.mouse_pos) {
                1
            } else {
                0
            };
        self.start_button.render(ctx);

        self.credits_button.sheet.get_mut().active_sprite =
            if self.credits_button.hovered(ctx.viewport, ctx.mouse_pos) {
                1
            } else {
                0
            };
        self.credits_button.render(ctx);
    }

    pub fn render_countdown(&mut self, ctx: &RenderContext, countdown: u32) {
        self.countdown.sheet.get_mut().active_sprite = countdown;
        self.countdown.render(ctx);
    }

    pub fn render_pos_centered(&mut self, ctx: &RenderContext, place: u32) {
        self.pos_indicator.set_anchor(Anchor::CENTER);
        self.pos_indicator_suffix.set_anchor(Anchor::CENTER);

        let indicator_dim = self.pos_indicator.sheet.get().sprite_dimensions();
        let suffix_dim = self.pos_indicator_suffix.sheet.get().sprite_dimensions();

        self.pos_indicator_suffix.pos =
            UiVec::new(Px(suffix_dim.x as i32), Px(indicator_dim.y as i32 / 2));

        let x = -(indicator_dim.x as i32);

        self.render_pos_inner(ctx, place, x);
    }

    pub fn render_pos(&mut self, ctx: &RenderContext, place: u32) {
        self.pos_indicator.set_anchor(Anchor::BOTTOM_RIGHT);
        self.pos_indicator_suffix.set_anchor(Anchor::BOTTOM_RIGHT);

        // let indicator_dim = self.pos_indicator.sheet.get().sprite_dimensions();
        let suffix_dim = self.pos_indicator_suffix.sheet.get().sprite_dimensions();

        self.pos_indicator_suffix.pos = UiVec::new(Px(-3), Px(-3));
        let x = -(suffix_dim.x as i32) * 2 - 2;

        self.render_pos_inner(ctx, place, x);
    }

    fn render_pos_inner(&mut self, ctx: &RenderContext, mut place: u32, mut x: i32) {
        // let mut x = -(self.pos_indicator_suffix.sheet.get().sprite_dimensions().x as i32) * 2 - 2;
        let x_diff = self.pos_indicator.sheet.get().sprite_dimensions().x as i32;

        let suffix = match place % 10 {
            1 if place != 11 => 0,
            2 if place != 12 => 1,
            3 if place != 13 => 2,
            _ => 3,
        };
        self.pos_indicator_suffix.sheet.get_mut().active_sprite = suffix;
        self.pos_indicator_suffix.render(ctx);

        loop {
            let digit = place % 10;
            place /= 10;

            self.pos_indicator.sheet.get_mut().active_sprite = digit;
            self.pos_indicator.pos.x = Px(x).into();
            self.pos_indicator.render(ctx);

            x -= x_diff;

            if place == 0 {
                break;
            }
        }
    }
}
