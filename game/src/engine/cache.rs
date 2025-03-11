use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::engine::{mesh::Mesh, sprite::SpriteSheet};

#[derive(Debug, Clone)]
pub struct SheetRef(Rc<RefCell<SpriteSheet>>);

impl SheetRef {
    pub fn get(&self) -> Ref<SpriteSheet> {
        self.0.borrow()
    }
    pub fn get_mut(&self) -> RefMut<SpriteSheet> {
        self.0.borrow_mut()
    }
}

#[derive(Debug, Clone)]
pub struct MeshRef(Rc<RefCell<Mesh>>);

impl MeshRef {
    pub fn get(&self) -> Ref<Mesh> {
        self.0.borrow()
    }
    pub fn get_mut(&self) -> RefMut<Mesh> {
        self.0.borrow_mut()
    }
}

#[derive(Debug, Default)]
pub struct AssetCache {
    sprites: RefCell<HashMap<String, SheetRef>>,
    meshes: RefCell<HashMap<String, MeshRef>>,
}

impl AssetCache {
    pub fn load_sheet<F: FnOnce() -> SpriteSheet>(&self, name: &str, loader: F) -> SheetRef {
        if let Some(sheet) = self.sprites.borrow().get(name) {
            sheet.clone()
        } else {
            let sheet = loader();
            let sheet_ref = SheetRef(Rc::new(RefCell::new(sheet)));

            self.sprites
                .borrow_mut()
                .insert(name.to_string(), sheet_ref.clone());

            sheet_ref
        }
    }

    pub fn load_mesh<F: FnOnce() -> Mesh>(&self, name: &str, loader: F) -> MeshRef {
        if let Some(mesh) = self.meshes.borrow().get(name) {
            mesh.clone()
        } else {
            let mesh = loader();
            let mesh_ref = MeshRef(Rc::new(RefCell::new(mesh)));

            self.meshes
                .borrow_mut()
                .insert(name.to_string(), mesh_ref.clone());

            mesh_ref
        }
    }

    pub fn clear(&self) {
        self.sprites.borrow_mut().clear();
        self.meshes.borrow_mut().clear();
    }
}
