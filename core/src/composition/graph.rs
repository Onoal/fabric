use std::collections::{BTreeMap, VecDeque};

use crate::ModuleDeclaration;
use crate::composition::DeclaredBindings;
use crate::error::CompositionError;

pub(crate) fn dependency_order(
    modules: &[ModuleDeclaration],
    bindings: &DeclaredBindings,
) -> Result<Vec<usize>, CompositionError> {
    let index_by_id: BTreeMap<_, _> = modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_id().clone(), index))
        .collect();

    let mut indegree = vec![0usize; modules.len()];
    let mut adjacency = vec![Vec::<usize>::new(); modules.len()];

    for (consumer_index, module) in modules.iter().enumerate() {
        for contract_id in module
            .required_contracts()
            .iter()
            .chain(module.optional_contracts())
        {
            let Some(provider_id) = bindings
                .get(module.module_id())
                .and_then(|module_bindings| module_bindings.get(contract_id.id()))
                .map(|bound| bound.provider.clone())
            else {
                continue;
            };
            let provider_index = *index_by_id
                .get(&provider_id)
                .expect("provider module id must exist");
            if provider_index == consumer_index {
                continue;
            }
            adjacency[provider_index].push(consumer_index);
            indegree[consumer_index] += 1;
        }
    }

    let mut queue = indegree
        .iter()
        .enumerate()
        .filter(|(_, count)| **count == 0)
        .map(|(index, _)| index)
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(modules.len());

    while let Some(index) = queue.pop_front() {
        order.push(index);
        for consumer in &adjacency[index] {
            indegree[*consumer] -= 1;
            if indegree[*consumer] == 0 {
                queue.push_back(*consumer);
            }
        }
    }

    if order.len() == modules.len() {
        Ok(order)
    } else {
        let cycle = indegree
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .map(|(index, _)| modules[index].module_id().clone())
            .collect();
        Err(CompositionError::DependencyCycle { module_ids: cycle })
    }
}
