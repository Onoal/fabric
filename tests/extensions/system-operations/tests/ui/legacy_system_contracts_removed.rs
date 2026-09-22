use fabric::system;

system! {
    Legacy {
        id: "fabric.test.legacy.system-contracts";
        contracts { primary Api { id: "fabric.test.legacy.system-contracts.api"; } }
    }
}

fn main() {}
