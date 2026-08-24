use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{AnalysisDomain, Morphology, NodeId, NodeIx};

/// Non-destructive selection shared by morphometrics, TMD, and tree rendering.
///
/// Nodes are filtered first. Edges are then retained only when both endpoints
/// are selected, so every result is an induced forest and no bridging geometry
/// is invented. An empty `kinds`, `root_ids`, or `node_ids` list means that the
/// corresponding restriction is not applied.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionQuery {
    #[serde(default)]
    pub domain: AnalysisDomain,
    #[serde(default)]
    pub kinds: Vec<i32>,
    #[serde(default)]
    pub root_ids: Vec<i64>,
    #[serde(default)]
    pub node_ids: Vec<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Selector {
    All,
    Roots,
    BranchPoints,
    Terminals,
    Soma,
    BranchOrder {
        #[serde(default)]
        exact: Option<u32>,
        #[serde(default)]
        min: Option<u32>,
        #[serde(default)]
        max: Option<u32>,
    },
    StrahlerOrder {
        #[serde(default)]
        exact: Option<u32>,
        #[serde(default)]
        min: Option<u32>,
        #[serde(default)]
        max: Option<u32>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeSelection {
    pub selector: Selector,
    pub node_ids: Vec<i64>,
    pub morphology_fingerprint: String,
    pub topology_fingerprint: String,
    pub selection_fingerprint: String,
    pub query: SelectionQuery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryError {
    UnknownRoot(i64),
    UnknownNode(i64),
    Empty,
    InvalidSelector(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRoot(id) => write!(f, "selection root node {id} does not exist"),
            Self::UnknownNode(id) => write!(f, "selected node {id} does not exist"),
            Self::Empty => f.write_str("selection contains no nodes"),
            Self::InvalidSelector(message) => write!(f, "invalid selector: {message}"),
        }
    }
}

impl std::error::Error for QueryError {}

pub(crate) struct SelectionView<'a> {
    pub(crate) morphology: &'a Morphology,
    pub(crate) query: SelectionQuery,
    included: Vec<bool>,
    fingerprint: String,
}

impl<'a> SelectionView<'a> {
    pub(crate) fn new(
        morphology: &'a Morphology,
        query: &SelectionQuery,
    ) -> Result<Self, QueryError> {
        let mut normalized = query.clone();
        normalized.kinds.sort_unstable();
        normalized.kinds.dedup();
        normalized.root_ids.sort_unstable();
        normalized.root_ids.dedup();
        normalized.node_ids.sort_unstable();
        normalized.node_ids.dedup();

        let mut below_roots = vec![normalized.root_ids.is_empty(); morphology.len()];
        for id in &normalized.root_ids {
            let root = morphology
                .index_of(NodeId(*id))
                .ok_or(QueryError::UnknownRoot(*id))?;
            let mut stack = vec![root];
            while let Some(node) = stack.pop() {
                below_roots[node.0 as usize] = true;
                for child in morphology.children(node).rev() {
                    stack.push(child);
                }
            }
        }

        let explicit: HashSet<i64> = normalized.node_ids.iter().copied().collect();
        for id in &explicit {
            if morphology.index_of(NodeId(*id)).is_none() {
                return Err(QueryError::UnknownNode(*id));
            }
        }
        let kind_filter: HashSet<i32> = normalized.kinds.iter().copied().collect();
        let included: Vec<bool> = (0..morphology.len())
            .map(|ix| {
                let domain =
                    normalized.domain == AnalysisDomain::Raw || morphology.kinds()[ix] != 1;
                let kind = kind_filter.is_empty() || kind_filter.contains(&morphology.kinds()[ix]);
                let node = explicit.is_empty() || explicit.contains(&morphology.ids()[ix]);
                domain && kind && node && below_roots[ix]
            })
            .collect();
        if !included.iter().any(|value| *value) {
            return Err(QueryError::Empty);
        }
        let fingerprint = selection_fingerprint(morphology, &normalized, &included);
        Ok(Self {
            morphology,
            query: normalized,
            included,
            fingerprint,
        })
    }

    pub(crate) fn includes(&self, node: NodeIx) -> bool {
        self.included[node.0 as usize]
    }

    pub(crate) fn nodes(&self) -> impl DoubleEndedIterator<Item = NodeIx> + '_ {
        (0..self.morphology.len() as u32)
            .map(NodeIx)
            .filter(|node| self.includes(*node))
    }

    pub(crate) fn parent(&self, node: NodeIx) -> Option<NodeIx> {
        self.morphology
            .parent(node)
            .filter(|parent| self.includes(*parent))
    }

    pub(crate) fn children(&self, node: NodeIx) -> impl DoubleEndedIterator<Item = NodeIx> + '_ {
        self.morphology
            .children(node)
            .filter(|child| self.includes(*child))
    }

    pub(crate) fn child_count(&self, node: NodeIx) -> usize {
        self.children(node).count()
    }

    pub(crate) fn roots(&self) -> Vec<NodeIx> {
        self.nodes()
            .filter(|node| self.parent(*node).is_none())
            .collect()
    }

    pub(crate) fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub(crate) fn branch_orders(&self) -> Vec<u32> {
        let mut order = vec![0_u32; self.morphology.len()];
        let roots = self.roots();
        let mut stack: Vec<NodeIx> = roots.iter().copied().rev().collect();
        for root in roots {
            order[root.0 as usize] = 1;
        }
        while let Some(parent) = stack.pop() {
            let increment = u32::from(self.child_count(parent) > 1);
            for child in self.children(parent).rev() {
                order[child.0 as usize] = order[parent.0 as usize] + increment;
                stack.push(child);
            }
        }
        order
    }

    pub(crate) fn strahler_orders(&self) -> Vec<u32> {
        let mut order = vec![0_u32; self.morphology.len()];
        let mut stack: Vec<(NodeIx, bool)> = self
            .roots()
            .into_iter()
            .rev()
            .map(|root| (root, false))
            .collect();
        while let Some((node, visited)) = stack.pop() {
            if !visited {
                stack.push((node, true));
                for child in self.children(node).rev() {
                    stack.push((child, false));
                }
                continue;
            }
            let mut maximum = 0_u32;
            let mut maximum_count = 0_u32;
            for child in self.children(node) {
                let child_order = order[child.0 as usize];
                if child_order > maximum {
                    maximum = child_order;
                    maximum_count = 1;
                } else if child_order == maximum {
                    maximum_count += 1;
                }
            }
            order[node.0 as usize] = if maximum == 0 {
                1
            } else {
                maximum + u32::from(maximum_count >= 2)
            };
        }
        order
    }

    pub(crate) fn root_path_lengths(&self) -> Vec<f64> {
        let mut paths = vec![0.0_f64; self.morphology.len()];
        let mut corrections = vec![0.0_f64; self.morphology.len()];
        let mut stack: Vec<NodeIx> = self.roots().into_iter().rev().collect();
        while let Some(parent) = stack.pop() {
            for child in self.children(parent).rev() {
                let parent_ix = parent.0 as usize;
                let child_ix = child.0 as usize;
                let edge = self
                    .morphology
                    .position(parent)
                    .distance(self.morphology.position(child));
                let next = paths[parent_ix] + edge;
                let correction = if paths[parent_ix].abs() >= edge.abs() {
                    corrections[parent_ix] + (paths[parent_ix] - next) + edge
                } else {
                    corrections[parent_ix] + (edge - next) + paths[parent_ix]
                };
                paths[child_ix] = next;
                corrections[child_ix] = correction;
                stack.push(child);
            }
        }
        for node in self.nodes() {
            paths[node.0 as usize] += corrections[node.0 as usize];
        }
        paths
    }

    pub(crate) fn radial_distances(&self) -> Vec<f64> {
        let mut distances = vec![0.0_f64; self.morphology.len()];
        let mut stack: Vec<(NodeIx, crate::Vec3)> = self
            .roots()
            .into_iter()
            .rev()
            .map(|root| (root, self.morphology.position(root)))
            .collect();
        while let Some((node, origin)) = stack.pop() {
            distances[node.0 as usize] = self.morphology.position(node).distance(origin);
            for child in self.children(node).rev() {
                stack.push((child, origin));
            }
        }
        distances
    }
}

impl Morphology {
    pub fn query_nodes(
        &self,
        query: &SelectionQuery,
        selector: Selector,
    ) -> Result<NodeSelection, QueryError> {
        let view = SelectionView::new(self, query)?;
        let orders = match &selector {
            Selector::BranchOrder { exact, min, max } => {
                validate_order_selector(*exact, *min, *max)?;
                Some(view.branch_orders())
            }
            Selector::StrahlerOrder { exact, min, max } => {
                validate_order_selector(*exact, *min, *max)?;
                Some(view.strahler_orders())
            }
            _ => None,
        };
        let node_ids = view
            .nodes()
            .filter(|node| match &selector {
                Selector::All => true,
                Selector::Roots => view.parent(*node).is_none(),
                Selector::BranchPoints => view.child_count(*node) > 1,
                Selector::Terminals => view.child_count(*node) == 0,
                Selector::Soma => self.kind(*node) == 1,
                Selector::BranchOrder { exact, min, max }
                | Selector::StrahlerOrder { exact, min, max } => matches_order(
                    orders.as_ref().expect("order selector initializes values")[node.0 as usize],
                    *exact,
                    *min,
                    *max,
                ),
            })
            .map(|node| self.id(node).0)
            .collect();
        Ok(NodeSelection {
            selector,
            node_ids,
            morphology_fingerprint: self.fingerprint().to_owned(),
            topology_fingerprint: self.topology_fingerprint().to_owned(),
            selection_fingerprint: view.fingerprint().to_owned(),
            query: view.query,
        })
    }
}

fn validate_order_selector(
    exact: Option<u32>,
    min: Option<u32>,
    max: Option<u32>,
) -> Result<(), QueryError> {
    if exact.is_none() && min.is_none() && max.is_none() {
        return Err(QueryError::InvalidSelector(
            "an order selector requires `exact`, `min`, or `max`".to_owned(),
        ));
    }
    if exact.is_some() && (min.is_some() || max.is_some()) {
        return Err(QueryError::InvalidSelector(
            "`exact` cannot be combined with `min` or `max`".to_owned(),
        ));
    }
    if exact == Some(0) || min == Some(0) || max == Some(0) {
        return Err(QueryError::InvalidSelector(
            "branch and Strahler orders are positive integers".to_owned(),
        ));
    }
    if min
        .zip(max)
        .is_some_and(|(minimum, maximum)| minimum > maximum)
    {
        return Err(QueryError::InvalidSelector(
            "`min` must not exceed `max`".to_owned(),
        ));
    }
    Ok(())
}

fn matches_order(value: u32, exact: Option<u32>, min: Option<u32>, max: Option<u32>) -> bool {
    exact.map_or_else(
        || min.is_none_or(|minimum| value >= minimum) && max.is_none_or(|maximum| value <= maximum),
        |expected| value == expected,
    )
}

fn selection_fingerprint(
    morphology: &Morphology,
    query: &SelectionQuery,
    included: &[bool],
) -> String {
    let mut hash = Fnv1a64::new();
    hash.update(b"axodendron-selection-v1\0");
    hash.update(morphology.topology_fingerprint().as_bytes());
    hash.update(&[match query.domain {
        AnalysisDomain::Raw => 0,
        AnalysisDomain::Neurites => 1,
    }]);
    for kind in &query.kinds {
        hash.update(&kind.to_le_bytes());
    }
    hash.update(&[0xff]);
    for id in &query.root_ids {
        hash.update(&id.to_le_bytes());
    }
    hash.update(&[0xfe]);
    for id in &query.node_ids {
        hash.update(&id.to_le_bytes());
    }
    hash.update(&[0xfd]);
    for (ix, selected) in included.iter().copied().enumerate() {
        if selected {
            hash.update(&morphology.ids()[ix].to_le_bytes());
        }
    }
    hash.finish()
}

struct Fnv1a64(u64);

impl Fnv1a64 {
    const fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn finish(self) -> String {
        format!("fnv1a64:{:016x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ValidationProfile, parse_swc};

    use super::*;

    fn morphology() -> Morphology {
        parse_swc(
            "1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 1 0 1 2\n4 3 2 -1 0 1 2\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap()
    }

    #[test]
    fn query_is_an_induced_forest_and_selectors_are_stable() {
        let morphology = morphology();
        let query = SelectionQuery {
            domain: AnalysisDomain::Neurites,
            ..Default::default()
        };
        assert_eq!(
            morphology
                .query_nodes(&query, Selector::Roots)
                .unwrap()
                .node_ids,
            vec![2]
        );
        assert_eq!(
            morphology
                .query_nodes(&query, Selector::BranchPoints)
                .unwrap()
                .node_ids,
            vec![2]
        );
        assert_eq!(
            morphology
                .query_nodes(&query, Selector::Terminals)
                .unwrap()
                .node_ids,
            vec![3, 4]
        );
    }

    #[test]
    fn query_fingerprint_is_canonical() {
        let morphology = morphology();
        let a = morphology
            .query_nodes(
                &SelectionQuery {
                    kinds: vec![3, 3],
                    ..Default::default()
                },
                Selector::All,
            )
            .unwrap();
        let b = morphology
            .query_nodes(
                &SelectionQuery {
                    kinds: vec![3],
                    ..Default::default()
                },
                Selector::All,
            )
            .unwrap();
        assert_eq!(a.selection_fingerprint, b.selection_fingerprint);
    }

    #[test]
    fn order_selectors_are_exact_ranged_and_validated() {
        let morphology = morphology();
        let query = SelectionQuery {
            domain: AnalysisDomain::Neurites,
            ..Default::default()
        };
        let branch = morphology
            .query_nodes(
                &query,
                Selector::BranchOrder {
                    exact: Some(2),
                    min: None,
                    max: None,
                },
            )
            .unwrap();
        assert_eq!(branch.node_ids, vec![3, 4]);

        let strahler = morphology
            .query_nodes(
                &query,
                Selector::StrahlerOrder {
                    exact: None,
                    min: Some(2),
                    max: Some(2),
                },
            )
            .unwrap();
        assert_eq!(strahler.node_ids, vec![2]);

        assert!(matches!(
            morphology.query_nodes(
                &query,
                Selector::BranchOrder {
                    exact: None,
                    min: None,
                    max: None,
                },
            ),
            Err(QueryError::InvalidSelector(_))
        ));
        assert!(matches!(
            morphology.query_nodes(
                &query,
                Selector::StrahlerOrder {
                    exact: Some(1),
                    min: Some(1),
                    max: None,
                },
            ),
            Err(QueryError::InvalidSelector(_))
        ));
    }
}
