use fabric::{adapter, resource};

resource! {
    Store {
        id: "fabric.test.ui.adapter-id.store";
        api { async fn get(&self) -> usize; }
    }
}

adapter! {
    DuplicateId for Store {
        id: "test.duplicate-one";
        id: "test.duplicate-two";
        runtime { async fn get(&self) -> usize { 1 } }
    }
}

fn main() {}
