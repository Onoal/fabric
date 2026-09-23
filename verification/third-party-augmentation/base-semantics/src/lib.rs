#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading(pub u64);

fabric::resource! {
    pub ThirdPartyStore {
        id: "third.party.store";
        config { value: u64; }
        api { fn read(&self) -> crate::Reading; }
        runtime {
            fn read(&self) -> crate::Reading { crate::Reading(self.config.value) }
        }
    }
}

fabric::system! {
    pub ThirdPartyClock {
        id: "third.party.clock";
        config { value: u64; }
        api { fn now(&self) -> crate::Reading; }
        runtime {
            fn now(&self) -> crate::Reading { crate::Reading(self.config.value) }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EchoInput;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EchoOutput(pub &'static str);

fabric::component! {
    pub ThirdPartyComponent {
        id: "third.party.component";
        api {
            fn echo(&self, input: crate::EchoInput) -> crate::EchoOutput;
        }
        runtime {
            fn echo(&self, input: crate::EchoInput) -> crate::EchoOutput {
                let _ = input;
                crate::EchoOutput("base")
            }
        }
    }
}
