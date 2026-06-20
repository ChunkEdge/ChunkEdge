use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::sync::atomic::{AtomicU32, Ordering};

use parking_lot::Mutex; // Using nonstandard mutex to avoid poisoning API.
use valence_binary::Encode;
use valence_generated::block::{BlockKind, PropName, PropValue};
use valence_nbt::Compound;
use valence_protocol::encode::{PacketWriter, WritePacket};
use valence_protocol::packets::play::level_chunk_with_light_s2c::{
    ChunkDataBlockEntity, HeightMap, HeightMapKind,
};
use valence_protocol::packets::play::section_blocks_update_s2c::ChunkDeltaUpdateEntry;
use valence_protocol::packets::play::{
    BlockEntityDataS2c, BlockUpdateS2c, LevelChunkWithLightS2c, SectionBlocksUpdateS2c,
};
use valence_protocol::{
    BlockPos, BlockState, ChunkPos, ChunkSectionPos, FixedArray, VariableBitSet,
};
use valence_registry::biome::BiomeId;
use valence_registry::RegistryIdx;

use super::chunk::{
    bit_width, check_biome_oob, check_block_oob, check_section_oob, BiomeContainer,
    BlockStateContainer, Chunk, SECTION_BLOCK_COUNT,
};
use super::paletted_container::PalettedContainer;
use super::unloaded::{self, UnloadedChunk};
use super::{ChunkLayerInfo, ChunkLayerMessages, LocalMsg};

#[derive(Debug)]
pub struct LoadedChunk {
    /// A count of the clients viewing this chunk. Useful for knowing if it's
    /// necessary to record changes, since no client would be in view to receive
    /// the changes if this were zero.
    viewer_count: AtomicU32,
    /// Block and biome data for the chunk.
    sections: Box<[Section]>,
    /// Sky light data for the chunk. Light sections have one extra section at
    /// the top and bottom to account for skylight changes above and below the
    /// chunk.
    sky_light_sections: Box<[LightSection]>,
    /// Block light data for the chunk. Light sections have one extra section at
    /// the top and bottom to account for light changes above and below the
    /// chunk.
    block_light_sections: Box<[LightSection]>,
    /// The block entities in this chunk.
    block_entities: BTreeMap<u32, Compound>,
    /// The set of block entities that have been modified this tick.
    changed_block_entities: BTreeSet<u32>,
    /// If any biomes in this chunk have been modified this tick.
    changed_biomes: bool,
    /// Cached bytes of the chunk initialization packet. The cache is considered
    /// invalidated if empty. This should be cleared whenever the chunk is
    /// modified in an observable way, even if the chunk is not viewed.
    cached_init_packets: Mutex<Vec<u8>>,
}

#[derive(Clone, Debug, Default)]
pub struct Section {
    block_states: BlockStateContainer,
    biomes: BiomeContainer,
    /// Contains modifications for the update section packet. (Or the regular
    /// block update packet if len == 1).
    updates: Vec<ChunkDeltaUpdateEntry>,
}

impl Section {
    fn count_non_air_blocks(&self) -> u16 {
        let mut count = 0;

        match &self.block_states {
            PalettedContainer::Single(s) => {
                if !s.is_air() {
                    count += SECTION_BLOCK_COUNT as u16;
                }
            }
            PalettedContainer::Indirect(ind) => {
                for i in 0..SECTION_BLOCK_COUNT {
                    if !ind.get(i).is_air() {
                        count += 1;
                    }
                }
            }
            PalettedContainer::Direct(dir) => {
                for s in dir.as_ref() {
                    if !s.is_air() {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

/// Enum describing the light contents of a data section.
///
/// We need to differentiate between [`LightSection::NotSet`] and
/// [`LightSection::FullyDark`]. This is because, for sky light,
/// [`LightSection::NotSet`] could mean the section is either fully lit or fully
/// dark, and the client should deduce that from the sky light data that is
/// included.
#[derive(Clone, Debug, Default)]
pub enum LightSection {
    #[default]
    NotSet,
    FullyDark,
    FullData(Box<[u8; 2048]>),
}

impl LightSection {
    /// Create a new section of light data with the given raw byte array
    pub fn from_data(data: [u8; 2048]) -> Self {
        Self::FullData(Box::new(data))
    }
}

impl LoadedChunk {
    pub(crate) fn new(height: u32) -> Self {
        let section_count = height as usize / 16;
        let light_section_count = section_count + 2;
        Self {
            viewer_count: AtomicU32::new(0),
            sections: vec![Section::default(); section_count].into(),
            // We don't have a full lighting engine implemented so we set all sky light to be
            // NotSet so that no light data is sent to the client and we rely
            // on a hack that sets ambient light to full brightness for all dimensions
            // to make the chunks appear fully lit. We don't send a full light section filled with
            // 0xFF here, instead of the ambient light hack, because this is extremely unoptimized
            // in terms of memory consumption and crashes the many_players_spread_out
            // benchmark/test.
            sky_light_sections: vec![LightSection::NotSet; light_section_count].into(),
            // We don't have a full lighting engine implemented so we set all block light to be
            // fully dark.
            block_light_sections: vec![LightSection::FullyDark; light_section_count].into(),
            block_entities: BTreeMap::new(),
            changed_block_entities: BTreeSet::new(),
            changed_biomes: false,
            cached_init_packets: Mutex::new(vec![]),
        }
    }

    /// Sets the content of this chunk to the supplied [`UnloadedChunk`]. The
    /// given unloaded chunk is [resized] to match the height of this loaded
    /// chunk prior to insertion.
    ///
    /// The previous chunk data is returned.
    ///
    /// [resized]: UnloadedChunk::set_height
    pub(crate) fn insert(&mut self, mut chunk: UnloadedChunk) -> UnloadedChunk {
        chunk.set_height(self.height());

        let old_sections = self
            .sections
            .iter_mut()
            .zip(chunk.sections)
            .map(|(sect, other_sect)| {
                sect.updates.clear();

                unloaded::Section {
                    block_states: mem::replace(&mut sect.block_states, other_sect.block_states),
                    biomes: mem::replace(&mut sect.biomes, other_sect.biomes),
                }
            })
            .collect();
        let old_block_entities = mem::replace(&mut self.block_entities, chunk.block_entities);
        self.changed_block_entities.clear();
        self.changed_biomes = false;
        self.cached_init_packets.get_mut().clear();
        self.assert_no_changes();

        UnloadedChunk {
            sections: old_sections,
            block_entities: old_block_entities,
        }
    }

    pub(crate) fn remove(&mut self) -> UnloadedChunk {
        let old_sections = self
            .sections
            .iter_mut()
            .map(|sect| {
                sect.updates.clear();

                unloaded::Section {
                    block_states: mem::take(&mut sect.block_states),
                    biomes: mem::take(&mut sect.biomes),
                }
            })
            .collect();
        let old_block_entities = mem::take(&mut self.block_entities);
        self.changed_block_entities.clear();
        self.changed_biomes = false;
        self.cached_init_packets.get_mut().clear();

        self.assert_no_changes();

        UnloadedChunk {
            sections: old_sections,
            block_entities: old_block_entities,
        }
    }

    /// Returns the number of clients in view of this chunk.
    pub fn viewer_count(&self) -> u32 {
        self.viewer_count.load(Ordering::Relaxed)
    }

    /// Like [`Self::viewer_count`], but avoids an atomic operation.
    pub fn viewer_count_mut(&mut self) -> u32 {
        *self.viewer_count.get_mut()
    }

    /// Increments the viewer count.
    pub(crate) fn inc_viewer_count(&self) {
        self.viewer_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrements the viewer count.
    #[track_caller]
    pub(crate) fn dec_viewer_count(&self) {
        let old = self.viewer_count.fetch_sub(1, Ordering::Relaxed);
        debug_assert_ne!(old, 0, "viewer count underflow!");
    }

    /// Performs the changes necessary to prepare this chunk for client updates.
    /// - Chunk change messages are written to the layer.
    /// - Recorded changes are cleared.
    pub(crate) fn update_pre_client(
        &mut self,
        pos: ChunkPos,
        info: &ChunkLayerInfo,
        messages: &mut ChunkLayerMessages,
    ) {
        if *self.viewer_count.get_mut() == 0 {
            // Nobody is viewing the chunk, so no need to send any update packets. There
            // also shouldn't be any changes that need to be cleared.
            self.assert_no_changes();

            return;
        }

        // Block states
        for (sect_y, sect) in self.sections.iter_mut().enumerate() {
            match sect.updates.as_slice() {
                &[] => {}
                &[entry] => {
                    let global_x = pos.x * 16 + i32::from(entry.off_x());
                    let global_y = info.min_y + sect_y as i32 * 16 + i32::from(entry.off_y());
                    let global_z = pos.z * 16 + i32::from(entry.off_z());

                    messages.send_local_infallible(LocalMsg::PacketAt { pos }, |buf| {
                        let mut writer = PacketWriter::new(buf, info.threshold);

                        writer.write_packet(&BlockUpdateS2c {
                            position: BlockPos::new(global_x, global_y, global_z),
                            block_id: BlockState::from_raw(entry.block_state() as u16).unwrap(),
                        });
                    });
                }
                entries => {
                    let chunk_sect_pos = ChunkSectionPos {
                        x: pos.x,
                        y: sect_y as i32 + info.min_y.div_euclid(16),
                        z: pos.z,
                    };

                    messages.send_local_infallible(LocalMsg::PacketAt { pos }, |buf| {
                        let mut writer = PacketWriter::new(buf, info.threshold);

                        writer.write_packet(&SectionBlocksUpdateS2c {
                            chunk_sect_pos,
                            blocks: Cow::Borrowed(entries),
                        });
                    });
                }
            }

            sect.updates.clear();
        }

        // Block entities
        for &idx in &self.changed_block_entities {
            let Some(nbt) = self.block_entities.get(&idx) else {
                continue;
            };

            let x = idx % 16;
            let z = (idx / 16) % 16;
            let y = idx / 16 / 16;

            let state = self.sections[y as usize / 16]
                .block_states
                .get(idx as usize % SECTION_BLOCK_COUNT);

            let Some(kind) = state.block_entity_kind() else {
                continue;
            };

            let global_x = pos.x * 16 + x as i32;
            let global_y = info.min_y + y as i32;
            let global_z = pos.z * 16 + z as i32;

            messages.send_local_infallible(LocalMsg::PacketAt { pos }, |buf| {
                let mut writer = PacketWriter::new(buf, info.threshold);

                writer.write_packet(&BlockEntityDataS2c {
                    location: BlockPos::new(global_x, global_y, global_z),
                    kind,
                    data: Cow::Borrowed(nbt),
                });
            });
        }

        self.changed_block_entities.clear();

        // Biomes
        if self.changed_biomes {
            self.changed_biomes = false;

            messages.send_local_infallible(LocalMsg::ChangeBiome { pos }, |buf| {
                for sect in &self.sections {
                    sect.biomes
                        .encode_mc_format(
                            &mut *buf,
                            |b| b.to_index() as u64,
                            0,
                            3,
                            bit_width(info.biome_registry_len - 1),
                        )
                        .expect("paletted container encode should always succeed");
                }
            });
        }

        // All changes should be cleared.
        self.assert_no_changes();
    }

    /// Generates the `MOTION_BLOCKING` heightmap for this chunk, which stores
    /// the height of the highest non motion-blocking block in each column.
    ///
    /// `MOTION_BLOCKING` considers:
    ///
    /// "Solid" blocks, except bamboo saplings and cactuses; fluids. To
    /// determine where to display rain and snow.
    ///
    /// [Minecraft Wiki `MOTION_BLOCKING`](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Heightmap_structure:~:text=MOTION%5FBLOCKING)
    pub(crate) fn motion_blocking(&self) -> [u32; 16 * 16] {
        self.build_heightmap(Self::is_motion_blocking_occupied)
    }

    /// Generates the `MOTION_BLOCKING_NO_LEAVES` heightmap for this chunk,
    /// which stores the height of the highest non motion-blocking and non-leaf
    /// block in each column.
    ///
    /// `MOTION_BLOCKING_NO_LEAVES` is the same as `MOTION_BLOCKING`, but also
    /// considers leaf blocks to be non-blocking.
    ///
    /// [Minecraft Wiki `MOTION_BLOCKING_NO_LEAVES`](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Heightmap_structure:~:text=MOTION%5FBLOCKING%5FNO%5FLEAVES)
    pub(crate) fn motion_blocking_no_leaves(&self) -> [u32; 16 * 16] {
        self.build_heightmap(Self::is_motion_blocking_no_leaves_occupied)
    }

    /// Generates the `WORLD_SURFACE` heightmap for this chunk, which stores the
    /// height of the highest non-air block in each column.
    ///
    /// `WORLD_SURFACE` cosiders:
    ///
    /// All blocks other than air, cave air and void air. To determine if a
    /// beacon beam is obstructed.
    ///
    /// [Minecraft Wiki `WORLD_SURFACE`](https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Heightmap_structure:~:text=WORLD%5FSURFACE)
    pub(crate) fn world_surface(&self) -> [u32; 16 * 16] {
        self.build_heightmap(|state| !state.is_air())
    }

    /// Generates a heightmap for this chunk using the provided predicate.
    ///
    /// The lowest value of the heightmap is 0, which means that there are no
    /// blocks matching the predicate in the column. Since 0 is reserved for
    /// this case, the heightmap values are 1-indexed. A value of 1 means that
    /// the heightmap has the lowest possible height given by
    /// [`DimensionType::min_y`]. Note that blocks cannot be placed at `min_y -
    /// 1`.
    ///
    /// [`DimensionType::min_y`]: valence_registry::dimension_type::DimensionType::min_y
    pub(crate) fn build_heightmap(
        &self,
        mut is_occupied: impl FnMut(BlockState) -> bool,
    ) -> [u32; 16 * 16] {
        let mut heightmap = [0; 16 * 16];

        for z in 0_u32..16 {
            for x in 0_u32..16 {
                for y in (0..self.height()).rev() {
                    if is_occupied(self.block_state(x, y, z)) {
                        // Heightmap values are 1-indexed local Y coordinates, where 0
                        // means "no occupied block in this column".
                        heightmap[(z as usize) * 16 + (x as usize)] = y + 1;
                        break;
                    }
                }
            }
        }

        heightmap
    }

    fn is_motion_blocking_occupied(state: BlockState) -> bool {
        let kind = state.to_kind();

        if matches!(kind, BlockKind::BambooSapling | BlockKind::Cactus) {
            return false;
        }

        state.blocks_motion()
            || state.is_liquid()
            || state.get(PropName::Waterlogged) == Some(PropValue::True)
    }

    fn is_motion_blocking_no_leaves_occupied(state: BlockState) -> bool {
        if Self::is_leaf_block(state) {
            return false;
        }

        Self::is_motion_blocking_occupied(state)
    }

    fn is_leaf_block(state: BlockState) -> bool {
        state.to_kind().to_str().ends_with("_leaves")
    }

    /// Encodes a given heightmap into the packed long-array format used in
    /// `LevelChunkWithLightS2c`.
    fn encode_heightmap(heightmap: &[u32; 16 * 16], world_height: u32) -> Vec<i64> {
        let bits_per_entry = (u32::BITS - world_height.leading_zeros()).max(1);
        let entries_per_long = i64::BITS / bits_per_entry;
        let longs_per_packet =
            (16 * 16) / entries_per_long + u32::from((16 * 16) % entries_per_long != 0);

        let mut data: Vec<i64> = vec![0; longs_per_packet as usize];

        for (idx, y) in heightmap.iter().enumerate() {
            debug_assert!(*y <= world_height);

            let long_idx = idx / entries_per_long as usize;
            let bit_offset = (idx % entries_per_long as usize) as u32 * bits_per_entry;
            data[long_idx] |= i64::from(*y) << bit_offset;
        }

        data
    }

    fn fill_light_data(
        light: &LightSection,
        light_arrays: &mut Vec<FixedArray<u8, 2048>>,
        light_mask: &mut VariableBitSet,
        empty_light_mask: &mut VariableBitSet,
        i: usize,
        is_block_light: bool,
    ) {
        match light {
            LightSection::NotSet => {
                // For sky light, the client will deduce this section to be either fully lit or
                // fully dark based on the presence of light data in other light sections in the
                // chunk.
                if is_block_light {
                    empty_light_mask.set(i);
                }
            }
            LightSection::FullyDark => {
                empty_light_mask.set(i);
            }
            LightSection::FullData(data) => {
                light_arrays.push(FixedArray(**data));
                light_mask.set(i);
            }
        }
    }

    /// Writes the packet data needed to initialize this chunk.
    pub(crate) fn write_init_packets(
        &self,
        mut writer: impl WritePacket,
        pos: ChunkPos,
        info: &ChunkLayerInfo,
    ) {
        let mut init_packets = self.cached_init_packets.lock();

        if init_packets.is_empty() {
            let world_surface = self.world_surface();
            let motion_blocking = self.motion_blocking();
            let motion_blocking_no_leaves = self.motion_blocking_no_leaves();
            let world_height = self.height();

            let heightmaps = vec![
                HeightMap {
                    kind: HeightMapKind::WorldSurface,
                    data: LoadedChunk::encode_heightmap(&world_surface, world_height),
                },
                HeightMap {
                    kind: HeightMapKind::MotionBlocking,
                    data: LoadedChunk::encode_heightmap(&motion_blocking, world_height),
                },
                HeightMap {
                    kind: HeightMapKind::MotionBlockingNoLeaves,
                    data: LoadedChunk::encode_heightmap(&motion_blocking_no_leaves, world_height),
                },
            ];

            let mut blocks_and_biomes: Vec<u8> = vec![];

            let light_section_count = self.sections.len() + 2;

            let mut sky_light_mask = VariableBitSet::default();
            let mut empty_sky_light_mask = VariableBitSet::default();
            let mut block_light_mask = VariableBitSet::default();
            let mut empty_block_light_mask = VariableBitSet::default();

            let mut sky_light_arrays = Vec::with_capacity(light_section_count);
            let mut block_light_arrays = Vec::with_capacity(light_section_count);

            for (i, sky_light) in self.sky_light_sections.iter().enumerate() {
                LoadedChunk::fill_light_data(
                    sky_light,
                    &mut sky_light_arrays,
                    &mut sky_light_mask,
                    &mut empty_sky_light_mask,
                    i,
                    false,
                );
            }

            for (i, block_light) in self.block_light_sections.iter().enumerate() {
                LoadedChunk::fill_light_data(
                    block_light,
                    &mut block_light_arrays,
                    &mut block_light_mask,
                    &mut empty_block_light_mask,
                    i,
                    true,
                );
            }

            for sect in &self.sections {
                sect.count_non_air_blocks()
                    .encode(&mut blocks_and_biomes)
                    .unwrap();

                sect.block_states
                    .encode_mc_format(
                        &mut blocks_and_biomes,
                        |b| b.to_raw().into(),
                        4,
                        8,
                        bit_width(BlockState::max_raw().into()),
                    )
                    .expect("paletted container encode should always succeed");

                sect.biomes
                    .encode_mc_format(
                        &mut blocks_and_biomes,
                        |b| b.to_index() as u64,
                        0,
                        3,
                        bit_width(info.biome_registry_len - 1),
                    )
                    .expect("paletted container encode should always succeed");
            }

            let block_entities: Vec<_> = self
                .block_entities
                .iter()
                .filter_map(|(&idx, nbt)| {
                    let x = idx % 16;
                    let z = idx / 16 % 16;
                    let y = idx / 16 / 16;

                    let kind = self.sections[y as usize / 16]
                        .block_states
                        .get(idx as usize % SECTION_BLOCK_COUNT)
                        .block_entity_kind();

                    kind.map(|kind| ChunkDataBlockEntity {
                        packed_xz: ((x << 4) | z) as i8,
                        y: y as i16 + info.min_y as i16,
                        kind,
                        data: Cow::Borrowed(nbt),
                    })
                })
                .collect();

            PacketWriter::new(&mut init_packets, info.threshold).write_packet(
                &LevelChunkWithLightS2c {
                    pos,
                    heightmaps: Cow::Owned(heightmaps),
                    blocks_and_biomes: &blocks_and_biomes,
                    block_entities: Cow::Owned(block_entities),
                    sky_light_mask: Cow::Borrowed(&sky_light_mask),
                    block_light_mask: Cow::Borrowed(&block_light_mask),
                    empty_sky_light_mask: Cow::Borrowed(&empty_sky_light_mask),
                    empty_block_light_mask: Cow::Borrowed(&empty_block_light_mask),
                    sky_light_arrays: Cow::Borrowed(&sky_light_arrays),
                    block_light_arrays: Cow::Borrowed(&block_light_arrays),
                },
            )
        }

        writer.write_packet_bytes(&init_packets);
    }

    /// Asserts that no changes to this chunk are currently recorded.
    #[track_caller]
    fn assert_no_changes(&self) {
        #[cfg(debug_assertions)]
        {
            assert!(!self.changed_biomes);
            assert!(self.changed_block_entities.is_empty());

            for sect in &self.sections {
                assert!(sect.updates.is_empty());
            }
        }
    }
}

impl Chunk for LoadedChunk {
    fn height(&self) -> u32 {
        self.sections.len() as u32 * 16
    }

    fn block_state(&self, x: u32, y: u32, z: u32) -> BlockState {
        check_block_oob(self, x, y, z);

        let idx = x + z * 16 + y % 16 * 16 * 16;
        self.sections[y as usize / 16]
            .block_states
            .get(idx as usize)
    }

    fn set_block_state(&mut self, x: u32, y: u32, z: u32, block: BlockState) -> BlockState {
        check_block_oob(self, x, y, z);

        let sect_y = y / 16;
        let sect = &mut self.sections[sect_y as usize];
        let idx = x + z * 16 + y % 16 * 16 * 16;

        let old_block = sect.block_states.set(idx as usize, block);

        if block != old_block {
            self.cached_init_packets.get_mut().clear();

            if *self.viewer_count.get_mut() > 0 {
                sect.updates.push(
                    ChunkDeltaUpdateEntry::new()
                        .with_off_x(x as u8)
                        .with_off_y((y % 16) as u8)
                        .with_off_z(z as u8)
                        .with_block_state(block.to_raw().into()),
                );
            }
        }

        old_block
    }

    fn fill_block_state_section(&mut self, sect_y: u32, block: BlockState) {
        check_section_oob(self, sect_y);

        let sect = &mut self.sections[sect_y as usize];

        if let PalettedContainer::Single(b) = &sect.block_states {
            if *b != block {
                self.cached_init_packets.get_mut().clear();

                if *self.viewer_count.get_mut() > 0 {
                    // The whole section is being modified, so any previous modifications would
                    // be overwritten.
                    sect.updates.clear();

                    // Push section updates for all the blocks in the section.
                    sect.updates.reserve_exact(SECTION_BLOCK_COUNT);
                    for z in 0..16 {
                        for x in 0..16 {
                            for y in 0..16 {
                                sect.updates.push(
                                    ChunkDeltaUpdateEntry::new()
                                        .with_off_x(x)
                                        .with_off_y(y)
                                        .with_off_z(z)
                                        .with_block_state(block.to_raw().into()),
                                );
                            }
                        }
                    }
                }
            }
        } else {
            for z in 0..16 {
                for x in 0..16 {
                    for y in 0..16 {
                        let idx = x + z * 16 + (sect_y * 16 + y) * (16 * 16);

                        if block != sect.block_states.get(idx as usize) {
                            self.cached_init_packets.get_mut().clear();

                            if *self.viewer_count.get_mut() > 0 {
                                sect.updates.push(
                                    ChunkDeltaUpdateEntry::new()
                                        .with_off_x(x as u8)
                                        .with_off_y(y as u8)
                                        .with_off_z(z as u8)
                                        .with_block_state(block.to_raw().into()),
                                );
                            }
                        }
                    }
                }
            }
        }

        sect.block_states.fill(block);
    }

    fn block_entity(&self, x: u32, y: u32, z: u32) -> Option<&Compound> {
        check_block_oob(self, x, y, z);

        let idx = x + z * 16 + y * 16 * 16;
        self.block_entities.get(&idx)
    }

    fn block_entity_mut(&mut self, x: u32, y: u32, z: u32) -> Option<&mut Compound> {
        check_block_oob(self, x, y, z);

        let idx = x + z * 16 + y * 16 * 16;

        if let Some(be) = self.block_entities.get_mut(&idx) {
            if *self.viewer_count.get_mut() > 0 {
                self.changed_block_entities.insert(idx);
            }
            self.cached_init_packets.get_mut().clear();

            Some(be)
        } else {
            None
        }
    }

    fn set_block_entity(
        &mut self,
        x: u32,
        y: u32,
        z: u32,
        block_entity: Option<Compound>,
    ) -> Option<Compound> {
        check_block_oob(self, x, y, z);

        let idx = x + z * 16 + y * 16 * 16;

        match block_entity {
            Some(nbt) => {
                if *self.viewer_count.get_mut() > 0 {
                    self.changed_block_entities.insert(idx);
                }
                self.cached_init_packets.get_mut().clear();

                self.block_entities.insert(idx, nbt)
            }
            None => {
                let res = self.block_entities.remove(&idx);

                if res.is_some() {
                    self.cached_init_packets.get_mut().clear();
                }

                res
            }
        }
    }

    fn clear_block_entities(&mut self) {
        if self.block_entities.is_empty() {
            return;
        }

        self.cached_init_packets.get_mut().clear();

        if *self.viewer_count.get_mut() > 0 {
            self.changed_block_entities
                .extend(mem::take(&mut self.block_entities).into_keys());
        } else {
            self.block_entities.clear();
        }
    }

    fn biome(&self, x: u32, y: u32, z: u32) -> BiomeId {
        check_biome_oob(self, x, y, z);

        let idx = x + z * 4 + y % 4 * 4 * 4;
        self.sections[y as usize / 4].biomes.get(idx as usize)
    }

    fn set_biome(&mut self, x: u32, y: u32, z: u32, biome: BiomeId) -> BiomeId {
        check_biome_oob(self, x, y, z);

        let idx = x + z * 4 + y % 4 * 4 * 4;
        let old_biome = self.sections[y as usize / 4]
            .biomes
            .set(idx as usize, biome);

        if biome != old_biome {
            self.cached_init_packets.get_mut().clear();

            if *self.viewer_count.get_mut() > 0 {
                self.changed_biomes = true;
            }
        }

        old_biome
    }

    fn fill_biome_section(&mut self, sect_y: u32, biome: BiomeId) {
        check_section_oob(self, sect_y);

        let sect = &mut self.sections[sect_y as usize];

        if let PalettedContainer::Single(b) = &sect.biomes {
            if *b != biome {
                self.cached_init_packets.get_mut().clear();
                self.changed_biomes = *self.viewer_count.get_mut() > 0;
            }
        } else {
            self.cached_init_packets.get_mut().clear();
            self.changed_biomes = *self.viewer_count.get_mut() > 0;
        }

        sect.biomes.fill(biome);
    }

    fn shrink_to_fit(&mut self) {
        self.cached_init_packets.get_mut().shrink_to_fit();

        for sect in &mut self.sections {
            sect.block_states.shrink_to_fit();
            sect.biomes.shrink_to_fit();
            sect.updates.shrink_to_fit();
        }
    }
}

#[cfg(test)]
mod tests {
    use valence_nbt::compound;
    use valence_protocol::CompressionThreshold;
    use valence_registry::dimension_type::DimensionTypeId;

    use super::*;

    fn heightmap_idx(x: usize, z: usize) -> usize {
        z * 16 + x
    }

    fn decode_heightmap(data: &[i64], bits_per_entry: u32) -> [u32; 16 * 16] {
        let entries_per_long = i64::BITS / bits_per_entry;
        let mask = (1_u64 << bits_per_entry) - 1;
        let mut decoded = [0; 16 * 16];

        for (idx, value) in decoded.iter_mut().enumerate() {
            let long_idx = idx / entries_per_long as usize;
            let bit_offset = (idx % entries_per_long as usize) as u32 * bits_per_entry;
            *value = ((data[long_idx] as u64 >> bit_offset) & mask) as u32;
        }

        decoded
    }

    #[test]
    fn loaded_chunk_unviewed_no_changes() {
        let mut chunk = LoadedChunk::new(512);

        chunk.set_block(0, 10, 0, BlockState::MAGMA_BLOCK);
        chunk.assert_no_changes();

        chunk.set_biome(0, 0, 0, BiomeId::from_index(5));
        chunk.assert_no_changes();

        chunk.fill_block_states(BlockState::ACACIA_BUTTON);
        chunk.assert_no_changes();

        chunk.fill_biomes(BiomeId::from_index(42));
        chunk.assert_no_changes();
    }

    #[test]
    fn loaded_chunk_changes_clear_packet_cache() {
        #[track_caller]
        fn check<T>(chunk: &mut LoadedChunk, change: impl FnOnce(&mut LoadedChunk) -> T) {
            let info = ChunkLayerInfo {
                dimension_type: DimensionTypeId::new(0),
                height: 512,
                min_y: -16,
                biome_registry_len: 200,
                threshold: CompressionThreshold(-1),
            };

            let mut buf = vec![];
            let mut writer = PacketWriter::new(&mut buf, CompressionThreshold(-1));

            // Rebuild cache.
            chunk.write_init_packets(&mut writer, ChunkPos::new(3, 4), &info);

            // Check that the cache is built.
            assert!(!chunk.cached_init_packets.get_mut().is_empty());

            // Making a change should clear the cache.
            change(chunk);
            assert!(chunk.cached_init_packets.get_mut().is_empty());

            // Rebuild cache again.
            chunk.write_init_packets(&mut writer, ChunkPos::new(3, 4), &info);
            assert!(!chunk.cached_init_packets.get_mut().is_empty());
        }

        let mut chunk = LoadedChunk::new(512);

        check(&mut chunk, |c| {
            c.set_block_state(0, 4, 0, BlockState::ACACIA_WOOD)
        });
        check(&mut chunk, |c| c.set_biome(1, 2, 3, BiomeId::from_index(4)));
        check(&mut chunk, |c| c.fill_biomes(BiomeId::DEFAULT));
        check(&mut chunk, |c| c.fill_block_states(BlockState::WET_SPONGE));
        check(&mut chunk, |c| {
            c.set_block_entity(3, 40, 5, Some(compound! {}))
        });
        check(&mut chunk, |c| {
            c.block_entity_mut(3, 40, 5).unwrap();
        });
        check(&mut chunk, |c| c.set_block_entity(3, 40, 5, None));

        // Old block state is the same as new block state, so the cache should still be
        // intact.
        assert_eq!(
            chunk.set_block_state(0, 0, 0, BlockState::WET_SPONGE),
            BlockState::WET_SPONGE
        );

        assert!(!chunk.cached_init_packets.get_mut().is_empty());
    }

    #[test]
    fn heightmap_occupancy_rules() {
        // Based on: https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Heightmap_structure
        let mut chunk = LoadedChunk::new(32);

        chunk.set_block_state(0, 0, 0, BlockState::STONE);
        chunk.set_block_state(1, 5, 0, BlockState::OAK_LEAVES);
        chunk.set_block_state(2, 6, 0, BlockState::CACTUS);
        chunk.set_block_state(3, 7, 0, BlockState::WATER);
        chunk.set_block_state(
            4,
            8,
            0,
            BlockState::OAK_LEAVES.set(PropName::Waterlogged, PropValue::True),
        );

        let world_surface = chunk.world_surface();
        let motion_blocking = chunk.motion_blocking();
        let motion_blocking_no_leaves = chunk.motion_blocking_no_leaves();

        assert_eq!(world_surface[heightmap_idx(0, 0)], 1);
        assert_eq!(world_surface[heightmap_idx(1, 0)], 6);
        assert_eq!(world_surface[heightmap_idx(2, 0)], 7);
        assert_eq!(world_surface[heightmap_idx(3, 0)], 8);
        assert_eq!(world_surface[heightmap_idx(4, 0)], 9);

        assert_eq!(motion_blocking[heightmap_idx(0, 0)], 1);
        assert_eq!(motion_blocking[heightmap_idx(1, 0)], 6);
        assert_eq!(motion_blocking[heightmap_idx(2, 0)], 0);
        assert_eq!(motion_blocking[heightmap_idx(3, 0)], 8);
        assert_eq!(motion_blocking[heightmap_idx(4, 0)], 9);

        assert_eq!(motion_blocking_no_leaves[heightmap_idx(0, 0)], 1);
        assert_eq!(motion_blocking_no_leaves[heightmap_idx(1, 0)], 0);
        assert_eq!(motion_blocking_no_leaves[heightmap_idx(2, 0)], 0);
        assert_eq!(motion_blocking_no_leaves[heightmap_idx(3, 0)], 8);
        assert_eq!(motion_blocking_no_leaves[heightmap_idx(4, 0)], 0);
    }

    #[test]
    fn encode_heightmap_uses_dynamic_bit_width() {
        let mut chunk = LoadedChunk::new(512);
        chunk.set_block_state(0, 511, 0, BlockState::STONE);

        let motion_blocking = chunk.motion_blocking();
        assert_eq!(motion_blocking[heightmap_idx(0, 0)], 512);

        let encoded = LoadedChunk::encode_heightmap(&motion_blocking, chunk.height());
        // 512 world height => ceil(log2(512 + 1)) = 10 bits, so 64/10 = 6 entries per
        // long.
        assert_eq!(encoded.len(), 43);

        let decoded = decode_heightmap(&encoded, 10);
        assert_eq!(decoded[heightmap_idx(0, 0)], 512);
        assert_eq!(decoded[heightmap_idx(1, 0)], 0);
    }
}
