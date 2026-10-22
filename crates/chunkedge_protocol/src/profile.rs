use base64::prelude::*;
use chunkedge_binary::{Bounded, Decode, Encode};
use serde::{Deserialize, Serialize};
use url::Url;

/// A property from the game profile.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Property<S = String> {
    pub name: S,
    pub value: S,
    pub signature: Option<S>,
}

impl<S: AsRef<str>> Encode for Property<S> {
    fn encode(&self, mut w: impl std::io::Write) -> anyhow::Result<()> {
        Bounded::<_, 64>(self.name.as_ref()).encode(&mut w)?;
        Bounded::<_, 32767>(self.value.as_ref()).encode(&mut w)?;
        self.signature
            .as_ref()
            .map(|s| Bounded::<_, 1024>(s.as_ref()))
            .encode(w)
    }
}

impl<'a, S: From<&'a str>> Decode<'a> for Property<S> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        Ok(Self {
            name: Bounded::<&str, 64>::decode(r)?.0.into(),
            value: Bounded::<&str, 32767>::decode(r)?.0.into(),
            signature: Option::<Bounded<&str, 1024>>::decode(r)?.map(|s| s.0.into()),
        })
    }
}

/// Contains URLs to the skin and cape of a player.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PlayerTextures {
    /// URL to the player's skin texture.
    pub skin: Url,
    /// URL to the player's cape texture. May be absent if the player does not
    /// have a cape.
    pub cape: Option<Url>,
}

impl PlayerTextures {
    /// Constructs player textures from the "textures" property of the game
    /// profile.
    ///
    /// "textures" is a base64 string of JSON data.
    pub fn try_from_textures(textures: &str) -> anyhow::Result<Self> {
        #[derive(Debug, Deserialize)]
        struct Textures {
            textures: PlayerTexturesPayload,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "UPPERCASE")]
        struct PlayerTexturesPayload {
            skin: TextureUrl,
            #[serde(default)]
            cape: Option<TextureUrl>,
        }

        #[derive(Debug, Deserialize)]
        struct TextureUrl {
            url: Url,
        }

        let decoded = BASE64_STANDARD.decode(textures.as_bytes())?;

        let Textures { textures } = serde_json::from_slice(&decoded)?;

        Ok(Self {
            skin: textures.skin.url,
            cape: textures.cape.map(|t| t.url),
        })
    }
}
