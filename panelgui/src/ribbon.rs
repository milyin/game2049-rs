use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use async_object::{Keeper, Tag};
use bindings::Windows::Foundation::Numerics::{Vector2, Vector3};
use futures::StreamExt;

use crate::{
    slot::{FocusedEvent, RawEvent, Size},
    FrameTag, SlotKeeper, SlotTag,
};

#[derive(PartialEq, Clone, Copy)]
pub enum RibbonOrientation {
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
    pub content_ratio: Vector2,
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
            content_ratio: Vector2::new(1., 1.),
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
    orientation: RibbonOrientation,
    cells: Vec<Cell>,
}

impl Ribbon {
    pub fn new(refs: RibbonRefs, orientation: RibbonOrientation) -> Self {
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
    pub fn on_size(&mut self, size: Vector2) -> crate::Result<()> {
        self.resize_cells(size)?;
        let focused = self.refs.slot.is_focused()?;
        for cell in &self.cells {
            cell.slot.on_size(cell.slot.container().Size()?, focused)?
        }
        Ok(())
    }

    pub fn add_cell(&mut self) -> crate::Result<SlotTag> {
        let slot_keeper = SlotKeeper::new(self.refs.frame.clone())?;
        let limit = CellLimit::default();
        let slot = slot_keeper.tag();
        self.cells.push(Cell {
            slot: slot_keeper,
            limit,
        });
        self.refs
            .slot
            .container()
            .Children()?
            .InsertAtBottom(slot.container().clone())?;
        self.resize_cells(slot.container().Size()?)?;
        Ok(slot)
    }

    fn resize_cells(&mut self, size: Vector2) -> crate::Result<()> {
        if self.orientation == RibbonOrientation::Stack {
            for cell in &self.cells {
                let content_size = size.clone() * cell.limit.content_ratio.clone();
                let content_offset = Vector3 {
                    X: (size.X - content_size.X) / 2.,
                    Y: (size.Y - content_size.Y) / 2.,
                    Z: 0.,
                };
                cell.slot.container().SetSize(&content_size)?;
                cell.slot.container().SetOffset(&content_offset)?;
            }
        } else {
            let limits = self.cells.iter().map(|c| c.limit).collect::<Vec<_>>();
            let hor = self.orientation == RibbonOrientation::Horizontal;
            let target = if hor { size.X } else { size.Y };
            let sizes = adjust_cells(limits, target);
            let mut pos: f32 = 0.;
            for i in 0..self.cells.len() {
                let size = if hor {
                    Vector2 {
                        X: sizes[i],
                        Y: size.Y,
                    }
                } else {
                    Vector2 {
                        X: size.X,
                        Y: sizes[i],
                    }
                };
                let cell = &mut self.cells[i];
                cell.slot.container().SetSize(&size)?;
                cell.slot.container().SetOffset(if hor {
                    Vector3 {
                        X: pos,
                        Y: 0.,
                        Z: 0.,
                    }
                } else {
                    Vector3 {
                        X: 0.,
                        Y: pos,
                        Z: 0.,
                    }
                })?;
                pos += sizes[i];
            }
        }
        Ok(())
    }
}

pub struct RibbonKeeper {
    keeper: Keeper<Ribbon>,
    refs: RibbonRefs,
}

impl RibbonKeeper {
    pub fn new(
        frame: &FrameTag,
        slot: SlotTag,
        orientation: RibbonOrientation,
    ) -> crate::Result<Self> {
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
            while let Some(event) = ribbon_tag.refs.slot.on_raw_size().next().await {
                let RawEvent(Size(size)) = event;
                ribbon_tag.on_size(size)?
            }
            Ok(())
        })
    }
}

fn adjust_cells(limits: Vec<CellLimit>, mut target: f32) -> Vec<f32> {
    //dbg!(&target);
    let mut lock = Vec::with_capacity(limits.len());
    let mut result = Vec::with_capacity(limits.len());
    lock.resize(limits.len(), false);
    result.resize(limits.len(), 0.);

    let mut sum_ratio = limits
        .iter()
        .map(|c| {
            assert!(c.ratio > 0.);
            c.ratio
        })
        .sum::<f32>();
    loop {
        let mut new_target = target;
        let mut all_lock = true;
        for i in 0..limits.len() {
            if !lock[i] {
                let mut share = target * limits[i].ratio / sum_ratio;
                if share <= limits[i].min_size {
                    share = limits[i].min_size;
                    lock[i] = true;
                }
                if let Some(max_size) = limits[i].max_size {
                    if share > max_size {
                        share = max_size;
                        lock[i] = true;
                    }
                }
                if lock[i] {
                    new_target -= share;
                    sum_ratio -= limits[i].ratio;
                    lock[i] = true;
                } else {
                    all_lock = false;
                }
                result[i] = share;
            }
        }
        if all_lock || new_target == target {
            break;
        }
        target = if new_target > 0. { new_target } else { 0. };
    }
    //dbg!(&result);
    result
}

#[derive(Clone)]
pub struct RibbonTag {
    tag: Tag<Ribbon>,
    refs: RibbonRefs,
}

impl RibbonTag {
    pub fn send_event<T: Clone + Send + Sync + 'static>(&self, event: T) -> crate::Result<()> {
        self.tag.call(|v| v.send_event(event))?
    }
    pub fn on_size(&self, size: Vector2) -> crate::Result<()> {
        Ok(self.tag.call_mut(|v| v.on_size(size))??)
    }
    pub fn add_cell(&self) -> crate::Result<SlotTag> {
        self.tag.call_mut(|v| v.add_cell())?
    }
}

impl PartialEq for RibbonTag {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}
