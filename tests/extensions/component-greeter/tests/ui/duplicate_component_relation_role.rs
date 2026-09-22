use fabric::{component, resource};

resource! {
    Store {
        id: "fabric.test.component.relation-store";
        api { fn get(&self) -> usize; }
    }
}

component! {
    DuplicateRelationRole {
        id: "fabric.test.component.duplicate-relation-role";
        relations {
            requires {
                store: Store;
                store: Store;
            }
        }
    }
}

fn main() {}
