# `chunkedge_entity`

Components and systems concerning Minecraft entities. This includes "zombie", "chicken", "player", etc.

Each Minecraft entity class has a module and a marker component. Spawning the marker adds its required components, including its parent markers and default tracked data.
For example:

```rust
use bevy_ecs::prelude::*;
use chunkedge_entity::{zombie, EntityKind};

let mut world = World::new();
let zombie = world.spawn((zombie::ZombieEntity, zombie::Baby(true))).id();
assert_eq!(*world.get::<EntityKind>(zombie).unwrap(), EntityKind::ZOMBIE);
```

In this example, `zombie::ZombieEntity` is a [Bevy component](https://docs.rs/bevy/latest/bevy/prelude/trait.Component.html) that identifies a Minecraft class and requires its inherited components. The value returned by `.id()` is Bevy's [`Entity` handle](https://docs.rs/bevy/latest/bevy/ecs/entity/index.html#usage). A query filtered with `With<monster::MonsterEntity>` also matches zombies because their marker requires the parent markers.
