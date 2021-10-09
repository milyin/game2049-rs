use bindings::Windows::UI::Composition::{ContainerVisual, ShapeVisual};

use crate::WindowTag;

struct Background {
    container: ContainerVisual,
    shape: ShapeVisual,
}

impl Background {
    fn new(window: WindowTag) -> crate::Result<Self> {
        let compositor = window.compositor();
        let container = compositor.CreateContainerVisual()?;
        let shape = compositor.CreateShapeVisual()?;
        container.Children()?.InsertAtBottom(shape.clone())?;
        Ok(Self { container, shape })
    }
}
