use chunkedge_build_utils::write_generated_file;

mod attributes;
mod block;
mod chunk_view;
mod command_argument_type;
mod item;
mod packet_id;
mod particle_type;
mod sound;
mod status_effects;

pub fn main() -> anyhow::Result<()> {
    write_generated_file(attributes::build()?, "attributes.rs")?;
    write_generated_file(block::build()?, "block.rs")?;
    write_generated_file(item::build()?, "item.rs")?;
    write_generated_file(sound::build()?, "sound.rs")?;
    write_generated_file(packet_id::build()?, "packet_id.rs")?;
    write_generated_file(particle_type::build()?, "particle_type.rs")?;
    write_generated_file(command_argument_type::build()?, "command_argument_type.rs")?;
    write_generated_file(chunk_view::build(), "chunk_view.rs")?;
    write_generated_file(status_effects::build()?, "status_effects.rs")?;

    Ok(())
}
