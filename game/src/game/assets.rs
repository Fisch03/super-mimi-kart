use crate::engine::{
    CreateContext, RenderContext,
    cache::SheetRef,
    mesh::Mesh,
    object::Transform,
    sprite::{Skybox, SpriteSheet},
    ui::*,
};
use common::{ItemKind, types::*};

#[derive(Debug)]
pub struct SharedAssets {
    pub skybox: Skybox,

    game_logo: UiSprite,
    logo_player: UiSprite,

    pub start_button: UiSprite,
    pub credits_button: UiSprite,
    pub settings_button: UiSprite,
    pub back_button: UiSprite,

    pub credits: UiSprite,
    pub controls: UiSprite,
    controls_label: UiSprite,

    pub item_frame: UiSprite,
    pub banana_icon: UiSprite,
    pub shell_background: UiMesh,
    pub red_shell_icon: UiMesh,
    pub green_shell_icon: UiMesh,
    pub boost_icon: UiSprite,
    pub explosion: SheetRef,

    countdown: UiSprite,
    pos_indicator: UiSprite,
    pos_indicator_suffix: UiSprite,
    coin_indicator: UiSprite,
    coin_indicator_prefix: UiSprite,

    pub join_waiting: UiSprite,
    pub load_waiting: UiSprite,
    pub download_waiting: UiSprite,

    cursor: UiSprite,
}

impl SharedAssets {
    pub fn load(ctx: &CreateContext) -> Self {
        let mut game_logo =
            UiSprite::load_single(&ctx, "logo.png", UiVec::new(Pct(-85.0), Pct(-25.0)))
                .anchor(Anchor::CENTER);
        game_logo.width = Pct(50.0).into();
        let logo_player = UiSprite::load_single(
            &ctx,
            "player/player-07.png",
            UiVec::new(Pct(-50.0), Pct(-25.0)),
        )
        .anchor(Anchor::CENTER);

        let item_frame = UiSprite::load_single(&ctx, "item_frame.png", UiVec::new(Px(-1), Px(1)))
            .anchor(Anchor::TOP_RIGHT);
        let banana_icon = UiSprite::load_single(&ctx, "yuri.png", UiVec::new(Px(-9), Px(7)))
            .anchor(Anchor::TOP_RIGHT);

        let shell_outline_mesh = ctx
            .assets
            .load_mesh("chorb", || Mesh::load(&ctx, "chorb.glb"));
        let red_shell_mesh = ctx.assets.load_mesh("chorb_red_nooutline", || {
            Mesh::load(&ctx, "chorb_red_nooutline.glb")
        });
        let green_shell_mesh = ctx.assets.load_mesh("chorb_nooutline", || {
            Mesh::load(&ctx, "chorb_nooutline.glb")
        });
        let shell_background = UiMesh {
            transform: Transform::new().scale(7.2, 7.2, 7.2),
            mesh: shell_outline_mesh,
        };
        let red_shell_icon = UiMesh {
            transform: Transform::new().scale(7.0, 7.0, 7.0),
            mesh: red_shell_mesh,
        };
        let green_shell_icon = UiMesh {
            transform: Transform::new().scale(7.0, 7.0, 7.0),
            mesh: green_shell_mesh,
        };
        let boost_icon = UiSprite::load_single(&ctx, "saber.png", UiVec::new(Px(-9), Px(7)))
            .anchor(Anchor::TOP_RIGHT);

        let countdown = UiSprite::load_multi(&ctx, "pos_indicator", UiVec::new(Px(0), Pct(20.0)))
            .global_anchor(Anchor::TOP_CENTER);

        let pos_indicator = UiSprite::load_multi(&ctx, "pos_indicator", UiVec::new(Px(0), Px(-1)))
            .anchor(Anchor::BOTTOM_RIGHT);
        let pos_indicator_suffix =
            UiSprite::load_multi(&ctx, "pos_suffix", UiVec::new(Px(-3), Px(-3)))
                .anchor(Anchor::BOTTOM_RIGHT);

        let coin_indicator =
            UiSprite::load_multi(&ctx, "coin_indicator", UiVec::new(Px(37), Px(-3)))
                .anchor(Anchor::BOTTOM_LEFT);
        let coin_indicator_prefix =
            UiSprite::load_single(&ctx, "coin_prefix.png", UiVec::new(Px(1), Px(-1)))
                .anchor(Anchor::BOTTOM_LEFT);

        let start_button = UiSprite::load_multi(&ctx, "start_button", UiVec::new(Px(0), Pct(10.0)))
            .anchor(Anchor::CENTER);
        let settings_button =
            UiSprite::load_multi(&ctx, "settings_button", UiVec::new(Px(0), Pct(20.0)))
                .anchor(Anchor::CENTER);
        let credits_button =
            UiSprite::load_multi(&ctx, "credits_button", UiVec::new(Px(0), Pct(30.0)))
                .anchor(Anchor::CENTER);
        let back_button = UiSprite::load_multi(&ctx, "back_button", UiVec::new(Px(0), Px(-8)))
            .anchor(Anchor::BOTTOM_CENTER);

        let mut credits = UiSprite::load_single(&ctx, "credits.png", UiVec::new(Px(0), Px(35)))
            .anchor(Anchor::CENTER);
        credits.width = Ratio(0.5).into();
        let controls = UiSprite::load_single(&ctx, "controls.png", UiVec::new(Px(8), Px(35)))
            .anchor(Anchor::CENTER);
        let controls_label =
            UiSprite::load_multi(&ctx, "controls_label", UiVec::new(Px(0), Px(75)))
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
        cursor.layer = 99;

        let explosion = ctx
            .assets
            .load_sheet("explosion", || SpriteSheet::load_multi(&ctx, "explosion"));

        Self {
            skybox: Skybox::load(&ctx, "skybox"),

            game_logo,
            logo_player,

            banana_icon,
            shell_background,
            red_shell_icon,
            green_shell_icon,
            boost_icon,
            explosion,

            start_button,
            credits_button,
            settings_button,
            back_button,

            credits,
            controls,
            controls_label,

            item_frame,

            countdown,
            pos_indicator,
            pos_indicator_suffix,
            coin_indicator,
            coin_indicator_prefix,

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

    pub fn render_logo(&mut self, ctx: &RenderContext) -> bool {
        let UiDim::Percent(player_x) = self.logo_player.pos.x else {
            unreachable!();
        };
        let UiDim::Percent(logo_x) = self.game_logo.pos.x else {
            unreachable!();
        };

        self.logo_player.pos.x = UiDim::Percent(player_x + ctx.dt * 50.0);
        if logo_x < 0.0 {
            self.game_logo.pos.x = UiDim::Percent(logo_x + ctx.dt * 50.0);
        }
        self.game_logo.pos.y = UiDim::Percent(-25.0 + (ctx.time() * 0.002).sin() as f32 * 2.0);

        // let diff = logo_x.abs();
        // self.game_logo.rot = diff * 0.75;

        self.logo_player.render(ctx);
        self.game_logo.render(ctx);

        logo_x >= 0.0
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

        self.settings_button.sheet.get_mut().active_sprite =
            if self.settings_button.hovered(ctx.viewport, ctx.mouse_pos) {
                1
            } else {
                0
            };
        self.settings_button.render(ctx);
    }

    pub fn render_back(&mut self, ctx: &RenderContext) {
        self.back_button.sheet.get_mut().active_sprite =
            if self.back_button.hovered(ctx.viewport, ctx.mouse_pos) {
                1
            } else {
                0
            };
        self.back_button.render(ctx);
    }

    pub fn render_controls(&mut self, ctx: &RenderContext, swap: bool) {
        self.controls_label.sheet.get_mut().active_sprite = if swap { 1 } else { 0 };
        self.controls_label.render(ctx);
        self.controls.render(ctx);
    }

    pub fn render_countdown(&mut self, ctx: &RenderContext, countdown: u32) {
        self.countdown.sheet.get_mut().active_sprite = countdown;
        self.countdown.render(ctx);
    }

    pub fn render_coin_count(&mut self, ctx: &RenderContext, coins: u32) {
        self.coin_indicator_prefix.render(ctx);
        self.coin_indicator.sheet.get_mut().active_sprite = coins.min(10);
        self.coin_indicator.render(ctx);
    }

    pub fn render_item(&mut self, ctx: &RenderContext, kind: ItemKind) {
        match kind {
            ItemKind::Banana => self.banana_icon.render(ctx),
            ItemKind::RedShell => {
                self.red_shell_icon.transform.pos = Vec3::new(ctx.viewport.x - 27.0, 25.0, 0.0);
                self.red_shell_icon.transform.rot.y += ctx.dt * 90.0;
                self.red_shell_icon.transform.rot.z += ctx.dt * 110.0;

                self.shell_background.transform.pos = self.red_shell_icon.transform.pos;
                self.shell_background.transform.rot = self.red_shell_icon.transform.rot;
                self.shell_background.render(ctx);
                self.red_shell_icon.render(ctx);
            }

            ItemKind::GreenShell => {
                self.green_shell_icon.transform.pos = Vec3::new(ctx.viewport.x - 27.0, 25.0, 0.0);
                self.green_shell_icon.transform.rot.y += ctx.dt * 90.0;
                self.green_shell_icon.transform.rot.z += ctx.dt * 110.0;
                self.shell_background.transform.pos = self.green_shell_icon.transform.pos;
                self.shell_background.transform.rot = self.green_shell_icon.transform.rot;
                self.shell_background.render(ctx);
                self.green_shell_icon.render(ctx);
            }
            ItemKind::Boost => self.boost_icon.render(ctx),
        }
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
        let x_diff = self.pos_indicator.sheet.get().sprite_dimensions().x as i32 * 2;

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
