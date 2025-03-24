use crate::engine::{
    CreateContext, RenderContext, UpdateContext,
    cache::MeshRef,
    mesh::Mesh,
    object::{Object, Transform},
};
use common::{ActiveItem, ActiveItemKind, types::*};

#[derive(Debug)]
pub struct Item {
    transform: Transform,
    state: ItemState,
}

#[derive(Debug)]
enum ItemState {
    RedShell { mesh: MeshRef },
    GreenShell { mesh: MeshRef },
    Banana { mesh: MeshRef },
}

impl Item {
    pub fn preload_assets(ctx: &CreateContext) {
        ctx.assets
            .load_mesh("chorb", || Mesh::load(&ctx, "chorb.glb"));
    }

    pub fn new(ctx: &CreateContext, item: ActiveItem) -> Self {
        let state = match item.kind {
            ActiveItemKind::RedShell { .. } => ItemState::RedShell {
                mesh: ctx.assets.get_mesh("chorb").unwrap(),
            },

            ActiveItemKind::GreenShell { .. } => ItemState::GreenShell {
                mesh: ctx.assets.get_mesh("chorb").unwrap(),
            },

            ActiveItemKind::Banana => ItemState::Banana {
                mesh: ctx.assets.get_mesh("chorb").unwrap(),
            },
        };

        let roll = match item.kind {
            ActiveItemKind::RedShell { roll } | ActiveItemKind::GreenShell { roll } => roll,
            ActiveItemKind::Banana => 0.0,
        };

        let mut transform = Transform::new();
        transform.pos = Vec3::new(item.pos.x, -0.2, item.pos.y);
        transform.scale_uniform(0.2);
        transform.rot = Rotation::new(roll, -item.rot + 90.0, 0.0);

        Self { transform, state }
    }
}

impl Object for Item {
    fn update(&mut self, _ctx: &mut UpdateContext) {}

    fn render(&self, ctx: &RenderContext) {
        match &self.state {
            ItemState::RedShell { mesh }
            | ItemState::GreenShell { mesh }
            | ItemState::Banana { mesh } => mesh.get().render(ctx, &self.transform),
        }
    }
}

impl AsRef<Transform> for Item {
    fn as_ref(&self) -> &Transform {
        &self.transform
    }
}
