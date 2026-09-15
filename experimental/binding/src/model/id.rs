use sha2::{Digest, Sha256};

use crate::{BindingConsumer, BindingError, BindingName, BindingTargetId, BindingTargetKind};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingId(String);

impl BindingId {
    pub fn parse(value: impl Into<String>) -> Result<Self, BindingError> {
        let value = value.into();
        if !value.starts_with("fabric-binding-")
            || value.len() != "fabric-binding-".len() + 64
            || !value["fabric-binding-".len()..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(BindingError::invalid_input("invalid binding id"));
        }
        Ok(Self(value))
    }

    pub fn canonical(
        consumer: &BindingConsumer,
        name: &BindingName,
        target_kind: &BindingTargetKind,
        target_id: &BindingTargetId,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"fabric.binding.identity.v1");
        hasher.update([0]);
        hasher.update(consumer.kind().as_str().as_bytes());
        hasher.update([0]);
        hasher.update(consumer.id().as_str().as_bytes());
        hasher.update([0]);
        hasher.update(name.as_str().as_bytes());
        hasher.update([0]);
        hasher.update(target_kind.as_str().as_bytes());
        hasher.update([0]);
        hasher.update(target_id.as_str().as_bytes());
        encode_binding_id(hasher.finalize())
    }

    pub fn resource_instance_import(
        resource: &str,
        resource_instance_id: &str,
        context_identity: &[u8],
        name: &BindingName,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"fabric.resource.binding.identity.v1");
        hasher.update([0]);
        hasher.update(resource.as_bytes());
        hasher.update([0]);
        hasher.update(resource_instance_id.as_bytes());
        hasher.update([0]);
        hasher.update(context_identity);
        hasher.update([0]);
        hasher.update(name.as_str().as_bytes());
        encode_binding_id(hasher.finalize())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn encode_binding_id(digest: impl AsRef<[u8]>) -> BindingId {
    BindingId::parse(format!("fabric-binding-{}", encode_hex(digest.as_ref())))
        .expect("derived binding id")
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(hex(byte >> 4));
        encoded.push(hex(byte & 0x0f));
    }
    encoded
}

fn hex(nibble: u8) -> char {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    TABLE[nibble as usize] as char
}
