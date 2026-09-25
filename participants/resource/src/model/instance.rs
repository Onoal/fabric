use sha2::{Digest, Sha256};

use crate::model::context::{ResourceContext, ResourceName};
use crate::{ResourceError, ResourceId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceInstanceId(String);

impl ResourceInstanceId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ResourceError> {
        let value = value.into();
        validate_resource_instance_id(&value)?;
        Ok(Self(value))
    }

    pub fn canonical(
        resource: &ResourceId,
        context: &ResourceContext,
        name: &ResourceName,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"fabric.resource.identity.v1");
        hasher.update([0]);
        hasher.update(resource.as_str().as_bytes());
        hasher.update([0]);
        write_context(&mut hasher, context);
        hasher.update([0]);
        hasher.update(name.as_str().as_bytes());
        encode_resource_instance_id(resource, hasher.finalize())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn resource(&self) -> ResourceId {
        resource_id_from_instance_id(&self.0)
            .expect("validated resource instance id has a resource")
    }
}

fn write_context(hasher: &mut Sha256, context: &ResourceContext) {
    hasher.update(context.boundary().as_str().as_bytes());
    hasher.update([0]);
    for scope in context.path() {
        hasher.update(scope.as_str().as_bytes());
        hasher.update([0]);
    }
}

fn encode_resource_instance_id(
    resource: &ResourceId,
    digest: impl AsRef<[u8]>,
) -> ResourceInstanceId {
    ResourceInstanceId::parse(format!(
        "fabric-resource-{}-{}",
        resource.as_str(),
        encode_hex(digest.as_ref())
    ))
    .expect("derived resource instance id")
}

fn validate_resource_instance_id(value: &str) -> Result<(), ResourceError> {
    let Some(suffix) = value.strip_prefix("fabric-resource-") else {
        return Err(ResourceError::InvalidInput {
            message: "invalid resource instance id".to_owned(),
        });
    };

    let Some((resource, digest)) = suffix.rsplit_once('-') else {
        return Err(ResourceError::InvalidInput {
            message: "invalid resource instance id".to_owned(),
        });
    };

    if ResourceId::new(resource).is_err()
        || digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(ResourceError::InvalidInput {
            message: "invalid resource instance id".to_owned(),
        });
    }

    Ok(())
}

fn resource_id_from_instance_id(value: &str) -> Option<ResourceId> {
    let suffix = value.strip_prefix("fabric-resource-")?;
    let (resource, _digest) = suffix.rsplit_once('-')?;
    ResourceId::new(resource).ok()
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
