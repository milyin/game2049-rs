use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_object::{EventStream, Keeper, Tag};
use bindings::Windows::UI::Composition::ContainerVisual;
use futures::StreamExt;

use crate::SizeEvent;

#[derive(Clone)]
pub struct RawEvent<T: Clone + Send + Sync>(pub T);

#[derive(Clone)]
pub struct FocusedEvent<T: Clone + Send + Sync>(pub T);

#[derive(Clone, Debug)]

pub struct Slot {
    container: ContainerVisual,
    focused: bool,
}

impl Slot {
    fn new(container: ContainerVisual) -> crate::Result<Self> {
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
    pub fn container(&self) -> ContainerVisual {
        self.container.clone()
    }
}

pub struct SlotKeeper(Keeper<Slot, ContainerVisual>);

impl SlotKeeper {
    pub fn new(container: ContainerVisual) -> crate::Result<Self> {
        let slot = Slot::new(container.clone())?;
        Ok(Self(Keeper::new_with_shared(
            slot,
            Arc::new(RwLock::new(container)),
        )))
    }
    pub fn tag(&self) -> SlotTag {
        SlotTag(self.0.tag())
    }
    pub fn container(&self) -> crate::Result<ContainerVisual> {
        Ok(self.0.clone_shared())
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Slot> {
        self.0.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Slot> {
        self.0.get_mut()
    }
    pub fn send_event<T: Clone + Send + Sync + 'static>(&self, event: T, parent_focused: bool) {
        self.0.send_event(RawEvent(event.clone()));
        if parent_focused && self.get().is_focused() {
            self.0.send_event(FocusedEvent(event));
        }
    }
    pub fn on_size(&self, parent_focused: bool) -> crate::Result<()> {
        self.send_event(SizeEvent, parent_focused);
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub struct SlotTag(Tag<Slot, ContainerVisual>);

impl SlotTag {
    pub fn is_focused(&self) -> crate::Result<bool> {
        Ok(self.0.call(|v| v.is_focused())?)
    }
    pub async fn wait_for_destroy(&self) -> crate::Result<()> {
        let mut stream = EventStream::<()>::new(self.0.clone());
        while let Some(_) = stream.next().await {}
        Ok(())
    }
    pub fn on_raw_size(&self) -> EventStream<RawEvent<SizeEvent>> {
        EventStream::new(self.0.clone())
    }
    pub fn on_focused_size(&self) -> EventStream<FocusedEvent<SizeEvent>> {
        EventStream::new(self.0.clone())
    }
    pub fn container(&self) -> crate::Result<ContainerVisual> {
        Ok(self.0.clone_shared()?)
    }
}
