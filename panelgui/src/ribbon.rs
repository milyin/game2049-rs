use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use async_object::{Keeper, Tag};
use bindings::Windows::Foundation::Numerics::Vector2;
use futures::StreamExt;

use crate::{
    slot::{FocusedEvent, RawEvent, Size},
    FrameTag, SlotKeeper, SlotTag,
};

pub enum Orientation {
    Stack,
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct RibbonRefs {
    frame: FrameTag,
    slot: SlotTag,
}

impl RibbonRefs {
    pub fn new(frame: FrameTag, slot: SlotTag) -> Self {
        Self { frame, slot }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CellLimit {
    pub ratio: f32,
    pub min_size: f32,
    pub max_size: Option<f32>,
}

impl CellLimit {
    pub fn set_size(&mut self, size: f32) {
        self.min_size = size;
        self.max_size = Some(size);
    }
}

impl Default for CellLimit {
    fn default() -> Self {
        Self {
            ratio: 1.,
            min_size: 0.,
            max_size: None,
        }
    }
}

struct Cell {
    slot: SlotKeeper,
    limit: CellLimit,
}

pub struct Ribbon {
    refs: RibbonRefs,
    orientation: Orientation,
    cells: Vec<Cell>,
}

impl Ribbon {
    pub fn new(refs: RibbonRefs, orientation: Orientation) -> Self {
        Self {
            refs,
            orientation,
            cells: Vec::new(),
        }
    }
    pub fn send_event<T: Clone + Send + Sync + 'static>(&self, event: T) -> crate::Result<()> {
        let focused = self.refs.slot.is_focused()?;
        for cell in &self.cells {
            cell.slot.send_event(event.clone(), focused)
        }
        Ok(())
    }
    pub fn send_size_event(&self, event: Size) -> crate::Result<()> {
        let focused = self.refs.slot.is_focused()?;
        for cell in &self.cells {
            cell.slot.send_event(event.clone(), focused)
        }
        Ok(())
    }

    pub fn add_cell(&mut self) -> crate::Result<SlotTag> {
        let slot = SlotKeeper::new(self.refs.frame.clone())?;
        let limit = CellLimit::default();
        let slot_tag = slot.tag();
        self.cells.push(Cell { slot, limit });
        Ok(slot_tag)
    }
}

pub struct RibbonKeeper {
    keeper: Keeper<Ribbon>,
    refs: RibbonRefs,
}

impl RibbonKeeper {
    pub fn new(frame: &FrameTag, slot: SlotTag, orientation: Orientation) -> crate::Result<Self> {
        let refs = RibbonRefs::new(frame.clone(), slot);
        let keeper = Keeper::new(Ribbon::new(refs.clone(), orientation));
        let keeper = Self { keeper, refs };
        Self::spawn_event_handlers(keeper.tag())?;
        Ok(keeper)
    }
    pub fn tag(&self) -> RibbonTag {
        RibbonTag {
            tag: self.keeper.tag(),
            refs: self.refs.clone(),
        }
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Ribbon> {
        self.keeper.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Ribbon> {
        self.keeper.get_mut()
    }
    fn spawn_event_handlers(ribbon_tag: RibbonTag) -> crate::Result<()> {
        ribbon_tag.clone().refs.frame.spawn_local(async move {
            while let Some(RawEvent(size)) = ribbon_tag.refs.slot.on_raw_size().next().await {
                ribbon_tag.send_event(size).await?
            }
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct RibbonTag {
    tag: Tag<Ribbon>,
    refs: RibbonRefs,
}

impl RibbonTag {
    pub async fn send_event<T: Clone + Send + Sync + 'static>(
        &self,
        event: T,
    ) -> crate::Result<()> {
        self.tag.async_call(|v| v.send_event(event)).await?
    }

    pub async fn add_cell(&mut self) -> crate::Result<SlotTag> {
        self.tag.async_call_mut(|v| v.add_cell()).await?
    }
}

impl PartialEq for RibbonTag {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}
