use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use async_object::{Keeper, Tag};
use bindings::Windows::{
    Foundation::Numerics::Vector2,
    UI::{
        Color,
        Composition::{CompositionShape, ShapeVisual},
    },
};
use float_ord::FloatOrd;
use futures::StreamExt;

use crate::{FrameTag, SlotTag};

pub struct Background {
    frame: FrameTag,
    slot: SlotTag,
    shape: ShapeVisual,
    round_corners: bool,
    color: Color,
}

impl Background {
    fn new(
        frame: FrameTag,
        slot: SlotTag,
        color: Color,
        round_corners: bool,
    ) -> crate::Result<Self> {
        let compositor = frame.compositor()?;
        let shape = compositor.CreateShapeVisual()?;
        let container = slot.container()?;
        container.Children()?.InsertAtTop(shape.clone())?;
        shape.SetSize(container.Size()?)?;
        let background = Self {
            frame,
            slot,
            shape,
            color,
            round_corners,
        };
        background.redraw()?;
        Ok(background)
    }
    fn detach(&self) -> crate::Result<()> {
        self.slot.container()?.Children()?.Remove(&self.shape)?;
        Ok(())
    }

    fn set_color(&mut self, color: Color) -> crate::Result<()> {
        self.color = color;
        self.redraw()?;
        Ok(())
    }

    fn on_size(&mut self) -> crate::Result<()> {
        self.shape.SetSize(self.slot.container()?.Size()?)?;
        self.redraw()?;
        Ok(())
    }

    fn redraw(&self) -> crate::Result<()> {
        self.shape.Shapes()?.Clear()?;
        self.shape
            .Shapes()?
            .Append(self.create_background_shape()?)?;
        Ok(())
    }
    fn create_background_shape(&self) -> crate::Result<CompositionShape> {
        let compositor = self.frame.compositor()?;
        let container_shape = compositor.CreateContainerShape()?;
        let rect_geometry = compositor.CreateRoundedRectangleGeometry()?;
        rect_geometry.SetSize(self.shape.Size()?)?;
        if self.round_corners {
            let size = rect_geometry.Size()?;
            let radius = std::cmp::min(FloatOrd(size.X), FloatOrd(size.Y)).0 / 20.;
            rect_geometry.SetCornerRadius(Vector2 {
                X: radius,
                Y: radius,
            })?;
        } else {
            rect_geometry.SetCornerRadius(Vector2 { X: 0., Y: 0. })?;
        }
        let brush = compositor.CreateColorBrushWithColor(self.color.clone())?;
        let rect = compositor.CreateSpriteShapeWithGeometry(rect_geometry)?;
        rect.SetFillBrush(brush)?;
        rect.SetOffset(Vector2 { X: 0., Y: 0. })?;
        container_shape.Shapes()?.Append(rect)?;
        let shape = container_shape.into();
        Ok(shape)
    }
}

impl Drop for Background {
    fn drop(&mut self) {
        let _ = self.detach();
    }
}

#[derive(Clone)]
pub struct BackgroundKeeper(Keeper<Background>);

impl BackgroundKeeper {
    pub fn new(
        frame: FrameTag,
        slot: SlotTag,
        color: Color,
        round_corners: bool,
    ) -> crate::Result<Self> {
        let keeper = Keeper::new(Background::new(frame, slot, color, round_corners)?);
        let keeper = Self(keeper);
        keeper.spawn_event_handlers()?;
        Ok(keeper)
    }
    pub fn tag(&self) -> BackgroundTag {
        BackgroundTag(self.0.tag())
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Background> {
        self.0.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Background> {
        self.0.get_mut()
    }
    fn spawn_event_handlers(&self) -> crate::Result<()> {
        let tag = self.tag();
        let frame = self.get().frame.clone();
        let slot = self.get().slot.clone();
        frame.spawn_local(async move {
            while let Some(_) = slot.on_raw_size().next().await {
                tag.on_size()?;
            }
            Ok(())
        })
    }
}
#[derive(Clone, PartialEq)]
pub struct BackgroundTag(Tag<Background>);

impl BackgroundTag {
    pub fn round_corners(&self) -> crate::Result<bool> {
        Ok(self.0.call(|v| v.round_corners)?)
    }
    pub fn color(&self) -> crate::Result<Color> {
        Ok(self.0.call(|v| v.color)?)
    }
    pub fn set_color(&self, color: Color) -> crate::Result<()> {
        Ok(self.0.call_mut(|v| v.set_color(color))??)
    }
    pub fn on_size(&self) -> crate::Result<()> {
        Ok(self.0.call_mut(|v| v.on_size())??)
    }
}
