use application::ports::IdGenerator;

/// UUIDv4 adapter for the [`IdGenerator`] port.
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn generate(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
