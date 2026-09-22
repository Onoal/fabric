use fabric::resource;

resource! {
    Legacy {
        id: "fabric.test.legacy.realization";
        api { fn value(&self) -> u64; }
        adapter Adapter { id: "fabric.test.legacy.realization.adapter"; }
    }
}

fn main() {}
