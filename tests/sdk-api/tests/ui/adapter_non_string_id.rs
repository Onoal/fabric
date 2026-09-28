use fabric::{adapter, resource};

resource! {
    Store {
        id: "fabric.test.ui.adapter-id.store";
        api { async fn get(&self) -> usize; }
    }
}

adapter! {
    NonStringId for Store {
        id: 42;
        runtime { async fn get(&self) -> usize { 1 } }
    }
}

fn main() {}
