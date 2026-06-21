<div align="center">
<img src="https://raw.githubusercontent.com/ChunkEdge/ChunkEdge/main/assets/logo-full.svg" width="650" align="center">
</div>

# ChunkEdge

<p align="center">

![License](https://img.shields.io/github/license/ChunkEdge/ChunkEdge)
![Minecraft version](https://img.shields.io/badge/Minecraft_version-1.21.5-blue)
[![Documentation](https://img.shields.io/badge/Documentation-main-blue)](https://docs.chunkedge.com/chunkedge/)
![Tests](https://github.com/ChunkEdge/ChunkEdge/actions/workflows/ci.yml/badge.svg)
![ChunkEdge Repo stars](https://img.shields.io/github/stars/ChunkEdge/ChunkEdge)

</p>

A Rust framework for building Minecraft: Java Edition servers.

Built on top of [Bevy ECS](https://bevy.org/learn/quick-start/getting-started/ecs/), ChunkEdge is an effort to create a Minecraft-compatible server completely from scratch in Rust. You can think of ChunkEdge as a _game engine for Minecraft servers_ similar to the [Minestom project](https://github.com/Minestom/Minestom). It doesn't do much by default, but by writing game logic yourself and leveraging Bevy's powerful [plugin system](https://bevy.org/learn/quick-start/getting-started/plugins/), you can make almost anything.

Opinionated features like dynamic scripting, dedicated executables, and vanilla game mechanics are all expected to be built as optional plugins. This level of modularity is desirable for those looking to build highly custom experiences in Minecraft such as minigame servers.

> [!WARNING]  
> ChunkEdge is still early in development with many features unimplemented or incomplete. Expect to encounter bugs, limitations, and breaking changes. At the moment, ChunkEdge is only compatible with Minecraft `1.21.5`. The aim is to support the latest stable version of Minecraft. For now, you can use a proxy (for example [ViaProxy](https://github.com/ViaVersion/ViaProxy) combined with [Velocity](https://papermc.io/software/velocity/)) to allow players on both older and newer clients to connect to your server.

## Goals

ChunkEdge aims to be the following:

- **Complete**. Abstractions for the full breadth of the Minecraft protocol.
- **Flexible**. ChunkEdge can be easily extended from within user code. Direct access to the Minecraft protocol is provided.
- **Modular**. Pick and choose the components you need. Some crates can also be reused in projects that don't use the full ChunkEdge framework.
- **Intuitive**. An API that is easy to use and difficult to misuse. Extensive documentation and examples are important.
- **Efficient**. Optimal use of system resources with multiple CPU cores in mind.
- **Up-to-date**. Tries to be up-to-date with the most recent stable version of Minecraft. Currently Minecraft `1.21.5` is supported. Support for multiple versions at once is not planned. However, you can use a proxy (for example [ViaProxy](https://github.com/ViaVersion/ViaProxy) combined with [Velocity](https://papermc.io/software/velocity/)) to allow players on both older and newer clients to connect to your server.

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
- Anvil file format (read-only)
- Proxy support ([Velocity](https://papermc.io/software/velocity/))

## Getting Started

### Creating a New Project

To create your own server project using ChunkEdge, please see the [Getting Started guide](https://docs.chunkedge.com/chunkedge/docs/guides/getting_started/).

Other usefull resources from the documentation include:

- [Full API Documentation (rustdoc)](https://docs.chunkedge.com/chunkedge/)
- [General docs page with links to guides and the FAQ](https://docs.chunkedge.com/chunkedge/docs/)

### Running the Examples

After cloning the repository, run this command to try an example.

```bash
cargo r -r --example parkour
```

List all available examples with:

```bash
cargo run --example
```

Recommended examples to try are `parkour`, `game_of_life`, `terrain`, and `cow_sphere`.

Open your Minecraft client and connect to the address `localhost`. If all goes well, you should be able to play the example.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](https://github.com/ChunkEdge/ChunkEdge/blob/main/CONTRIBUTING.md). You can use [GitHub Discussions](https://github.com/ChunkEdge/ChunkEdge/discussions) to discuss the project and ask questions.

## License

The source code in this repository is licensed under the [MIT License](https://opensource.org/licenses/MIT), except where otherwise noted.

The project name, logo, icons, and other branding assets are not licensed under the MIT License. They are reserved by the repository owner [Job Paardekooper](https://github.com/jobpaardekooper) and may not be used in a way that suggests endorsement, affiliation, or official status without prior written permission.

## History

This project is a fork of [valence-rs/valence](https://github.com/valence-rs/valence), combined with code from the then-open Minecraft protocol update [PR](https://github.com/valence-rs/valence/pull/675) / [branch](https://github.com/JackCrumpLeys/valence/tree/update-minecraft-1.21).

## Star History

<a href="https://www.star-history.com/?repos=ChunkEdge%2FChunkEdge&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=ChunkEdge/ChunkEdge&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=ChunkEdge/ChunkEdge&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=ChunkEdge/ChunkEdge&type=date&legend=top-left" />
 </picture>
</a>
