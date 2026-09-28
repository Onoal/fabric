use std::rc::Rc;
use std::sync::Arc;

use fabric::authoring::{CompositionExt, FabricBuilder, Requires};
use fabric::core::{CompositionExport, ContractId};
use fabric::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProbeError(&'static str);

fabric::resource! {
    LocalAsyncProbe {
        id: "fabric.test.resource-async.local";
        api {
            async fn read(&self) -> usize;
            async fn fail(&self) -> Result<(), ProbeError>;
        }
        runtime {
            async fn read(&self) -> usize { 7 }
            async fn fail(&self) -> Result<(), ProbeError> { Err(ProbeError("semantic")) }
        }
    }
}

fabric::resource! {
    EventLoopProbe {
        id: "fabric.test.resource-async.event-loop";
        api { async fn read(&self) -> usize; }
    }
}

fabric::adapter! {
    NonSendEventLoopProbe for EventLoopProbe {
        id: "test.resource-async.non-send-event-loop";
        runtime {
            async fn read(&self) -> usize {
                let marker = Rc::new(41usize);
                std::future::ready(()).await;
                *marker + 1
            }
        }
    }
}

fabric::component! {
    EventLoopProbeConsumer {
        id: "fabric.test.resource-async.event-loop-consumer";
        relations { requires { probe: EventLoopProbe; } }
        api { fn run(&self) -> usize; }
        runtime {
            fn run(&self) -> usize {
                crate::support::expect_resource_ready(self.relations().probe.read())
            }
        }
    }
}

struct ManualProbeService;

impl local_async_probe::raw::ApiService for ManualProbeService {
    fn read<'a>(&'a self) -> fabric::resource::ResourceFuture<'a, usize> {
        Box::pin(async move { 11 })
    }

    fn fail<'a>(&'a self) -> fabric::resource::ResourceFuture<'a, Result<(), ProbeError>> {
        Box::pin(async move { Err(ProbeError("manual")) })
    }
}

#[test]
fn local_resource_operations_are_awaitable_and_preserve_errors() {
    let export = CompositionExport::new(
        ContractId::new("fabric.test.resource-async.local.export").expect("export id"),
        Requires::<LocalAsyncProbe>::provisional()
            .as_contract_requirement()
            .clone(),
    );
    let composition = FabricBuilder::new("fabric.test.resource-async.local")
        .expect("builder")
        .block("runtime", |block| {
            block.module(LocalAsyncProbe::select("primary").expect("selection"))
        })
        .expect("block")
        .export(export.clone())
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_core_on(
            "fabric.test.resource-async.local.instance",
            &HostDescriptor::native(),
        )
        .expect("materialize");
    instance.start().expect("start");
    let service = instance.export(&export).expect("resource export");

    assert_eq!(futures::executor::block_on(service.read()), 7);
    assert_eq!(
        futures::executor::block_on(service.fail()),
        Err(ProbeError("semantic"))
    );

    instance.stop().expect("stop");
}

#[test]
fn non_send_resource_operation_future_is_representable() {
    let selection = EventLoopProbe::select("primary")
        .expect("selection")
        .using(NonSendEventLoopProbe::new())
        .expect("adapter");
    let composition = Fabric::new("fabric.test.resource-async.event-loop")
        .expect("fabric")
        .resource(selection)
        .component(EventLoopProbeConsumer::define())
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_on(
            "fabric.test.resource-async.event-loop.instance",
            &HostDescriptor::native(),
        )
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<EventLoopProbeConsumer>()
        .expect("materialize component");
    let app = instance
        .component::<EventLoopProbeConsumer>()
        .expect("component handle");

    assert_eq!(futures::executor::block_on(app.run()).expect("run"), 42);

    components
        .dematerialize::<EventLoopProbeConsumer>()
        .expect("dematerialize");

    instance.stop().expect("stop");
}

#[test]
fn generated_resource_service_is_object_safe() {
    let service: Arc<dyn local_async_probe::raw::ApiService> = Arc::new(ManualProbeService);
    let contract = local_async_probe::raw::ApiContract::new(service);

    assert_eq!(futures::executor::block_on(contract.read()), 11);
    assert_eq!(
        futures::executor::block_on(contract.fail()),
        Err(ProbeError("manual"))
    );
}
