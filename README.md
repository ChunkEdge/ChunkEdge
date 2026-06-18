<div align="center">

# ChunkEdge

<img src="https://raw.githubusercontent.com/ChunkEdge/ChunkEdge/rebrand/assets/logo-full.svg" width="650" align="center">

</div>

<!-- TODO: replace rebrand with main -->
<p align="center">
    <a href="https://github.com/ChunkEdge/ChunkEdge/blob/rebrand/LICENSE.txt">
        <img src="https://img.shields.io/github/license/ChunkEdge/ChunkEdge" alt="license">
    </a>
</p>

A Rust framework for building Minecraft: Java Edition servers.

Built on top of [Bevy ECS](https://bevyengine.org/learn/book/getting-started/ecs/), ChunkEdge is an effort to create a Minecraft compatible server completely from scratch in Rust. You can think of ChunkEdge as a _game engine for Minecraft servers_. It doesn't do much by default, but by writing game logic yourself and leveraging Bevy's powerful [plugin system](https://bevyengine.org/learn/book/getting-started/plugins/), you can make almost anything.

Opinionated features like dynamic scripting, dedicated executables, and vanilla game mechanics are all expected to be built as optional plugins. This level of modularity is desirable for those looking to build highly custom experiences in Minecraft such as minigame servers.

> [!WARNING]  
> ChunkEdge is still early in development with many features unimplemented or incomplete. Expect to encounter bugs, limitations, and breaking changes.

## Goals

ChunkEdge aims to be the following:

- **Complete**. Abstractions for the full breadth of the Minecraft protocol.
- **Flexible**. Can easily extend ChunkEdge from within user code. Direct access to the Minecraft protocol is provided.
- **Modular**. Pick and choose the components you need.
- **Intuitive**. An API that is easy to use and difficult to misuse. Extensive documentation and examples are important.
- **Efficient**. Optimal use of system resources with multiple CPU cores in mind. ChunkEdge uses very little memory and can support [thousands](https://raw.githubusercontent.com/ChunkEdge/ChunkEdge/rebrand/assets/many-players.png) of players at the same time without lag (assuming you have the bandwidth).
- **Up to date**. Targets the most recent stable version of Minecraft. Support for multiple versions at once is not planned. However, you can use a proxy with [ViaBackwards](https://www.spigotmc.org/resources/viabackwards.27448/) to achieve backwards compatibility with older clients.

### Current Status

Here are some noteworthy achievements:

- `chunkedge_nbt`: A speedy new library for Minecraft's Named Binary Tag (NBT) format.
- Authentication, encryption, and compression
- Block states
- Chunks
- Entities and metadata
- Bounding volume hierarchy for fast spatial entity queries
- Player list and player skins
- Dimensions, biomes, and worlds
- JSON Text API
- A Fabric mod for extracting data from the game into JSON files. These files are processed by a build script to generate Rust code for the project. The JSON files can be used in other projects as well.
- Inventories
- Items
- Particles
- Anvil file format (read only)
- Proxy support ([Velocity](https://velocitypowered.com/), [Bungeecord](https://www.spigotmc.org/wiki/bungeecord/) and [Waterfall](https://docs.papermc.io/waterfall))

Here is a [short video](https://www.youtube.com/watch?v=jkw9fZx9Etg) showing the examples and some of ChunkEdge's capabilities.

## Getting Started

### Running the Examples

After cloning the repository, run this command to try an example.

```shell
cargo r -r --example parkour
```

I also recommend giving `game_of_life`, `terrain`, and `cow_sphere` a try.

Next, open your Minecraft client and connect to the address `localhost`. If all goes well you should be playing on the server.

### Adding ChunkEdge as a Dependency

To use the most recent development version, add ChunkEdge as a [git dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories).

```toml
[dependencies]
chunkedge = { git = "https://github.com/ChunkEdge/ChunkEdge" }
```

<!-- TODO: Add actual docs website -->

Documentation from the main branch is available [here](https://docs.chunkedge.com/).

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](https://github.com/ChunkEdge/ChunkEdge/blob/rebrand/CONTRIBUTING.md). You can use [GitHub Discussions](https://github.com/ChunkEdge/ChunkEdge/discussions) to discuss the project and ask questions.

## License

The source code in this repository is licensed under the [MIT License](https://opensource.org/licenses/MIT), except where otherwise noted.

The project name, logo, icons, and other branding assets are not licensed under the MIT License. They are reserved by the repository owner [Job Paardekooper](https://github.com/jobpaardekooper) and may not be used in a way that suggests endorsement, affiliation, or official status without prior written permission.

## History

This project is a fork of [valence-rs/valence](https://github.com/valence-rs/valence) combined with the main protocol update [PR](https://github.com/valence-rs/valence/pull/675) [branch](https://github.com/JackCrumpLeys/valence/tree/update-minecraft-1.21) which was still open at the time of the fork.
