use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use super::error::CyclicDependencyError;
use super::types::Var;
use std::fmt::Debug;

use syn::Result;

#[derive(Debug, PartialEq)]
enum SearchState {
    Unsearched,
    Searching,
    Searched,
}

pub fn tsort<'a>(
    deps: &'a HashMap<Var<'a>, HashSet<Var<'a>>>,
) -> Result<impl Iterator<Item = Var<'a>>>
where
{
    let mut dependencies: HashMap<Var, HashSet<Var>> = HashMap::new();
    let mut search_states: HashMap<Var, SearchState> = HashMap::new();
    let mut all_nodes = HashSet::new();

    deps.into_iter().for_each(|(value, nodes)| {
        all_nodes.insert(*value);
        search_states.insert(*value, SearchState::Unsearched);

        for node in nodes.into_iter() {
            all_nodes.insert(node);
            search_states.insert(node, SearchState::Unsearched);

            match dependencies.entry(node) {
                Entry::Vacant(e) => {
                    let mut s = HashSet::new();
                    s.insert(*value);
                    e.insert(s);
                }
                Entry::Occupied(mut e) => {
                    let v = e.get_mut();
                    v.insert(*value);
                }
            }
        }
    });

    let mut all_nodes: Vec<_> = all_nodes.into_iter().collect();
    all_nodes.sort();

    let result = all_nodes
        .into_iter()
        .try_fold::<_, _, Result<_>>(vec![], |mut acc, node| {
            let v = dfs(&node, &dependencies, &mut search_states)?;
            acc.extend(v);
            Ok(acc)
        })?;

    Ok(result.into_iter().rev())
}

fn dfs<'a>(
    node: &Var<'a>,
    dependencies: &HashMap<Var<'a>, HashSet<Var<'a>>>,
    search_states: &mut HashMap<Var<'a>, SearchState>,
) -> Result<Vec<Var<'a>>> {
    let mut res = vec![];

    // unsearched -> searching
    // searching  -> cyclic dependency
    // searched   -> return self
    match search_states.get_mut(node) {
        None => return Ok(res),
        Some(s) => match s {
            SearchState::Unsearched => *s = SearchState::Searching,
            SearchState::Searching => {
                return Err(CyclicDependencyError(*node).into());
            }
            SearchState::Searched => return Ok(res),
        },
    }

    for next in dependencies.get(node).into_iter().flatten() {
        let v = dfs(next, dependencies, search_states)?;
        res.extend(v);
    }

    // unsearched -> ?
    // searching  -> searched
    // searched   -> ?
    match search_states.get_mut(node) {
        None => return Ok(res),
        Some(s) => match s {
            SearchState::Unsearched => unreachable!(),
            SearchState::Searching => *s = SearchState::Searched,
            SearchState::Searched => unreachable!(),
        },
    }

    res.push(*node);
    Ok(res)
}
