use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use async_object::{Keeper, Tag};
use bindings::{
    Microsoft::Graphics::Canvas::{CanvasDevice, UI::Composition::CanvasComposition},
    Windows::{
        Foundation::Numerics::Vector2,
        UI::Composition::{CompositionGraphicsDevice, Compositor, ContainerVisual},
    },
};
use futures::{executor::LocalSpawner, task::LocalSpawnExt, Future};

use crate::slot::{Size, SlotKeeper, SlotTag};

pub struct Frame {
    tag: FrameTag,
    slots: Vec<SlotKeeper>,
}

impl Frame {
    fn new(spawner: LocalSpawner) -> crate::Result<Self> {
        let compositor = Compositor::new()?;
        let canvas_device = CanvasDevice::GetSharedDevice()?;
        let root_visual = compositor.CreateContainerVisual()?;
        let composition_graphics_device =
            CanvasComposition::CreateCompositionGraphicsDevice(&compositor, &canvas_device)?;

        Ok(Self {
            tag: FrameTag::new(
                spawner,
                compositor,
                root_visual,
                composition_graphics_device,
            ),
            slots: Vec::new(),
        })
    }
    pub fn set_size(&mut self, size: Vector2) -> crate::Result<()> {
        self.tag.root_visual.SetSize(size)?;
        if let Some(top) = self.slots.last() {
            top.send_size(Size::new(size))
        }
        Ok(())
    }
    pub fn open_slot_modal(&mut self) -> crate::Result<SlotTag> {
        let slot = SlotKeeper::new(self.tag.clone())?;
        self.tag
            .root_visual
            .Children()?
            .InsertAtTop(slot.container().clone())?;
        slot.send_size(Size::new(self.tag.root_visual.Size()?));
        let slot_tag = slot.tag();
        self.slots.push(slot);
        Ok(slot_tag)
    }
    pub fn close_slot(&mut self, slot: SlotTag) -> crate::Result<()> {
        self.tag.root_visual.Children()?.Remove(slot.container())?;
        if let Some(index) = self.slots.iter().position(|v| v.tag() == slot) {
            self.slots.remove(index);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct FrameKeeper {
    keeper: Keeper<Frame>,
    tag: FrameTag,
}
impl FrameKeeper {
    pub fn new(spawner: LocalSpawner) -> crate::Result<Self> {
        let keeper = Keeper::new(Frame::new(spawner)?);
        let tag = keeper.get().tag.clone();
        let keeper = Self { keeper, tag };
        keeper.get_mut().tag.tag = keeper.keeper.tag();
        Ok(keeper)
    }
    pub fn tag(&self) -> FrameTag {
        self.tag.clone()
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Frame> {
        self.keeper.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Frame> {
        self.keeper.get_mut()
    }
    pub fn compositor(&self) -> &Compositor {
        &self.tag.compositor
    }
    pub fn root_visual(&self) -> &ContainerVisual {
        &self.tag.root_visual
    }
    pub fn spawner(&self) -> &LocalSpawner {
        &self.tag.spawner
    }
}

#[derive(Clone)]
pub struct FrameTag {
    tag: Tag<Frame>,
    spawner: LocalSpawner,
    compositor: Compositor,
    root_visual: ContainerVisual,
    composition_graphics_device: CompositionGraphicsDevice,
}

impl FrameTag {
    fn new(
        spawner: LocalSpawner,
        compositor: Compositor,
        root_visual: ContainerVisual,
        composition_graphics_device: CompositionGraphicsDevice,
    ) -> Self {
        Self {
            tag: Tag::default(),
            spawner,
            compositor,
            root_visual,
            composition_graphics_device,
        }
    }
    pub fn compositor(&self) -> &Compositor {
        &self.compositor
    }
    pub fn root_visual(&self) -> &ContainerVisual {
        &self.root_visual
    }
    pub fn spawner(&self) -> &LocalSpawner {
        &self.spawner
    }
    pub fn set_size(&self, size: Vector2) -> crate::Result<()> {
        self.tag.call_mut(|g| g.set_size(size))?
    }
    pub fn spawn_local<Fut>(&self, future: Fut) -> crate::Result<()>
    where
        Fut: Future<Output = crate::Result<()>> + 'static,
    {
        self.spawner().spawn_local(async {
            future.await.unwrap() // TODO: store error somethere (thread_local? special inrerface in tag?)
        })?;
        Ok(())
    }
    pub fn open_modal_slot(&self) -> crate::Result<SlotTag> {
        self.tag.call_mut(|frame| frame.open_slot_modal())?
    }
    pub fn close_slot(&self, slot: SlotTag) -> crate::Result<()> {
        self.tag.call_mut(|frame| frame.close_slot(slot))?
    }
}

impl PartialEq for FrameTag {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}

// pub fn show_panel<T: 'static, P: Panel + Clone + AsRef<Tag<T>> + 'static>(
//     mut frame: FrameTag,
//     panel: P,
// ) -> crate::Result<()> {
//     frame.attach(panel.visual())?;
//     frame.spawn_local({
//         let mut frame = frame.clone();
//         let panel = panel.clone();
//         async move {
//             let mut wait_for_death = panel.as_ref().receive_events::<()>();
//             while let Some(_) = wait_for_death.next().await {}
//             frame.detach(panel.visual())
//         }
//     })?;
//     Ok(())
// }