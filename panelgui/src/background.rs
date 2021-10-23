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

use crate::{
    slot::{RawEvent, Size},
    FrameTag, SlotTag,
};

#[derive(Clone)]
struct BackgroundRefs {
    frame: FrameTag,
    slot: SlotTag,
    shape: ShapeVisual,
}

impl BackgroundRefs {
    fn new(frame: FrameTag, slot: SlotTag) -> crate::Result<Self> {
        let shape = frame.compositor().CreateShapeVisual()?;
        slot.container().Children()?.InsertAtBottom(shape.clone())?;
        shape.SetSize(slot.container().Size()?)?;
        Ok(Self { frame, slot, shape })
    }
    fn detach(&self) -> crate::Result<()> {
        self.slot.container().Children()?.Remove(&self.shape)?;
        Ok(())
    }
    fn redraw_background(&self, round_corners: bool, color: Color) -> windows::Result<()> {
        self.shape.Shapes()?.Clear()?;
        self.shape
            .Shapes()?
            .Append(self.create_background_shape(round_corners, color)?)?;
        Ok(())
    }
    fn create_background_shape(
        &self,
        round_corners: bool,
        color: Color,
    ) -> windows::Result<CompositionShape> {
        let compositor = self.frame.compositor();
        let container_shape = compositor.CreateContainerShape()?;
        let rect_geometry = compositor.CreateRoundedRectangleGeometry()?;
        rect_geometry.SetSize(self.shape.Size()?)?;
        if round_corners {
            let size = rect_geometry.Size()?;
            let radius = std::cmp::min(FloatOrd(size.X), FloatOrd(size.Y)).0 / 20.;
            rect_geometry.SetCornerRadius(Vector2 {
                X: radius,
                Y: radius,
            })?;
        } else {
            rect_geometry.SetCornerRadius(Vector2 { X: 0., Y: 0. })?;
        }
        let brush = compositor.CreateColorBrushWithColor(color.clone())?;
        let rect = compositor.CreateSpriteShapeWithGeometry(rect_geometry)?;
        rect.SetFillBrush(brush)?;
        rect.SetOffset(Vector2 { X: 0., Y: 0. })?;
        container_shape.Shapes()?.Append(rect)?;
        let shape = container_shape.into();
        Ok(shape)
    }
}

pub struct Background {
    refs: BackgroundRefs,
    round_corners: bool,
    color: Color,
}

impl Background {
    fn new(refs: BackgroundRefs, color: Color, round_corners: bool) -> crate::Result<Self> {
        refs.redraw_background(round_corners, color)?;
        Ok(Self {
            refs,
            color,
            round_corners,
        })
    }
    fn set_color(&mut self, color: Color) -> crate::Result<()> {
        self.color = color;
        self.refs
            .redraw_background(self.round_corners, self.color)?;
        Ok(())
    }
}

impl Drop for Background {
    fn drop(&mut self) {
        let _ = self.refs.detach();
    }
}

#[derive(Clone)]
pub struct BackgroundKeeper {
    keeper: Keeper<Background>,
    refs: BackgroundRefs,
}

impl BackgroundKeeper {
    pub fn new(
        frame: &FrameTag,
        slot: SlotTag,
        color: Color,
        round_corners: bool,
    ) -> crate::Result<Self> {
        let refs = BackgroundRefs::new(frame.clone(), slot)?;
        let keeper = Keeper::new(Background::new(refs.clone(), color, round_corners)?);
        let keeper = Self { keeper, refs };
        Self::spawn_event_handlers(keeper.tag())?;
        Ok(keeper)
    }
    pub fn tag(&self) -> BackgroundTag {
        BackgroundTag {
            tag: self.keeper.tag(),
            refs: self.refs.clone(),
        }
    }
    pub fn get(&self) -> RwLockReadGuard<'_, Background> {
        self.keeper.get()
    }
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Background> {
        self.keeper.get_mut()
    }
    fn spawn_event_handlers(tag: BackgroundTag) -> crate::Result<()> {
        tag.clone().refs.frame.spawn_local(async move {
            while let Some(size) = tag.refs.slot.on_raw_size().next().await {
                let RawEvent(Size(size)) = size;
                tag.refs.shape.SetSize(size)?;
                tag.refs
                    .redraw_background(tag.round_corners()?, tag.color()?)?;
            }
            Ok(())
        })
    }
}
#[derive(Clone)]
pub struct BackgroundTag {
    tag: Tag<Background>,
    refs: BackgroundRefs,
}

impl BackgroundTag {
    pub fn round_corners(&self) -> crate::Result<bool> {
        Ok(self.tag.call(|v| v.round_corners)?)
    }
    pub fn color(&self) -> crate::Result<Color> {
        Ok(self.tag.call(|v| v.color)?)
    }
    pub fn set_color(&self, color: Color) -> crate::Result<()> {
        Ok(self.tag.call_mut(|v| v.set_color(color))??)
    }
}

impl PartialEq for BackgroundTag {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}
