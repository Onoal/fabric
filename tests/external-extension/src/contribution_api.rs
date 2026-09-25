use fabric::*;

fabric::resource! {
    pub ExternalContributionStore {
        id: "fabric.test.external.contribution-store";

        api {
            fn count(&self) -> usize;
        }
    }
}

fabric::adapter! {
    pub ExternalContributionStoreAdapter for ExternalContributionStore {
        id: "test.external-contribution-store-adapter";

        runtime {
            fn count(&self) -> usize { 13 }
        }
    }
}

pub fn external_storage(name: &'static str) -> impl IntoFabricContribution {
    let store = ExternalContributionStore::select(name)
        .expect("external contribution store")
        .using(ExternalContributionStoreAdapter::new())
        .expect("external contribution adapter");

    FabricContribution::new().resource(store)
}
