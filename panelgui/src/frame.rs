use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_object::{Keeper, Tag};
use bindings::Windows::UI::Composition::{Compositor, ContainerVisual};
use futures::{executor::LocalSpawner, task::LocalSpawnExt, Future};

use crate::{
    slot::{SlotKeeper, SlotTag},
    SizeEvent,
};

pub struct FrameShared {
    spawner: LocalSpawner,
    compositor: Compositor,
    frame_visual: ContainerVisual,
}
pub struct Frame {
    shared: Arc<RwLock<FrameShared>>,
    slots: Vec<SlotKeeper>,
}

impl Frame {
    fn new(spawner: LocalSpawner) -> crate::Result<Self> {
        let compositor = Compositor::new()?;
        let frame_visual = compositor.CreateContainerVisual()?;
        let shared = Arc::new(RwLock::new(FrameShared {
            spawner,
            compositor,
            frame_visual,
        }));
        Ok(Self {
            shared,
            slots: Vec::new(),
        })
    }
    fn shared(&self) -> Arc<RwLock<FrameShared>> {
        self.shared.clone()
    }
    fn compositor(&self) -> Compositor {
        self.shared.read().unwrap().compositor.clone()
    }
    fn frame_visual(&self) -> ContainerVisual {
        self.shared.read().unwrap().frame_visual.clone()
    }
    fn on_size(&mut self) -> crate::Result<()> {
        let size = self.frame_visual().Size()?;
        for slot in &self.slots {
            slot.container()?.SetSize(size)?;
            slot.send_event(SizeEvent, true); // true because frame itself is always focused
        }
        Ok(())
    }

    fn open_slot(&mut self) -> crate::Result<SlotTag> {
        let compositor = self.compositor();
        let container = compositor.CreateContainerVisual()?;
        self.shared
            .read()
            .unwrap()
            .frame_visual
            .Children()?
            .InsertAtTop(container.clone())?;
        let slot_keeper = SlotKeeper::new(container)?;
        let slot = slot_keeper.tag();
        if let Some(top) = self.slots.last_mut() {
            top.get_mut().set_focused(false);
        }
        slot_keeper.get_mut().set_focused(true);
        slot_keeper.on_size(true)?; // true because frame itself is always focused
        self.slots.push(slot_keeper);
        Ok(slot)
    }

    pub fn close_slot(&mut self, slot: SlotTag) -> crate::Result<()> {
        self.shared
            .read()
            .unwrap()
            .frame_visual
            .Children()?
            .Remove(slot.container()?)?;
        if let Some(index) = self.slots.iter().position(|v| v.tag() == slot) {
            self.slots.remove(index);
        }
        if let Some(top) = self.slots.last_mut() {
            top.get_mut().set_focused(true);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct FrameKeeper(Keeper<Frame, FrameShared>);

impl FrameKeeper {
    pub fn new(spawner: LocalSpawner) -> crate::Result<Self> {
        let frame = Frame::new(spawner)?;
        let shared = frame.shared();
        let keeper = Keeper::new_with_shared(frame, shared);
        Ok(Self(keeper))
    }
    pub fn tag(&self) -> FrameTag {
        FrameTag(self.0.tag())
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Frame> {
        self.0.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Frame> {
        self.0.get_mut()
    }
}

#[derive(Clone, PartialEq)]
pub struct FrameTag(Tag<Frame, FrameShared>);

impl FrameTag {
    pub fn compositor(&self) -> crate::Result<Compositor> {
        Ok(self.0.read_shared(|v| v.compositor.clone())?)
    }
    pub fn frame_visual(&self) -> crate::Result<ContainerVisual> {
        Ok(self.0.read_shared(|v| v.frame_visual.clone())?)
    }
    pub fn spawner(&self) -> crate::Result<LocalSpawner> {
        Ok(self.0.read_shared(|v| v.spawner.clone())?)
    }
    pub fn spawn_local<Fut>(&self, future: Fut) -> crate::Result<()>
    where
        Fut: Future<Output = crate::Result<()>> + 'static,
    {
        self.spawner()?.spawn_local(async {
            future.await.unwrap() // TODO: store error somethere (thread_local? special inrerface in tag?)
        })?;
        Ok(())
    }
    pub fn open_slot(&self) -> crate::Result<SlotTag> {
        self.0.call_mut(|frame| frame.open_slot())?
    }
    pub fn close_slot(&self, slot: SlotTag) -> crate::Result<()> {
        self.0.call_mut(|frame| frame.close_slot(slot))?
    }
    pub fn on_size(&self) -> crate::Result<()> {
        self.0.call_mut(|frame| frame.on_size())?
    }
}
