use std::{
    default,
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use async_object::{Keeper, Tag};
use bindings::Windows::Foundation::Numerics::Vector2;
use futures::{task::LocalSpawnExt, StreamExt};

use crate::{CellKeeper, CellTag, FrameKeeper, FrameTag};

pub enum Orientation {
    Stack,
    Horizontal,
    Vertical,
}

#[derive(Clone)]
struct CellHolder {
    cell: CellKeeper,
    ratio: f32,
}

impl CellHolder {
    fn new(window: FrameTag) -> crate::Result<Self> {
        Ok(Self {
            cell: CellKeeper::new(window)?,
            ratio: 1.0,
        })
    }
}

pub struct Ribbon {
    window: FrameTag,
    orientation: Orientation,
    cells: Vec<CellHolder>,
}

impl Ribbon {
    fn new(window: FrameTag) -> crate::Result<Self> {
        Ok(Self {
            window,
            orientation: Orientation::Horizontal,
            cells: Vec::new(),
        })
    }
    pub fn set_size(size: Vector2) -> crate::Result<()> {
        
    }
    pub fn set_sell_count(&mut self, count: usize) -> crate::Result<()> {
        self.cells
            .resize(count, CellHolder::new(self.window.clone())?);
        Ok(())
    }
    pub fn set_cell_ratio(&mut self, index: usize, ratio: f32) -> crate::Result<()> {
        self.cells
            .get_mut(index)
            .map_or(Err(crate::Error::BadIndex), |cell| {
                cell.ratio = ratio;
                Ok(())
            })
    }
    pub fn get_cell(&self, index: usize) -> crate::Result<CellTag> {
        self.cells
            .get(index)
            .map_or(Err(crate::Error::BadIndex), |cell| Ok(cell.cell.tag()))
    }
}

#[derive(Clone)]
pub struct RibbonKeeper {
    keeper: Keeper<Ribbon>,
}

impl RibbonKeeper {
    pub fn new(window: FrameTag) -> crate::Result<Self> {
        let keeper = Keeper::new(Ribbon::new(window)?);
        let keeper = Self { keeper };
        let ribbon = keeper.tag();

        let spawner = window.spawner().clone();
        spawner.spawn_local({
            let window = window.clone();
            let ribbon = ribbon.clone();
            async { while let Some(size) = window.on_size().next().await {} }
        })?;

        Ok(keeper)
    }
    pub fn tag(&self) -> RibbonTag {
        RibbonTag {
            tag: self.keeper.tag(),
        }
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Ribbon> {
        self.keeper.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Ribbon> {
        self.keeper.get_mut()
    }
}

#[derive(Clone)]
pub struct RibbonTag {
    tag: Tag<Ribbon>,
}

impl RibbonTag {
    pub async fn set_sell_count(&mut self, count: usize) -> crate::Result<()> {
        self.tag.async_call_mut(|v| v.set_sell_count(count)).await?
    }
    pub async fn set_cell_ratio(&mut self, index: usize, ratio: f32) -> crate::Result<()> {
        self.tag
            .async_call_mut(|v| v.set_cell_ratio(index, ratio))
            .await?
    }
    pub async fn get_cell(&self, index: usize) -> crate::Result<CellTag> {
        self.tag.async_call(|v| v.get_cell(index)).await?
    }
}
