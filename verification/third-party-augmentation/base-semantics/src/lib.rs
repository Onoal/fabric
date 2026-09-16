#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading(pub u64);

fabric::resource! {
    pub ThirdPartyStore {
        id: "third.party.store";
        schema: provisional;
        config { value: u64; }
        contracts {
            primary Api {
                id: "third.party.store.api";
                version: provisional;
                fn read(&self) -> crate::Reading;
            }
        }
        runtime {
            fn read(&self) -> crate::Reading { crate::Reading(self.config.value) }
        }
    }
}

fabric::system! {
    pub ThirdPartyClock {
        id: "third.party.clock";
        schema: provisional;
        config { value: u64; }
        contracts {
            primary Api {
                id: "third.party.clock.api";
                version: provisional;
                fn now(&self) -> crate::Reading;
            }
        }
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
        config {}
        operations {
            echo {
                id: "third.party.component.echo";
                input: crate::EchoInput = "third.party.component.echo.input";
                output: crate::EchoOutput = "third.party.component.echo.output";
                handler |_input: crate::EchoInput| async move { Ok(crate::EchoOutput("base")) };
            }
        }
    }
}
