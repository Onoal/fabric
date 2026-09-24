use fabric::{adapter, resource};

resource! {
    Store {
        id: "fabric.test.ui.adapter-id.store";
        api { fn get(&self) -> usize; }
    }
}

adapter! {
    DuplicateId for Store {
        id: "test.duplicate-one";
        id: "test.duplicate-two";
        runtime { fn get(&self) -> usize { 1 } }
    }
}

fn main() {}
