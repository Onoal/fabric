fabric_sdk::adapter! {
    pub GenericRuntimeAdapter for system fabric_test_system_operations::AdaptedOperations implements fabric_test_system_operations::AdaptedOperationsRealization {
        schema: "^2";
        realization: "1.0.0";

        config {
            value: u64;
        }

        runtime {
            fn current_marker<T>(&self) -> fabric_test_system_operations::OperationMarker {
                let _ = ::std::marker::PhantomData::<T>;
                fabric_test_system_operations::OperationMarker::new(self.config.value)
            }
        }
    }
}

fn main() {}
