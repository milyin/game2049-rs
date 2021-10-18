use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use async_object::{EventStream, Keeper, Tag};
use bindings::Windows::{
    Foundation::Numerics::Vector2,
    UI::Composition::{ContainerVisual, Visual},
};

use crate::FrameTag;

#[derive(Clone)]
pub struct Size(Vector2);

impl Size {
    pub fn new(size: Vector2) -> Self {
        Self(size)
    }
}

impl AsRef<Vector2> for Size {
    fn as_ref(&self) -> &Vector2 {
        &self.0
    }
}

pub struct Slot {
    frame: FrameTag,
    container: ContainerVisual,
}

impl Slot {
    fn new(frame: FrameTag) -> crate::Result<Self> {
        let container = frame.compositor().CreateContainerVisual()?;
        Ok(Self { frame, container })
    }
}

pub struct SlotKeeper {
    keeper: Keeper<Slot>,
    container: ContainerVisual,
}

impl SlotKeeper {
    pub fn new(frame: FrameTag) -> crate::Result<Self> {
        let panel = Slot::new(frame)?;
        let container = panel.container.clone();
        let keeper = Keeper::new(panel);
        Ok(Self { keeper, container })
    }
    pub fn tag(&self) -> SlotTag {
        SlotTag {
            tag: self.keeper.tag(),
            container: self.container.clone(),
        }
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Slot> {
        self.keeper.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Slot> {
        self.keeper.get_mut()
    }
    pub fn container(&self) -> &ContainerVisual {
        &self.container
    }
    pub fn send_size(&self, size: Size) {
        self.keeper.send_event(size)
    }
}

#[derive(Clone, PartialEq)]
pub struct SlotTag {
    tag: Tag<Slot>,
    container: ContainerVisual,
}

impl SlotTag {
    pub fn container(&self) -> &ContainerVisual {
        &self.container
    }
    pub fn alive(&self) -> EventStream<()> {
        EventStream::new(self.tag.clone())
    }
    pub fn on_size(&self) -> EventStream<Size> {
        EventStream::new(self.tag.clone())
    }
}
