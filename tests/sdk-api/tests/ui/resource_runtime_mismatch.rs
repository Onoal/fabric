use fabric::*;

resource! {
    BadResource {
        id: "fabric.test.ui.resource-runtime-mismatch";
        api { async fn read(&self, value: usize) -> usize; }
        runtime { async fn read(&self, value: String) -> usize { value.len() } }
    }
}

fn main() {}
