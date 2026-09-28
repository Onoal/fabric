use fabric::*;

resource! {
    BadResource {
        id: "fabric.test.ui.resource-sync-api";
        api { fn read(&self) -> usize; }
    }
}

fn main() {}
