use async_object::EventStream;
use bindings::Windows::Foundation::Numerics::Vector2;

#[derive(Clone)]
pub struct SlotSize(pub Vector2);

pub trait SendSlotEvent {
    fn send_size(&self, size: SlotSize) -> crate::Result<()>;
}

pub trait ReceiveSlotEvent {
    fn on_size(&self) -> EventStream<SlotSize>;
}
