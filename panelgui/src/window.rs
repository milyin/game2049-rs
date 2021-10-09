use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_object::{Keeper, Tag};
use bindings::{
    Microsoft::Graphics::Canvas::{CanvasDevice, UI::Composition::CanvasComposition},
    Windows::{
        Foundation::Numerics::Vector2,
        UI::Composition::{CompositionGraphicsDevice, Compositor, ContainerVisual},
    },
};
use futures::executor::LocalSpawner;

pub struct Window {
    compositor: Compositor,
    root_visual: ContainerVisual,
    canvas_device: CanvasDevice,
    composition_graphics_device: CompositionGraphicsDevice,
}

impl Window {
    fn new() -> crate::Result<Self> {
        let compositor = Compositor::new()?;
        let canvas_device = CanvasDevice::GetSharedDevice()?;
        let root_visual = compositor.CreateContainerVisual()?;
        let composition_graphics_device =
            CanvasComposition::CreateCompositionGraphicsDevice(&compositor, &canvas_device)?;

        let background_shape = compositor.CreateShapeVisual()?;

        Ok(Self {
            compositor,
            root_visual,
            canvas_device,
            composition_graphics_device,
        })
    }
    pub fn set_window_size(&mut self, size: Vector2) -> crate::Result<()> {
        Ok(self.root_visual.SetSize(size)?)
    }
}

#[derive(Clone)]
pub struct WindowKeeper {
    keeper: Keeper<Window>,
    spawner: LocalSpawner,
    compositor: Compositor,
    root_visual: ContainerVisual,
}
impl WindowKeeper {
    pub fn new(spawner: LocalSpawner) -> crate::Result<Self> {
        let window = Window::new()?;
        let compositor = window.compositor.clone();
        let root_visual = window.root_visual.clone();
        let keeper = Keeper::new(window);
        Ok(Self {
            spawner,
            keeper,
            compositor,
            root_visual,
        })
    }
    pub fn tag(&self) -> WindowTag {
        WindowTag {
            tag: self.keeper.tag(),
            spawner: self.spawner.clone(),
            compositor: self.compositor.clone(),
            root_visual: self.root_visual.clone(),
        }
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Window> {
        self.keeper.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Window> {
        self.keeper.get_mut()
    }
}

impl AsRef<Arc<RwLock<Window>>> for WindowKeeper {
    fn as_ref(&self) -> &Arc<RwLock<Window>> {
        self.keeper.as_ref()
    }
}

#[derive(Clone)]
pub struct WindowTag {
    tag: Tag<Window>,
    spawner: LocalSpawner,
    compositor: Compositor,
    root_visual: ContainerVisual,
}

impl WindowTag {
    pub fn compositor(&self) -> &Compositor {
        &self.compositor
    }
    pub fn root_visual(&self) -> &ContainerVisual {
        &self.root_visual
    }
    pub fn spawner(&self) -> &LocalSpawner {
        &self.spawner
    }
    pub async fn set_window_size(&self, size: Vector2) -> crate::Result<()> {
        self.tag.async_call_mut(|g| g.set_window_size(size)).await?
    }
}
