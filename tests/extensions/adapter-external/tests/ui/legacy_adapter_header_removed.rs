use fabric::adapter;

adapter! {
    LegacyAdapter for resource example::Store implements example::StoreRealization {
        runtime {}
    }
}

fn main() {}
