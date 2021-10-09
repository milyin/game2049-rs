use async_object::{Keeper, Tag};
use bindings::{
    Microsoft::Graphics::Canvas::{CanvasDevice, UI::Composition::CanvasComposition},
    Windows::{
        Foundation::Numerics::Vector2,
        Win32::Graphics::Gdi::HGDIOBJ,
        UI::Composition::{CompositionGraphicsDevice, Compositor, ContainerVisual},
    },
};

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
        Ok(Self {
            compositor,
            root_visual,
            canvas_device,
            composition_graphics_device,
        })
    }
    fn set_window_size(&mut self, size: Vector2) -> crate::Result<()> {
        Ok(self.root_visual.SetSize(size)?)
    }
}

pub struct WindowKeeper {
    keeper: Keeper<Window>,
    compositor: Compositor,
    root_visual: ContainerVisual,
}
impl WindowKeeper {
    pub fn new() -> crate::Result<Self> {
        let globals = Window::new()?;
        let compositor = globals.compositor.clone();
        let root_visual = globals.root_visual.clone();
        let keeper = Keeper::new(globals);
        Ok(Self {
            keeper,
            compositor,
            root_visual,
        })
    }
    pub fn handle(&self) -> WindowTag {
        WindowTag {
            tag: self.keeper.tag(),
            compositor: self.compositor.clone(),
            root_visual: self.root_visual.clone(),
        }
    }
}
#[derive(Clone)]
pub struct WindowTag {
    tag: Tag<Window>,
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
    pub async fn set_window_size(&self, size: Vector2) -> crate::Result<()> {
        self.tag
            .call_mut(|g| g.set_window_size(size))
            .await
            .unwrap_or(Result::Err(crate::Error::AsyncObjectDestroyed))
    }
}
