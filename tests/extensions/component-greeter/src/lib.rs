mod definition;
mod package_only;

#[cfg(test)]
mod tests;

pub use definition::{Greeter, GreeterConfig, GreeterInput, GreeterOutput, greeter};
pub use package_only::{
    EmptyComponent, EmptyComponentConfig, PackageComponent, PackageComponentConfig,
};
