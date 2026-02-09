use bevy::ecs::entity::Entity;
use std::cell::RefCell;

thread_local! {
    static ENTITY_COUNTER: RefCell<u32> = const { RefCell::new(0) };
}

/// Creates a mock Entity for testing purposes.
/// Each call returns a unique Entity with an incrementing index.
pub fn create_test_entity() -> Entity {
    ENTITY_COUNTER.with(|counter| {
        let index = *counter.borrow();
        *counter.borrow_mut() += 1;
        Entity::from_raw_u32(index).unwrap()
    })
}
