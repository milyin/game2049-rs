use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use async_object::{EventStream, Keeper, Tag};
use bindings::Windows::{Foundation::Numerics::Vector2, UI::Composition::ContainerVisual};
use futures::StreamExt;

use crate::FrameTag;

#[derive(Clone)]
pub struct RawEvent<T: Clone + Send + Sync>(pub T);

#[derive(Clone)]
pub struct FocusedEvent<T: Clone + Send + Sync>(pub T);

#[derive(Clone, Debug)]
pub struct Size(pub Vector2);

pub struct Slot {
    container: ContainerVisual,
    focused: bool,
}

impl Slot {
    fn new(frame: FrameTag) -> crate::Result<Self> {
        let container = frame.compositor().CreateContainerVisual()?;
        Ok(Self {
            container,
            focused: false,
        })
    }
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused
    }
    pub fn is_focused(&self) -> bool {
        self.focused
    }
}

pub struct SlotKeeper {
    keeper: Keeper<Slot>,
    container: ContainerVisual,
}

impl SlotKeeper {
    pub fn new(frame: FrameTag) -> crate::Result<Self> {
        let slot = Slot::new(frame)?;
        let container = slot.container.clone();
        let keeper = Keeper::new(slot);
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
    pub fn send_event<T: Clone + Send + Sync + 'static>(&self, event: T, focused: bool) {
        self.keeper.send_event(RawEvent(event.clone()));
        if focused && self.get().is_focused() {
            self.keeper.send_event(FocusedEvent(event));
        }
    }
    pub fn on_size(&self, size: Vector2, focused: bool) -> crate::Result<()> {
        self.container().SetSize(size)?;
        self.send_event(Size(size), focused);
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct SlotTag {
    tag: Tag<Slot>,
    container: ContainerVisual,
}

impl SlotTag {
    pub fn is_focused(&self) -> crate::Result<bool> {
        Ok(self.tag.call(|v| v.is_focused())?)
    }
    pub fn container(&self) -> &ContainerVisual {
        &self.container
    }
    pub async fn join(&self) -> crate::Result<()> {
        let mut stream = EventStream::<()>::new(self.tag.clone());
        while let Some(_) = stream.next().await {}
        Ok(())
    }
    pub fn on_raw_size(&self) -> EventStream<RawEvent<Size>> {
        EventStream::new(self.tag.clone())
    }
    pub fn on_focused_size(&self) -> EventStream<FocusedEvent<Size>> {
        EventStream::new(self.tag.clone())
    }
}
