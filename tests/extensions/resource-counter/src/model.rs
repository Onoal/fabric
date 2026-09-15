use fabric_resource::ResourceId;

const COUNTER_RESOURCE_ID: &str = "fabric.test.counter";

pub fn counter_resource_id() -> ResourceId {
    ResourceId::new(COUNTER_RESOURCE_ID).expect("static counter resource id")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterValue(u64);

impl CounterValue {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}
