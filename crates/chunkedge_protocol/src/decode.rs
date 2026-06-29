#[cfg(feature = "encryption")]
use aes::cipher::KeyIvInit;
use anyhow::{bail, ensure, Context};
use bytes::{Buf, BytesMut};
use chunkedge_binary::{Decode, VarInt, VarIntDecodeError};

#[cfg(feature = "compression")]
use crate::CompressionThreshold;
use crate::{Packet, MAX_PACKET_SIZE};

/// The AES block cipher with a 128 bit key, using the CFB-8 mode of
/// operation.
#[cfg(feature = "encryption")]
type Cipher = cfb8::Decryptor<aes::Aes128>;

#[derive(Default)]
pub struct PacketDecoder {
    buf: BytesMut,
    #[cfg(feature = "compression")]
    decompress_buf: BytesMut,
    #[cfg(feature = "compression")]
    threshold: CompressionThreshold,
    #[cfg(feature = "encryption")]
    cipher: Option<Cipher>,
}

impl PacketDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_next_packet(&mut self) -> anyhow::Result<Option<PacketFrame>> {
        let mut r = &self.buf[..];

        let packet_len = match VarInt::decode_partial(&mut r) {
            Ok(len) => len,
            Err(VarIntDecodeError::Incomplete) => return Ok(None),
            Err(VarIntDecodeError::TooLarge) => bail!("malformed packet length VarInt"),
        };

        ensure!(
            (0..=MAX_PACKET_SIZE).contains(&packet_len),
            "packet length of {packet_len} is out of bounds"
        );

        if r.len() < packet_len as usize {
            // Not enough data arrived yet.
            return Ok(None);
        }

        let packet_len_len = self.buf.len() - r.len();

        let mut data;

        #[cfg(feature = "compression")]
        if self.threshold.0 >= 0 {
            use std::io::Write;

            use bytes::BufMut;
            use flate2::write::ZlibDecoder;

            r = &r[..packet_len as usize];

            let data_len = VarInt::decode(&mut r)?.0;

            ensure!(
                (0..MAX_PACKET_SIZE).contains(&data_len),
                "decompressed packet length of {data_len} is out of bounds"
            );

            // Is this packet compressed?
            if data_len > 0 {
                ensure!(
                    data_len >= self.threshold.0,
                    "decompressed packet length of {data_len} is below the compression threshold \
                     of {}",
                    self.threshold.0
                );

                debug_assert!(self.decompress_buf.is_empty());

                self.decompress_buf.put_bytes(0, data_len as usize);

                // TODO: use libdeflater or zune-inflate?
                let mut z = ZlibDecoder::new(&mut self.decompress_buf[..]);

                z.write_all(r)?;

                ensure!(
                    z.finish()?.is_empty(),
                    "decompressed packet length is shorter than expected"
                );

                self.buf.advance(packet_len_len + packet_len as usize);

                data = self.decompress_buf.split();
            } else {
                debug_assert_eq!(data_len, 0);

                ensure!(
                    r.len() < self.threshold.0 as usize,
                    "uncompressed packet length of {} is not below the compression threshold of {}",
                    r.len(),
                    self.threshold.0
                );

                let data_len_len = packet_len as usize - r.len();
                let remaining_len = r.len();

                self.buf.advance(packet_len_len + data_len_len);

                data = self.buf.split_to(remaining_len);
            }
        } else {
            self.buf.advance(packet_len_len);
            data = self.buf.split_to(packet_len as usize);
        }

        #[cfg(not(feature = "compression"))]
        {
            self.buf.advance(packet_len_len);
            data = self.buf.split_to(packet_len as usize);
        }

        // Decode the leading packet ID.
        r = &data[..];
        let packet_id = VarInt::decode(&mut r)
            .context("failed to decode packet ID")?
            .0;

        data.advance(data.len() - r.len());

        Ok(Some(PacketFrame {
            id: packet_id,
            body: data,
        }))
    }

    #[cfg(feature = "compression")]
    pub fn compression(&self) -> CompressionThreshold {
        self.threshold
    }

    #[cfg(feature = "compression")]
    pub fn set_compression(&mut self, threshold: CompressionThreshold) {
        self.threshold = threshold;
    }

    #[cfg(feature = "encryption")]
    pub fn enable_encryption(&mut self, key: &[u8; 16]) {
        assert!(self.cipher.is_none(), "encryption is already enabled");

        let mut cipher = Cipher::new_from_slices(key, key).expect("invalid key");

        // Don't forget to decrypt the data we already have.
        Self::decrypt_bytes(&mut cipher, &mut self.buf);

        self.cipher = Some(cipher);
    }

    /// Decrypts the provided byte slice in place using the cipher, without
    /// consuming the cipher.
    #[cfg(feature = "encryption")]
    fn decrypt_bytes(cipher: &mut Cipher, bytes: &mut [u8]) {
        cipher.decrypt(bytes);
    }

    pub fn queue_bytes(&mut self, mut bytes: BytesMut) {
        #![allow(unused_mut)]

        #[cfg(feature = "encryption")]
        if let Some(cipher) = &mut self.cipher {
            Self::decrypt_bytes(cipher, &mut bytes);
        }

        self.buf.unsplit(bytes);
    }

    pub fn queue_slice(&mut self, bytes: &[u8]) {
        #[cfg(feature = "encryption")]
        let len = self.buf.len();

        self.buf.extend_from_slice(bytes);

        #[cfg(feature = "encryption")]
        if let Some(cipher) = &mut self.cipher {
            let slice = &mut self.buf[len..];
            Self::decrypt_bytes(cipher, slice);
        }
    }

    pub fn take_capacity(&mut self) -> BytesMut {
        self.buf.split_off(self.buf.len())
    }

    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }
}

#[derive(Clone, Debug)]
pub struct PacketFrame {
    /// The ID of the decoded packet.
    pub id: i32,
    /// The contents of the packet after the leading `VarInt` ID.
    pub body: BytesMut,
}

impl PacketFrame {
    /// Attempts to decode this packet as type `P`. An error is returned if the
    /// packet ID does not match, the body of the packet failed to decode, or
    /// some input was missed.
    pub fn decode<'a, P>(&'a self) -> anyhow::Result<P>
    where
        P: Packet + Decode<'a>,
    {
        ensure!(
            P::ID == self.id,
            "packet ID mismatch while decoding '{}': expected {}, got {}",
            P::NAME,
            P::ID,
            self.id
        );

        let mut r = &self.body[..];

        #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
        crate::debug::enable_packet_recording(P::NAME, P::ID, r);

        let pkt_res = P::decode(&mut r).and_then(|pkt| {
            ensure!(
                r.is_empty(),
                "missed {} bytes while decoding '{}'",
                r.len(),
                P::NAME
            );
            Ok(pkt)
        });

        #[cfg(any(feature = "debug-packets", feature = "debug-packets-on-error"))]
        {
            let has_error = pkt_res.is_err();
            crate::debug::dump_packet_trace(has_error)
        };

        pkt_res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_padded_packet_length_and_packet_id() {
        let mut decoder = PacketDecoder::new();

        // packet_len = 2 encoded in two bytes, then packet_id = 0 encoded in two bytes.
        decoder.queue_slice(&[0x82, 0x00, 0x80, 0x00]);

        let frame = decoder.try_next_packet().unwrap().unwrap();

        assert_eq!(frame.id, 0);
        assert!(frame.body.is_empty());
        assert!(decoder.try_next_packet().unwrap().is_none());
    }

    #[cfg(feature = "compression")]
    #[test]
    fn accepts_padded_uncompressed_data_length_and_packet_id() {
        let mut decoder = PacketDecoder::new();
        decoder.set_compression(CompressionThreshold(256));

        // packet_len = 4 encoded in two bytes, then uncompressed data_len = 0 encoded
        // in two bytes, then packet_id = 0 encoded in two bytes.
        decoder.queue_slice(&[0x84, 0x00, 0x80, 0x00, 0x80, 0x00]);

        let frame = decoder.try_next_packet().unwrap().unwrap();

        assert_eq!(frame.id, 0);
        assert!(frame.body.is_empty());
        assert!(decoder.try_next_packet().unwrap().is_none());
    }

    #[cfg(feature = "compression")]
    #[test]
    fn accepts_compressed_packet_at_compression_threshold() {
        use std::io::Read;

        use chunkedge_binary::Encode;
        use flate2::bufread::ZlibEncoder;
        use flate2::Compression;

        let threshold = 3;
        let mut decoder = PacketDecoder::new();
        decoder.set_compression(CompressionThreshold(threshold));

        let uncompressed_packet = [0x00, 0xab, 0xcd];
        let mut compressed_packet = vec![];
        ZlibEncoder::new(&uncompressed_packet[..], Compression::new(4))
            .read_to_end(&mut compressed_packet)
            .unwrap();

        let packet_len = VarInt(threshold).written_size() + compressed_packet.len();
        let mut packet = vec![];
        VarInt(packet_len as i32).encode(&mut packet).unwrap();
        VarInt(threshold).encode(&mut packet).unwrap();
        packet.extend_from_slice(&compressed_packet);

        decoder.queue_slice(&packet);

        let frame = decoder.try_next_packet().unwrap().unwrap();

        assert_eq!(frame.id, 0);
        assert_eq!(&frame.body[..], [0xab, 0xcd]);
        assert!(decoder.try_next_packet().unwrap().is_none());
    }
}
