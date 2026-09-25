mod definition;
mod package_only;

#[cfg(test)]
mod tests;

pub use definition::{
    Greeter, GreeterConfig, GreeterInput, GreeterInstanceApi, GreeterOutput, greeter,
};
pub use package_only::{
    EmptyComponent, EmptyComponentConfig, PackageComponent, PackageComponentConfig,
};
