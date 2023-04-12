// Copyright (c) 2022 MASSA LABS <info@massa.net>

use crate::error::BootstrapError;
use crate::messages::{
    BootstrapClientMessage, BootstrapClientMessageSerializer, BootstrapServerMessage,
    BootstrapServerMessageDeserializer,
};
use crate::settings::BootstrapClientConfig;
use massa_hash::Hash;
use massa_models::serialization::{DeserializeMinBEInt, SerializeMinBEInt};
use massa_models::version::{Version, VersionSerializer};
use massa_serialization::{DeserializeError, Deserializer, Serializer};
use massa_signature::{PublicKey, Signature, SIGNATURE_SIZE_BYTES};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

/// Bootstrap client binder
pub struct BootstrapClientBinder {
    // max_bootstrap_message_size: u32,
    size_field_len: usize,
    remote_pubkey: PublicKey,
    duplex: TcpStream,
    prev_message: Option<Hash>,
    version_serializer: VersionSerializer,
    cfg: BootstrapClientConfig,
}

impl BootstrapClientBinder {
    /// Creates a new `WriteBinder`.
    ///
    /// # Argument
    /// * duplex: duplex stream.
    /// * limit: limit max bytes per second (up and down)
    #[allow(clippy::too_many_arguments)]
    pub fn new(duplex: TcpStream, remote_pubkey: PublicKey, cfg: BootstrapClientConfig) -> Self {
        let size_field_len = u32::be_bytes_min_length(cfg.max_bootstrap_message_size);
        BootstrapClientBinder {
            size_field_len,
            remote_pubkey,
            duplex,
            prev_message: None,
            version_serializer: VersionSerializer::new(),
            cfg,
        }
    }

    /// Performs a handshake. Should be called after connection
    /// NOT cancel-safe
    pub fn handshake(&mut self, version: Version) -> Result<(), BootstrapError> {
        // send version and randomn bytes
        let msg_hash = {
            let mut version_ser = Vec::new();
            self.version_serializer
                .serialize(&version, &mut version_ser)?;
            let mut version_random_bytes =
                vec![0u8; version_ser.len() + self.cfg.randomness_size_bytes];
            version_random_bytes[..version_ser.len()].clone_from_slice(&version_ser);
            StdRng::from_entropy().fill_bytes(&mut version_random_bytes[version_ser.len()..]);
            self.duplex.write_all(&version_random_bytes)?;
            Hash::compute_from(&version_random_bytes)
        };

        self.prev_message = Some(msg_hash);

        Ok(())
    }

    /// Reads the next message. NOT cancel-safe
    pub fn next_timeout(
        &mut self,
        duration: Option<Duration>,
    ) -> Result<BootstrapServerMessage, BootstrapError> {
        self.duplex.set_read_timeout(duration)?;

        // peek the signature and message len
        let peek_len = SIGNATURE_SIZE_BYTES + self.size_field_len;
        let mut peek_buff = vec![0u8; peek_len];
        // problematic if we can only peek some of it
        while self.duplex.peek(&mut peek_buff)? < peek_len {
            // TODO: Backoff spin of some sort
        }

        // construct the signature from the peek
        let sig_array = peek_buff.as_slice()[0..SIGNATURE_SIZE_BYTES]
            .try_into()
            .expect("logic error in array manipulations");
        let sig = Signature::from_bytes(&sig_array)?;

        // construct the message len from the peek
        let msg_len = u32::from_be_bytes_min(
            &peek_buff[SIGNATURE_SIZE_BYTES..],
            self.cfg.max_bootstrap_message_size,
        )?
        .0;

        // Update this bindings "most recently received" message hash, retaining the replaced value
        let message_deserializer = BootstrapServerMessageDeserializer::new((&self.cfg).into());
        let legacy_msg = self
            .prev_message
            .replace(Hash::compute_from(&sig.to_bytes()));

        let message = {
            if let Some(legacy_msg) = legacy_msg {
                // Consume the stream, and discard the peek
                let mut stream_bytes = vec![0u8; peek_len + (msg_len as usize)];
                // TODO: under the hood, this isn't actually atomic. For now, we use the ostrich algorithm.
                self.duplex.read_exact(&mut stream_bytes[..])?;
                let msg_bytes = &mut stream_bytes[peek_len..];

                // prepend the received message with the previous messages hash, and derive the new hash.
                // TODO: some sort of recovery if this fails?
                let rehash_seed = &[legacy_msg.to_bytes().as_slice(), msg_bytes].concat();
                let msg_hash = Hash::compute_from(rehash_seed);
                self.remote_pubkey.verify_signature(&msg_hash, &sig)?;

                // ...And deserialize
                let (_, msg) = message_deserializer
                    .deserialize::<DeserializeError>(&msg_bytes)
                    .map_err(|err| BootstrapError::DeserializeError(format!("{}", err)))?;
                msg
            } else {
                // Consume the stream and discard the peek
                let mut stream_bytes = vec![0u8; peek_len + msg_len as usize];
                // TODO: under the hood, this isn't actually atomic. For now, we use the ostrich algorithm.
                self.duplex.read_exact(&mut stream_bytes[..])?;
                let sig_msg_bytes = &mut stream_bytes[peek_len..];

                // Compute the hash and verify
                let msg_hash = Hash::compute_from(sig_msg_bytes);
                self.remote_pubkey.verify_signature(&msg_hash, &sig)?;

                // ...And deserialize
                let (_, msg) = message_deserializer
                    .deserialize::<DeserializeError>(sig_msg_bytes)
                    .map_err(|err| BootstrapError::DeserializeError(format!("{}", err)))?;
                msg
            }
        };
        Ok(message)
    }

    #[allow(dead_code)]
    /// Send a message to the bootstrap server
    pub fn send_timeout(
        &mut self,
        msg: &BootstrapClientMessage,
        duration: Option<Duration>,
    ) -> Result<(), BootstrapError> {
        let mut msg_bytes = Vec::new();
        let message_serializer = BootstrapClientMessageSerializer::new();
        message_serializer.serialize(msg, &mut msg_bytes)?;
        let msg_len: u32 = msg_bytes.len().try_into().map_err(|e| {
            BootstrapError::GeneralError(format!("bootstrap message too large to encode: {}", e))
        })?;

        if let Some(prev_message) = self.prev_message {
            // there was a previous message
            let prev_message = prev_message.to_bytes();

            // update current previous message to be hash(prev_msg_hash + msg)
            let mut hash_data =
                Vec::with_capacity(prev_message.len().saturating_add(msg_bytes.len()));
            hash_data.extend(prev_message);
            hash_data.extend(&msg_bytes);
            self.prev_message = Some(Hash::compute_from(&hash_data));

            // send old previous message
            self.duplex.write_all(prev_message)?;
        } else {
            // there was no previous message

            //update current previous message
            self.prev_message = Some(Hash::compute_from(&msg_bytes));
        }

        // send message length
        {
            self.duplex.set_write_timeout(duration)?;
            let msg_len_bytes = msg_len.to_be_bytes_min(self.cfg.max_bootstrap_message_size)?;
            self.duplex.write_all(&msg_len_bytes)?;
        }

        // send message
        self.duplex.write_all(&msg_bytes)?;
        Ok(())
    }
}
