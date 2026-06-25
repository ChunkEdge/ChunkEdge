use std::collections::HashMap;
use std::marker::PhantomData;

use bevy_app::{App, Plugin, PostStartup};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::message::{Message, MessageReader, MessageWriter};
use bevy_ecs::prelude::{Entity, IntoScheduleConfigs, Resource};
use chunkedge_server::EventLoopPreUpdate;
use petgraph::prelude::NodeIndex;

use crate::graph::CommandGraphBuilder;
use crate::modifier_value::ModifierValue;
use crate::parsers::ParseInput;
use crate::{
    Command, CommandProcessedMessage, CommandRegistry, CommandScopeRegistry, CommandSystemSet,
};

impl<T> Plugin for CommandHandlerPlugin<T>
where
    T: Command + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_message::<CommandResultMessage<T>>()
            .insert_resource(CommandResource::<T>::new())
            .add_systems(PostStartup, command_startup_system::<T>)
            .add_systems(
                EventLoopPreUpdate,
                command_message_system::<T>.after(CommandSystemSet),
            );
    }
}

pub struct CommandHandlerPlugin<T>
where
    T: Command,
{
    command: PhantomData<T>,
}

impl<T: Command> Default for CommandHandlerPlugin<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CommandHandlerPlugin<T>
where
    T: Command,
{
    pub fn new() -> Self {
        CommandHandlerPlugin {
            command: PhantomData,
        }
    }
}

#[derive(Resource)]
struct CommandResource<T: Command + Send + Sync> {
    command: PhantomData<T>,
    executables: HashMap<NodeIndex, fn(&mut ParseInput) -> T>,
}

impl<T: Command + Send + Sync> CommandResource<T> {
    fn new() -> Self {
        CommandResource {
            command: PhantomData,
            executables: HashMap::new(),
        }
    }
}

#[derive(Message)]
pub struct CommandResultMessage<T>
where
    T: Command,
    T: Send + Sync + 'static,
{
    pub result: T,
    pub executor: Entity,
    pub modifiers: HashMap<ModifierValue, ModifierValue>,
}

fn command_startup_system<T>(
    mut registry: ResMut<CommandRegistry>,
    mut scope_registry: ResMut<CommandScopeRegistry>,
    mut command: ResMut<CommandResource<T>>,
) where
    T: Command + Send + Sync + 'static,
{
    let mut executables = HashMap::new();
    let mut parsers = HashMap::new();
    let mut modifiers = HashMap::new();
    let graph_builder = &mut CommandGraphBuilder::new(
        &mut registry,
        &mut executables,
        &mut parsers,
        &mut modifiers,
    );
    T::assemble_graph(graph_builder);
    graph_builder.apply_scopes(&mut scope_registry);

    command.executables.extend(executables.clone());
    registry.parsers.extend(parsers);
    registry.modifiers.extend(modifiers);
    registry.executables.extend(executables.keys());
}

/// This system reads incoming command messages.
fn command_message_system<T>(
    mut commands_executed: MessageReader<CommandProcessedMessage>,
    mut messages: MessageWriter<CommandResultMessage<T>>,
    command: ResMut<CommandResource<T>>,
) where
    T: Command + Send + Sync,
{
    for command_message in commands_executed.read() {
        if let Some(executable) = command.executables.get(&command_message.node) {
            let result = executable(&mut ParseInput::new(&command_message.command));
            messages.write(CommandResultMessage {
                result,
                executor: command_message.executor,
                modifiers: command_message.modifiers.clone(),
            });
        }
    }
}
