use crate::model::context::ResourceContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceContextBindingProvenance {
    consumer_id: String,
    identity_material: Vec<u8>,
}

pub fn resource_context_binding_provenance(
    context: &ResourceContext,
) -> ResourceContextBindingProvenance {
    let identity_material = encode_context_identity_material(context);
    ResourceContextBindingProvenance {
        consumer_id: format!(
            "fabric-resource-context-v1:{}",
            encode_hex(&identity_material)
        ),
        identity_material,
    }
}

impl ResourceContextBindingProvenance {
    pub fn consumer_id(&self) -> &str {
        &self.consumer_id
    }

    pub fn identity_material(&self) -> &[u8] {
        &self.identity_material
    }
}

fn encode_context_identity_material(context: &ResourceContext) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(context.boundary().as_str().as_bytes());
    encoded.push(0);
    for scope in context.path() {
        encoded.extend_from_slice(scope.as_str().as_bytes());
        encoded.push(0);
    }
    encoded
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
