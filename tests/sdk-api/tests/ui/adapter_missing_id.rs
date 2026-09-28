use fabric::{adapter, resource};

resource! {
    Store {
        id: "fabric.test.ui.adapter-id.store";
        api { async fn get(&self) -> usize; }
    }
}

adapter! {
    MissingId for Store {
        runtime { async fn get(&self) -> usize { 1 } }
    }
}

fn main() {}
