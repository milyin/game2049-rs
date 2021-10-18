use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use async_object::{Keeper, Tag};
use bindings::Windows::UI::Composition::ContainerVisual;

use crate::FrameTag;

pub struct Cell {
    window: FrameTag,
    container: ContainerVisual,
}

impl Cell {
    fn new(window: FrameTag) -> crate::Result<Self> {
        let container = window.compositor().CreateContainerVisual()?;
        Ok(Self { window, container })
    }
}

#[derive(Clone)]
pub struct CellKeeper {
    keeper: Keeper<Cell>,
}

impl CellKeeper {
    pub fn new(window: FrameTag) -> crate::Result<Self> {
        let keeper = Keeper::new(Cell::new(window)?);
        Ok(Self { keeper })
    }
    pub fn tag(&self) -> CellTag {
        CellTag {
            tag: self.keeper.tag(),
        }
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Cell> {
        self.keeper.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Cell> {
        self.keeper.get_mut()
    }
}

#[derive(Clone)]
pub struct CellTag {
    tag: Tag<Cell>,
}
