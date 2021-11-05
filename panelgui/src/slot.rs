use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_object::{EventStream, Keeper, Tag};
use bindings::Windows::UI::Composition::{ContainerVisual, Visual};
use futures::StreamExt;

use crate::slot_event::{ReceiveSlotEvent, SendSlotEvent, SlotSize};

#[derive(Clone)]
pub struct Slot {
    tag: SlotTag,
    container: ContainerVisual,
    focused: bool,
}

impl Slot {
    fn new(container: ContainerVisual) -> crate::Result<Self> {
        Ok(Self {
            tag: SlotTag::default(),
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
    pub fn plug(&mut self, visual: Visual) -> crate::Result<SlotPlug> {
        visual.SetSize(self.container.Size()?)?;
        self.container.Children()?.InsertAtTop(visual.clone())?;
        Ok(SlotPlug {
            tag: self.tag.clone(),
            container: self.container.clone(),
            visual,
        })
    }
}

pub struct SlotPlug {
    tag: SlotTag,
    container: ContainerVisual,
    visual: Visual,
}

impl SlotPlug {
    pub fn tag(&self) -> SlotTag {
        self.tag.clone()
    }
}

impl From<SlotPlug> for SlotTag {
    fn from(plug: SlotPlug) -> Self {
        plug.tag()
    }
}

impl Drop for SlotPlug {
    fn drop(&mut self) {
        let _ = self.container.Children().map(|c| c.Remove(&self.visual));
    }
}

pub struct SlotKeeper(Keeper<Slot, ContainerVisual>);

impl SlotKeeper {
    pub fn new(container: ContainerVisual) -> crate::Result<Self> {
        let slot = Slot::new(container.clone())?;
        let keeper = Self(Keeper::new_with_shared(
            slot,
            Arc::new(RwLock::new(container)),
        ));
        keeper.get_mut().tag = keeper.tag();
        Ok(keeper)
    }
    pub fn tag(&self) -> SlotTag {
        SlotTag(self.0.tag())
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Slot> {
        self.0.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Slot> {
        self.0.get_mut()
    }
    pub fn container(&self) -> crate::Result<ContainerVisual> {
        Ok(self.0.clone_shared())
    }
}

impl SendSlotEvent for SlotKeeper {
    fn send_size(&self, size: SlotSize) -> crate::Result<()> {
        self.container()?.SetSize(size.0)?;
        self.0.send_event(size);
        Ok(())
    }
}

#[derive(Clone, PartialEq, Default)]
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
    pub fn plug(&self, visual: Visual) -> crate::Result<SlotPlug> {
        Ok(self.0.call_mut(|v| v.plug(visual))??)
    }
}

impl ReceiveSlotEvent for SlotTag {
    fn on_size(&self) -> EventStream<SlotSize> {
        EventStream::new(self.0.clone())
    }
}
