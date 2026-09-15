use fabric_resource::ResourceId;

const CLOCK_RESOURCE_ID: &str = "fabric.test.clock";

pub fn clock_resource_id() -> ResourceId {
    ResourceId::new(CLOCK_RESOURCE_ID).expect("static clock resource id")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClockTick(u64);

impl ClockTick {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}
