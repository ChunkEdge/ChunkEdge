//! Handles new connections to the server and the log-in process.

use std::borrow::Cow;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{bail, ensure, Context};
use base64::prelude::*;
use chunkedge_binary::{Bounded, Decode, RawBytes};
use chunkedge_lang::keys;
use chunkedge_protocol::packets::configuration::select_known_packs_s2c::KnownPack;
use chunkedge_protocol::packets::configuration::{
    ClientInformationC2s, CustomPayloadC2s, CustomPayloadS2c, FinishConfigurationC2s,
    FinishConfigurationS2c, RegistryDataS2c, SelectKnownPacksC2s, SelectKnownPacksS2c,
    UpdateEnabledFeaturesS2c, UpdateTagsS2c,
};
use chunkedge_protocol::packets::login::{LoginAcknowledgedC2s, LoginFinishedS2c};
use chunkedge_protocol::packets::status::{
    PingRequestC2s, PongResponseS2c, StatusRequestC2s, StatusResponseS2c,
};
use chunkedge_protocol::profile::Property;
use chunkedge_protocol::JsonText;
use chunkedge_server::client::Properties;
use chunkedge_server::nbt::serde::ser::CompoundSerializer;
use chunkedge_server::protocol::packets::handshake::intention_c2s::HandShakeIntent;
use chunkedge_server::protocol::packets::handshake::IntentionC2s;
use chunkedge_server::protocol::packets::login::{
    CustomQueryAnswerC2s, CustomQueryS2c, HelloC2s, HelloS2c, KeyC2s, LoginCompressionS2c,
    LoginDisconnectS2c,
};
use chunkedge_server::protocol::{PacketDecoder, PacketEncoder, VarInt};
use chunkedge_server::registry::{BiomeRegistry, DimensionTypeRegistry, RegistryCodec};
use chunkedge_server::text::{Color, IntoText};
use chunkedge_server::{ident, Ident, Text, MINECRAFT_VERSION, PROTOCOL_VERSION};
use hmac::digest::Update;
use hmac::{Hmac, KeyInit, Mac};
use num_bigint::BigInt;
use reqwest::StatusCode;
use rsa::Pkcs1v15Encrypt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, trace, warn};
use uuid::Uuid;

use crate::legacy_ping::try_handle_legacy_ping;
use crate::packet_io::PacketIo;
use crate::{
    CleanupOnDrop, ConnectionMode, NewClientInfo, ServerListPing, SharedNetworkState,
    WorldLoginState,
};

const VELOCITY_MIN_MAX_SUPPORTED_VERSION: u8 = 3;

/// Accepts new connections to the server as they occur.
pub(super) async fn do_accept_loop(shared: SharedNetworkState, world_state: WorldLoginState) {
    let listener = match TcpListener::bind(shared.0.address).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("failed to start TCP listener: {e}");
            return;
        }
    };

    let timeout = Duration::from_secs(5);

    loop {
        let world_state = world_state.clone();
        match shared.0.connection_sema.clone().acquire_owned().await {
            Ok(permit) => match listener.accept().await {
                Ok((stream, remote_addr)) => {
                    let shared = shared.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tokio::time::timeout(
                            timeout,
                            handle_connection(shared, stream, remote_addr, world_state),
                        )
                        .await
                        {
                            warn!("initial connection timed out: {e}");
                        }

                        drop(permit);
                    });
                }
                Err(e) => {
                    error!("failed to accept incoming connection: {e}");
                }
            },
            // Closed semaphore indicates server shutdown.
            Err(_) => return,
        }
    }
}

async fn handle_connection(
    shared: SharedNetworkState,
    mut stream: TcpStream,
    remote_addr: SocketAddr,
    world_state: WorldLoginState,
) {
    trace!("handling connection");

    if let Err(e) = stream.set_nodelay(true) {
        error!("failed to set TCP_NODELAY: {e}");
    }

    match try_handle_legacy_ping(&shared, &mut stream, remote_addr).await {
        Ok(true) => return, // Legacy ping succeeded.
        Ok(false) => {}     // No legacy ping.
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {}
        Err(e) => {
            warn!("legacy ping ended with error: {e:#}");
        }
    }

    let io = PacketIo::new(stream, PacketEncoder::new(), PacketDecoder::new());

    if let Err(e) = handle_handshake(shared, io, remote_addr, world_state).await {
        // EOF can happen if the client disconnects while joining, which isn't
        // very erroneous.
        if let Some(e) = e.downcast_ref::<io::Error>() {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return;
            }
        }
        warn!("connection ended with error: {e:#}");
    }
}

/// Basic information about a client, provided at the beginning of the
/// connection
#[derive(Default, Debug)]
pub struct HandshakeData {
    /// The protocol version of the client.
    pub protocol_version: i32,
    /// The address that the client used to connect.
    pub server_address: String,
    /// The port that the client used to connect.
    pub server_port: u16,
}

async fn handle_handshake(
    shared: SharedNetworkState,
    mut io: PacketIo,
    remote_addr: SocketAddr,
    world_state: WorldLoginState,
) -> anyhow::Result<()> {
    let handshake = io.recv_packet::<IntentionC2s>().await?;

    let next_state = handshake.intent;

    let handshake = HandshakeData {
        protocol_version: handshake.protocol_version.0,
        server_address: handshake.server_address.0.to_owned(),
        server_port: handshake.server_port,
    };

    // TODO: this is borked.
    ensure!(
        shared.0.connection_mode == ConnectionMode::BungeeCord
            || handshake.server_address.encode_utf16().count() <= 255,
        "handshake server address is too long"
    );

    match next_state {
        HandShakeIntent::Status => handle_status(shared, io, remote_addr, handshake)
            .await
            .context("handling status"),
        HandShakeIntent::Login => {
            match handle_login(&shared, &mut io, remote_addr, handshake, world_state)
                .await
                .context("handling login")?
            {
                Some((info, cleanup)) => {
                    let client = io.into_client_args(
                        info,
                        shared.0.incoming_byte_limit,
                        shared.0.outgoing_byte_limit,
                        cleanup,
                    );

                    let _ = shared.0.new_clients_send.send_async(client).await;

                    Ok(())
                }
                None => Ok(()),
            }
        }
        HandShakeIntent::Transfer => {
            // TODO: Implement
            bail!("transfer state is not yet implemented");
        }
    }
}

async fn handle_status(
    shared: SharedNetworkState,
    mut io: PacketIo,
    remote_addr: SocketAddr,
    handshake: HandshakeData,
) -> anyhow::Result<()> {
    io.recv_packet::<StatusRequestC2s>().await?;

    match shared
        .0
        .callbacks
        .inner
        .server_list_ping(&shared, remote_addr, &handshake)
        .await
    {
        ServerListPing::Respond {
            online_players,
            max_players,
            player_sample,
            mut description,
            favicon_png,
            version_name,
            protocol,
        } => {
            // For pre-1.16 clients, replace all webcolors with their closest
            // normal colors Because webcolor support was only
            // added at 1.16.
            if handshake.protocol_version < 735 {
                fn fallback_webcolors(txt: &mut Text) {
                    if let Some(Color::Rgb(color)) = txt.color {
                        txt.color = Some(Color::Named(color.to_named_lossy()));
                    }
                    for child in &mut txt.extra {
                        fallback_webcolors(child);
                    }
                }

                fallback_webcolors(&mut description);
            }

            let mut json = json!({
                "version": {
                    "name": version_name,
                    "protocol": protocol,
                },
                "players": {
                    "online": online_players,
                    "max": max_players,
                    "sample": player_sample,
                },
                "description": description,
            });

            if !favicon_png.is_empty() {
                let mut buf = "data:image/png;base64,".to_owned();
                BASE64_STANDARD.encode_string(favicon_png, &mut buf);
                json["favicon"] = Value::String(buf);
            }

            io.send_packet(&StatusResponseS2c {
                json: &json.to_string(),
            })
            .await?;
        }
        ServerListPing::Ignore => return Ok(()),
    }

    let PingRequestC2s { timestamp: payload } = io.recv_packet().await?;

    io.send_packet(&PongResponseS2c { timestamp: payload })
        .await?;

    Ok(())
}

/// Handle the login process and return the new client's data if successful.
async fn handle_login(
    shared: &SharedNetworkState,
    io: &mut PacketIo,
    remote_addr: SocketAddr,
    handshake: HandshakeData,
    world_state: WorldLoginState,
) -> anyhow::Result<Option<(NewClientInfo, CleanupOnDrop)>> {
    if handshake.protocol_version != PROTOCOL_VERSION {
        io.send_packet(&LoginDisconnectS2c {
            // TODO: use correct translation key.
            reason: Cow::Owned(JsonText(
                format!("Mismatched Minecraft version (server is on {MINECRAFT_VERSION})")
                    .color(Color::RED),
            )),
        })
        .await?;

        return Ok(None);
    }

    let HelloC2s {
        username,
        .. // TODO: profile_id
    } = io.recv_packet().await?;

    let username = username.0.to_owned();

    let mut info = match shared.connection_mode() {
        ConnectionMode::Online { .. } => login_online(shared, io, remote_addr, username).await?,
        ConnectionMode::Offline => login_offline(remote_addr, username)?,
        ConnectionMode::BungeeCord => {
            login_bungeecord(remote_addr, &handshake.server_address, username)?
        }
        ConnectionMode::Velocity { secret } => login_velocity(io, username, secret).await?,
    };

    if shared.0.threshold.0 > 0 {
        io.send_packet(&LoginCompressionS2c {
            threshold: shared.0.threshold.0.into(),
        })
        .await?;

        io.set_compression(shared.0.threshold);
    }

    let cleanup = match shared.0.callbacks.inner.login(shared, &info).await {
        Ok(f) => CleanupOnDrop(Some(f)),
        Err(reason) => {
            info!("disconnect at login: \"{reason}\"");
            io.send_packet(&LoginDisconnectS2c {
                reason: Cow::Owned(JsonText(reason)),
            })
            .await?;
            return Ok(None);
        }
    };

    io.send_packet(&LoginFinishedS2c {
        uuid: info.uuid,
        username: info.username.as_str().into(),
        properties: Default::default(),
    })
    .await?;

    let LoginAcknowledgedC2s {} = io.recv_packet().await?;
    if !matches!(shared.connection_mode(), ConnectionMode::Velocity { .. }) {
        let _: CustomPayloadC2s = io.recv_packet().await?;
    }
    let client_info: ClientInformationC2s = io.recv_packet().await?;

    info.view_distance = client_info.view_distance;
    info.locale = client_info.locale.0.to_owned();
    info.chat_mode = client_info.chat_mode;
    info.chat_colors = client_info.chat_colors;
    info.displayed_skin_parts = client_info.displayed_skin_parts;
    info.main_arm = client_info.main_arm;
    info.enable_text_filtering = client_info.enable_text_filtering;
    info.allow_server_listings = client_info.allow_server_listings;
    info.particle_mode = client_info.particle_mode;

    io.send_packet(&CustomPayloadS2c {
        channel: Ident::new("minecraft:brand").unwrap(),
        data: Bounded(RawBytes(&[&[0x07], "vanilla".as_bytes()].concat())),
    })
    .await?;

    io.send_packet(&UpdateEnabledFeaturesS2c {
        features: vec![ident!("minecraft:vanilla").into()],
    })
    .await?;

    io.send_packet(&SelectKnownPacksS2c {
        packs: vec![KnownPack {
            namespace: "minecraft".into(),
            id: "core".into(),
            version: MINECRAFT_VERSION.into(),
        }],
    })
    .await?;

    let _: SelectKnownPacksC2s = io.recv_packet().await?;

    // We have chunkedge support for the `worldgen/biome` and `dimension_type`
    // registries, therefore we use the current state of these registries here
    // (instead of the default values) This means the server can add/remove
    // biomes and dimensions at runtime.

    // BiomeRegistry
    io.send_packet(&RegistryDataS2c {
        id: BiomeRegistry::KEY.into(),
        entries: world_state
            .biome_registry
            .iter()
            .map(|(_, biome_ident, biome)| {
                (
                    biome_ident.into(),
                    Some(
                        biome
                            .serialize(CompoundSerializer)
                            .expect("failed to serialize biome"),
                    ),
                )
            })
            .collect(),
    })
    .await?;

    // DimensionTypeRegistry
    io.send_packet(&RegistryDataS2c {
        id: DimensionTypeRegistry::KEY.into(),
        entries: world_state
            .dimension_registry
            .iter()
            .map(|(_, dimension_ident, dimension_type)| {
                (
                    dimension_ident.into(),
                    Some(
                        dimension_type
                            .serialize(CompoundSerializer)
                            .expect("failed to serialize dimension type"),
                    ),
                )
            })
            .collect(),
    })
    .await?;

    // Send all other registries.
    //
    // Even if the remote end acknowledges the vanilla known pack, send the full
    // element data. Some protocol translators forward registry entries to newer
    // clients, and omitted entries force the client to resolve them from local
    // resources for the server's pack version.
    for (id, entries) in RegistryCodec::default().registries {
        if id == ident!("worldgen/biome") || id == ident!("dimension_type") {
            // We already sent these registries.
            continue;
        }

        io.send_packet(&RegistryDataS2c {
            id: id.into(),
            entries: entries
                .into_iter()
                .map(|value| (value.name.into(), Some(value.element)))
                .collect(),
        })
        .await?;
    }

    // TagsRegistry
    io.send_packet(&UpdateTagsS2c {
        groups: Cow::Owned(world_state.tag_registry),
    })
    .await?;

    io.send_packet(&FinishConfigurationS2c {}).await?;

    if matches!(shared.connection_mode(), ConnectionMode::Velocity { .. }) {
        let _: CustomPayloadC2s = io.recv_packet().await?;
    }
    let _: FinishConfigurationC2s = io.recv_packet().await?;

    Ok(Some((info, cleanup)))
}

/// Login procedure for online mode.
async fn login_online(
    shared: &SharedNetworkState,
    io: &mut PacketIo,
    remote_addr: SocketAddr,
    username: String,
) -> anyhow::Result<NewClientInfo> {
    let my_verify_token: [u8; 16] = rand::random();

    io.send_packet(&HelloS2c {
        server_id: "".into(), // Always empty
        public_key: &shared.0.public_key_der,
        verify_token: &my_verify_token,
        should_authenticate: true,
    })
    .await?;

    let KeyC2s {
        shared_secret,
        verify_token: encrypted_verify_token,
    } = io.recv_packet().await?;

    let shared_secret = shared
        .0
        .rsa_key
        .decrypt(Pkcs1v15Encrypt, shared_secret)
        .context("failed to decrypt shared secret")?;

    let verify_token = shared
        .0
        .rsa_key
        .decrypt(Pkcs1v15Encrypt, encrypted_verify_token)
        .context("failed to decrypt verify token")?;

    ensure!(
        my_verify_token.as_slice() == verify_token,
        "verify tokens do not match"
    );

    let crypt_key: [u8; 16] = shared_secret
        .as_slice()
        .try_into()
        .context("shared secret has the wrong length")?;

    io.enable_encryption(&crypt_key);

    let hash = Sha1::new()
        .chain(&shared_secret)
        .chain(&shared.0.public_key_der)
        .finalize();

    let url = shared
        .0
        .callbacks
        .inner
        .session_server(
            shared,
            username.as_str(),
            &auth_digest(&hash),
            &remote_addr.ip(),
        )
        .await;

    let resp = shared.0.http_client.get(url).send().await?;

    match resp.status() {
        StatusCode::OK => {}
        StatusCode::NO_CONTENT => {
            let reason =
                Text::translate(keys::MULTIPLAYER_DISCONNECT_UNVERIFIED_USERNAME, [], None);
            io.send_packet(&LoginDisconnectS2c {
                reason: Cow::Owned(JsonText(reason)),
            })
            .await?;
            bail!("session server could not verify username");
        }
        status => {
            bail!("session server GET request failed (status code {status})");
        }
    }

    #[derive(Deserialize)]
    struct GameProfile {
        id: Uuid,
        name: String,
        properties: Vec<Property>,
    }

    let profile: GameProfile = resp.json().await.context("parsing game profile")?;

    ensure!(profile.name == username, "usernames do not match");

    Ok(NewClientInfo {
        uuid: profile.id,
        username,
        ip: remote_addr.ip(),
        properties: Properties(profile.properties),
        view_distance: 0, // Will be changed later.
        locale: String::new(),
        chat_mode: Default::default(),
        chat_colors: false,
        displayed_skin_parts: Default::default(),
        main_arm: Default::default(),
        enable_text_filtering: false,
        allow_server_listings: false,
        particle_mode: Default::default(),
    })
}

fn auth_digest(bytes: &[u8]) -> String {
    BigInt::from_signed_bytes_be(bytes).to_str_radix(16)
}

fn offline_uuid(username: &str) -> anyhow::Result<Uuid> {
    Uuid::from_slice(&Sha256::digest(username)[..16]).map_err(Into::into)
}

/// Login procedure for offline mode.
fn login_offline(remote_addr: SocketAddr, username: String) -> anyhow::Result<NewClientInfo> {
    Ok(NewClientInfo {
        // Derive the client's UUID from a hash of their username.
        uuid: offline_uuid(username.as_str())?,
        username,
        properties: Default::default(),
        ip: remote_addr.ip(),
        view_distance: 0, // Will be changed later.
        locale: String::new(),
        chat_mode: Default::default(),
        chat_colors: false,
        displayed_skin_parts: Default::default(),
        main_arm: Default::default(),
        enable_text_filtering: false,
        allow_server_listings: false,
        particle_mode: Default::default(),
    })
}

/// Login procedure for `BungeeCord`.
fn login_bungeecord(
    remote_addr: SocketAddr,
    server_address: &str,
    username: String,
) -> anyhow::Result<NewClientInfo> {
    // Get data from server_address field of the handshake
    let data = server_address.split('\0').take(4).collect::<Vec<_>>();

    // Ip of player, only given if ip_forward on bungee is true
    let ip = match data.get(1) {
        Some(ip) => ip.parse()?,
        None => remote_addr.ip(),
    };

    // Uuid of player, only given if ip_forward on bungee is true
    let uuid = match data.get(2) {
        Some(uuid) => uuid.parse()?,
        None => offline_uuid(username.as_str())?,
    };

    // Read properties and get textures
    // Properties of player's game profile, only given if ip_forward and online_mode
    // on bungee both are true
    let properties: Vec<Property> = match data.get(3) {
        Some(properties) => serde_json::from_str(properties)
            .context("failed to parse BungeeCord player properties")?,
        None => vec![],
    };

    Ok(NewClientInfo {
        uuid,
        username,
        properties: Properties(properties),
        ip,
        view_distance: 0, // Will be changed later.
        locale: String::new(),
        chat_mode: Default::default(),
        chat_colors: false,
        displayed_skin_parts: Default::default(),
        main_arm: Default::default(),
        enable_text_filtering: false,
        allow_server_listings: false,
        particle_mode: Default::default(),
    })
}

/// Login procedure for Velocity.
async fn login_velocity(
    io: &mut PacketIo,
    username: String,
    velocity_secret: &str,
) -> anyhow::Result<NewClientInfo> {
    let message_id: i32 = 0; // TODO: make this random?

    // Send Player Info Request into the Plugin Channel
    io.send_packet(&CustomQueryS2c {
        message_id: VarInt(message_id),
        channel: ident!("velocity:player_info").into(),
        data: RawBytes(&[VELOCITY_MIN_MAX_SUPPORTED_VERSION]).into(),
    })
    .await?;

    // Get Response
    let plugin_response: CustomQueryAnswerC2s = io.recv_packet().await?;

    ensure!(
        plugin_response.message_id.0 == message_id,
        "mismatched plugin response ID (got {}, expected {message_id})",
        plugin_response.message_id.0,
    );

    let data = plugin_response
        .data
        .context("missing plugin response data")?;
    let payload = data.0;

    parse_velocity_player_info(payload.0, username, velocity_secret)
}

fn parse_velocity_player_info(
    data: &[u8],
    username: String,
    velocity_secret: &str,
) -> anyhow::Result<NewClientInfo> {
    ensure!(data.len() >= 32, "invalid plugin response data length");
    let (signature, mut data_without_signature) = data.split_at(32);

    // Verify signature
    let mut mac = Hmac::<Sha256>::new_from_slice(velocity_secret.as_bytes())?;
    Mac::update(&mut mac, data_without_signature);
    mac.verify_slice(signature)?;

    // Check Velocity version
    let version = VarInt::decode(&mut data_without_signature)
        .context("failed to decode velocity version")?
        .0;

    ensure!(
        version != i32::from(VELOCITY_MIN_MAX_SUPPORTED_VERSION),
        "Client tried to connect with an unsupported Velocity version: {version}. While we only \
         support version {VELOCITY_MIN_MAX_SUPPORTED_VERSION}."
    );

    // Get client address
    let remote_addr = String::decode(&mut data_without_signature)?.parse()?;

    // Get UUID
    let uuid = Uuid::decode(&mut data_without_signature)?;

    // Get username and validate
    ensure!(
        username == <&str>::decode(&mut data_without_signature)?,
        "mismatched usernames"
    );

    // Read game profile properties
    let properties = Vec::<Property>::decode(&mut data_without_signature)
        .context("decoding velocity game profile properties")?;

    Ok(NewClientInfo {
        uuid,
        username,
        properties: Properties(properties),
        ip: remote_addr,
        view_distance: 0, // Will be changed later.
        locale: String::new(),
        chat_mode: Default::default(),
        chat_colors: false,
        displayed_skin_parts: Default::default(),
        main_arm: Default::default(),
        enable_text_filtering: false,
        allow_server_listings: false,
        particle_mode: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use sha1::Digest;

    use super::*;

    #[test]
    fn auth_digest_usernames() {
        assert_eq!(
            auth_digest(&Sha1::digest("Notch")),
            "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48"
        );
        assert_eq!(
            auth_digest(&Sha1::digest("jeb_")),
            "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1"
        );
        assert_eq!(
            auth_digest(&Sha1::digest("simon")),
            "88e16a1019277b15d58faf0541e11910eb756f6"
        );
    }
}
