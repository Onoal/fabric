use fabric::*;

resource! {
    BadResource {
        id: "fabric.test.ui.resource-adapter-runtime-mismatch";
        api { async fn read(&self, value: usize) -> usize; }
    }
}

adapter! {
    BadAdapter for BadResource {
        id: "test.resource-adapter-runtime-mismatch";
        runtime { async fn read(&self, value: String) -> usize { value.len() } }
    }
}

fn main() {}
