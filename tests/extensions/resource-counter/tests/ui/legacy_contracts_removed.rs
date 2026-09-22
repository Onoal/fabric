use fabric::resource;

resource! {
    Legacy {
        id: "fabric.test.legacy.contracts";
        contracts { primary Api { id: "fabric.test.legacy.contracts.api"; } }
    }
}

fn main() {}
