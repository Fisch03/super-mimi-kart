use crate::engine::{
    CreateContext, RenderContext,
    cache::MeshRef,
    mesh::Mesh,
    object::{Object, Transform},
    sprite::{Billboard, SpriteSheet},
};
use common::{ActiveItem, ActiveItemKind, types::*};

#[derive(Debug)]
pub struct Item {
    state: ItemState,
}

#[derive(Debug)]
enum ItemState {
    RedShell { transform: Transform, mesh: MeshRef },
    GreenShell { transform: Transform, mesh: MeshRef },
    Banana { billboard: Billboard },
}

impl Item {
    pub fn preload_assets(ctx: &CreateContext) {
        ctx.assets
            .load_mesh("chorb", || Mesh::load(&ctx, "chorb.glb"));
        ctx.assets
            .load_mesh("chorb_red", || Mesh::load(&ctx, "chorb_red.glb"));

        ctx.assets
            .load_sheet("banana", || SpriteSheet::load_single(&ctx, "yuri.png"));
    }

    pub fn new(ctx: &CreateContext, item: ActiveItem) -> Self {
        let roll = match item.kind {
            ActiveItemKind::RedShell { roll } | ActiveItemKind::GreenShell { roll } => roll,
            ActiveItemKind::Banana => 0.0,
        };

        let mut transform = Transform::new();
        transform.pos = Vec3::new(item.pos.x, -0.2, item.pos.y);
        transform.scale_uniform(0.2);
        transform.rot = Rotation::new(roll, -item.rot + 90.0, 0.0);

        let state = match item.kind {
            ActiveItemKind::RedShell { .. } => ItemState::RedShell {
                mesh: ctx.assets.load_mesh("chorb_red", || {
                    log::warn!("had to reload mesh that shouldve been cached");
                    Mesh::load(&ctx, "chorb_red.glb")
                }),
                transform,
            },

            ActiveItemKind::GreenShell { .. } => ItemState::GreenShell {
                mesh: ctx.assets.load_mesh("chorb", || {
                    log::warn!("had to reload mesh that shouldve been cached");
                    Mesh::load(&ctx, "chorb.glb")
                }),
                transform,
            },

            ActiveItemKind::Banana => {
                let sheet = ctx.assets.load_sheet("banana", || {
                    log::warn!("had to reload sheet that shouldve been cached");
                    SpriteSheet::load_single(&ctx, "yuri.png")
                });

                let mut billboard = Billboard::new(&ctx, "banana", sheet);
                billboard.transform = transform;
                billboard.transform.scale_uniform(0.3);

                ItemState::Banana { billboard }
            }
        };

        Self { state }
    }
}

impl Object for Item {
    fn render(&self, ctx: &RenderContext) {
        match &self.state {
            ItemState::RedShell { mesh, transform } | ItemState::GreenShell { mesh, transform } => {
                mesh.get().render(ctx, &transform);
            }
            ItemState::Banana { billboard } => {
                billboard.render(ctx);
            }
        }
    }
}

impl AsRef<Transform> for Item {
    fn as_ref(&self) -> &Transform {
        match &self.state {
            ItemState::RedShell { transform, .. } | ItemState::GreenShell { transform, .. } => {
                transform
            }
            ItemState::Banana { billboard } => &billboard.transform,
        }
    }
}
