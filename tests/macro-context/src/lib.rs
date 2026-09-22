//! Compile fixtures for location-independent adaptable Resource and System
//! macro expansion. The declarations intentionally exercise caller-local and
//! imported Rust types at several module depths.

use std::sync::atomic::{AtomicBool, Ordering};

use fabric::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedValue(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootKey(pub u64);

pub static ROOT_ADAPTER_STOPPED: AtomicBool = AtomicBool::new(false);

pub fn reset_root_adapter_stop() {
    ROOT_ADAPTER_STOPPED.store(false, Ordering::SeqCst);
}

pub fn root_adapter_stopped() -> bool {
    ROOT_ADAPTER_STOPPED.load(Ordering::SeqCst)
}

#[derive(Default)]
struct RootStoreState;

resource! {
    pub RootVersionedStore {
        id: "fabric.test.macro-context.root-store";
        version: "0.1.0";

        contracts {
            primary Api {
                id: "fabric.test.macro-context.root-store.api";
                fn get(&self, key: RootKey) -> ImportedValue;
            }
        }

        adapter Adapter {
            id: "fabric.test.macro-context.root-store.adapter";
            compatibility: "^1";
            fn get(&self, key: RootKey) -> ImportedValue;
        }

        runtime {
            fn get(&self, key: RootKey) -> ImportedValue {
                self.adapter.get(key)
            }
        }
    }
}

adapter! {
    pub RootStatefulStore
        for resource RootVersionedStore
        implements RootVersionedStoreRealization
    {
        version: "1.0.0";

        state {
            RootStoreState = RootStoreState;
        }

        runtime {
            fn get(&self, key: RootKey) -> ImportedValue {
                ImportedValue(key.0)
            }
        }

        lifecycle {
            stop {
                let _ = self.state.get();
                ROOT_ADAPTER_STOPPED.store(true, Ordering::SeqCst);
                Ok(())
            }
        }
    }
}

system! {
    pub RootVersionedSystem {
        id: "fabric.test.macro-context.root-system";
        version: "0.1.0";

        contracts {
            primary Api {
                id: "fabric.test.macro-context.root-system.api";
                fn marker(&self) -> ImportedValue;
            }
        }

        adapter Adapter {
            id: "fabric.test.macro-context.root-system.adapter";
            compatibility: "^1";
            fn marker(&self) -> ImportedValue;
        }

        runtime {
            fn marker(&self) -> ImportedValue {
                self.adapter.marker()
            }
        }
    }
}

pub mod capability {
    use fabric::*;

    use crate::ImportedValue;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ModuleKey(pub u64);

    resource! {
        pub ModuleStore {
            id: "fabric.test.macro-context.module-store";

            contracts {
                primary Api {
                    id: "fabric.test.macro-context.module-store.api";
                    fn get(&self, key: ModuleKey) -> ImportedValue;
                }
            }

            adapter Adapter {
                id: "fabric.test.macro-context.module-store.adapter";
                compatibility: provisional;
                fn get(&self, key: ModuleKey) -> ImportedValue;
            }

            runtime {
                fn get(&self, key: ModuleKey) -> ImportedValue {
                    self.adapter.get(key)
                }
            }
        }
    }

    system! {
        pub ModuleSystem {
            id: "fabric.test.macro-context.module-system";

            contracts {
                primary Api {
                    id: "fabric.test.macro-context.module-system.api";
                    fn marker(&self) -> ImportedValue;
                }
            }

            adapter Adapter {
                id: "fabric.test.macro-context.module-system.adapter";
                compatibility: provisional;
                fn marker(&self) -> ImportedValue;
            }

            runtime {
                fn marker(&self) -> ImportedValue {
                    self.adapter.marker()
                }
            }
        }
    }
}

pub mod outer {
    pub mod middle {
        pub mod inner {
            use fabric::*;

            use crate::ImportedValue;

            #[derive(Clone, Debug, PartialEq, Eq)]
            pub struct DeepKey(pub u64);

            resource! {
                pub DeepStore {
                    id: "fabric.test.macro-context.deep-store";

                    contracts {
                        primary Api {
                            id: "fabric.test.macro-context.deep-store.api";
                            fn get(&self, key: DeepKey) -> ImportedValue;
                        }
                    }

                    adapter Adapter {
                        id: "fabric.test.macro-context.deep-store.adapter";
                        compatibility: provisional;
                        fn get(&self, key: DeepKey) -> ImportedValue;
                    }

                    runtime {
                        fn get(&self, key: DeepKey) -> ImportedValue {
                            self.adapter.get(key)
                        }
                    }
                }
            }

            system! {
                pub DeepSystem {
                    id: "fabric.test.macro-context.deep-system";

                    contracts {
                        primary Api {
                            id: "fabric.test.macro-context.deep-system.api";
                            fn marker(&self) -> ImportedValue;
                        }
                    }

                    adapter Adapter {
                        id: "fabric.test.macro-context.deep-system.adapter";
                        compatibility: provisional;
                        fn marker(&self) -> ImportedValue;
                    }

                    runtime {
                        fn marker(&self) -> ImportedValue {
                            self.adapter.marker()
                        }
                    }
                }
            }
        }
    }
}
