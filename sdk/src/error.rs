use std::error::Error;
use std::fmt;

use fabric_core::CompositionError;

#[derive(Debug)]
pub enum SdkAuthoringError {
    Composition(CompositionError),
    InvalidContractVersion {
        value: String,
        source: Box<dyn Error + Send + Sync>,
    },
    InvalidContractVersionRequirement {
        value: String,
        source: Box<dyn Error + Send + Sync>,
    },
}

impl SdkAuthoringError {
    pub(crate) fn invalid_contract_version(
        value: String,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::InvalidContractVersion {
            value,
            source: Box::new(source),
        }
    }

    pub(crate) fn invalid_contract_version_requirement(
        value: String,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::InvalidContractVersionRequirement {
            value,
            source: Box::new(source),
        }
    }
}

impl fmt::Display for SdkAuthoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Composition(error) => error.fmt(f),
            Self::InvalidContractVersion { value, source } => {
                write!(f, "invalid contract version `{value}`: {source}")
            }
            Self::InvalidContractVersionRequirement { value, source } => {
                write!(
                    f,
                    "invalid contract version requirement `{value}`: {source}"
                )
            }
        }
    }
}

impl Error for SdkAuthoringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Composition(error) => Some(error),
            Self::InvalidContractVersion { source, .. } => Some(source.as_ref()),
            Self::InvalidContractVersionRequirement { source, .. } => Some(source.as_ref()),
        }
    }
}

impl From<CompositionError> for SdkAuthoringError {
    fn from(value: CompositionError) -> Self {
        Self::Composition(value)
    }
}
