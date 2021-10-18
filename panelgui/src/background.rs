use bindings::Windows::UI::Composition::{ContainerVisual, ShapeVisual};

use crate::FrameTag;

struct Background {
    container: ContainerVisual,
    shape: ShapeVisual,
}

impl Background {
    fn new(window: FrameTag) -> crate::Result<Self> {
        let compositor = window.compositor();
        let container = compositor.CreateContainerVisual()?;
        let shape = compositor.CreateShapeVisual()?;
        container.Children()?.InsertAtBottom(shape.clone())?;
        Ok(Self { container, shape })
    }
}
