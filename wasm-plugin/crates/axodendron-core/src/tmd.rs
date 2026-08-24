use serde::{Deserialize, Serialize};

use crate::model::{Morphology, NodeIx, SomaClass};
use crate::query::{QueryError, SelectionQuery, SelectionView};

pub const TMD_DEFINITION_VERSION: u16 = 2;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TmdFiltration {
    #[default]
    RadialDistance,
    RootPathLength,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TmdCenter {
    #[serde(rename = "soma")]
    Soma,
    #[serde(rename = "root", alias = "arbor-root")]
    Root,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TmdOptions {
    #[serde(default)]
    pub selection: SelectionQuery,
    #[serde(default)]
    pub filtration: TmdFiltration,
    #[serde(default)]
    pub center: Option<TmdCenter>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistencePair {
    /// Filtration value where the leaf-associated component appears.
    pub birth: f64,
    /// Filtration value where the component merges into an older component.
    pub death: f64,
    pub persistence: f64,
    /// Sorted interval endpoints, convenient for barcode rendering even for a
    /// non-monotone radial reconstruction.
    pub start: f64,
    pub end: f64,
    pub terminal_node: i64,
    pub merge_node: i64,
    pub arbor_root: i64,
    pub essential: bool,
    pub non_monotone: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TmdProvenance {
    pub definition_version: u16,
    pub algorithm: String,
    pub elder_rule: String,
    pub tie_rule: String,
    pub root_connector_rule: String,
    pub pair_order: String,
    pub reference: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TmdResult {
    pub schema_version: u16,
    pub morphology_fingerprint: String,
    pub topology_fingerprint: String,
    pub selection_fingerprint: String,
    pub selection: SelectionQuery,
    pub filtration: TmdFiltration,
    pub center: Option<TmdCenter>,
    pub units: String,
    pub pairs: Vec<PersistencePair>,
    pub node_ids: Vec<i64>,
    pub filtration_values: Vec<f64>,
    pub non_monotone_pair_count: u32,
    pub provenance: TmdProvenance,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TmdError {
    Query(QueryError),
    NoSoma,
    AmbiguousSoma,
    CenterNotApplicable,
}

impl std::fmt::Display for TmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query(error) => error.fmt(f),
            Self::NoSoma => f.write_str("soma-centered TMD requires soma geometry"),
            Self::AmbiguousSoma => f.write_str("soma-centered TMD requires an unambiguous soma"),
            Self::CenterNotApplicable => {
                f.write_str("root-path-length TMD is rooted at each selected arbor root and does not accept `center`")
            }
        }
    }
}

impl std::error::Error for TmdError {}

impl From<QueryError> for TmdError {
    fn from(value: QueryError) -> Self {
        Self::Query(value)
    }
}

#[derive(Clone, Copy)]
struct ActiveComponent {
    terminal: NodeIx,
    birth: f64,
}

impl Morphology {
    /// Compute the Topological Morphology Descriptor with a deterministic elder
    /// rule. The longest component in every selected arbor is retained as an
    /// essential root-to-leaf bar.
    pub fn tmd(&self, options: &TmdOptions) -> Result<TmdResult, TmdError> {
        let view = SelectionView::new(self, &options.selection)?;
        let center = match options.filtration {
            TmdFiltration::RadialDistance => Some(options.center.unwrap_or(TmdCenter::Soma)),
            TmdFiltration::RootPathLength => {
                if options.center.is_some() {
                    return Err(TmdError::CenterNotApplicable);
                }
                None
            }
        };
        let soma_center = if center == Some(TmdCenter::Soma) {
            match self.soma_class() {
                SomaClass::Absent => return Err(TmdError::NoSoma),
                SomaClass::Disconnected | SomaClass::Ambiguous => {
                    return Err(TmdError::AmbiguousSoma);
                }
                _ => Some(self.soma_center()),
            }
        } else {
            None
        };
        let mut filtration = vec![0.0; self.len()];
        let roots = view.roots();
        for root in &roots {
            let radial_origin = soma_center.unwrap_or_else(|| self.position(*root));
            let mut stack = vec![(*root, 0.0)];
            while let Some((node, path)) = stack.pop() {
                filtration[node.0 as usize] = match options.filtration {
                    TmdFiltration::RadialDistance => self.position(node).distance(radial_origin),
                    TmdFiltration::RootPathLength => path,
                };
                for child in view.children(node).rev() {
                    let next = path + self.position(node).distance(self.position(child));
                    stack.push((child, next));
                }
            }
        }

        let mut pairs = Vec::new();
        let mut active = vec![None::<ActiveComponent>; self.len()];
        for root in &roots {
            let mut stack = vec![(*root, false)];
            while let Some((node, visited)) = stack.pop() {
                if !visited {
                    stack.push((node, true));
                    for child in view.children(node).rev() {
                        stack.push((child, false));
                    }
                    continue;
                }
                let children: Vec<NodeIx> = view.children(node).collect();
                if children.is_empty() {
                    active[node.0 as usize] = Some(ActiveComponent {
                        terminal: node,
                        birth: filtration[node.0 as usize],
                    });
                    continue;
                }
                let mut components: Vec<ActiveComponent> = children
                    .iter()
                    .filter_map(|child| active[child.0 as usize])
                    .collect();
                components.sort_by(|a, b| {
                    b.birth
                        .total_cmp(&a.birth)
                        .then_with(|| self.id(a.terminal).0.cmp(&self.id(b.terminal).0))
                });
                let winner = components[0];
                if components.len() > 1 {
                    for killed in &components[1..] {
                        pairs.push(persistence_pair(
                            self,
                            *killed,
                            node,
                            *root,
                            filtration[node.0 as usize],
                            false,
                        ));
                    }
                }
                active[node.0 as usize] = Some(winner);
            }
            let survivor =
                active[root.0 as usize].expect("selected root has a terminal descendant");
            pairs.push(persistence_pair(
                self,
                survivor,
                *root,
                *root,
                filtration[root.0 as usize],
                true,
            ));
        }
        pairs.sort_by(|a, b| {
            a.arbor_root
                .cmp(&b.arbor_root)
                .then_with(|| b.essential.cmp(&a.essential))
                .then_with(|| b.birth.total_cmp(&a.birth))
                .then_with(|| b.death.total_cmp(&a.death))
                .then_with(|| a.terminal_node.cmp(&b.terminal_node))
                .then_with(|| a.merge_node.cmp(&b.merge_node))
        });
        let non_monotone_pair_count = pairs.iter().filter(|pair| pair.non_monotone).count() as u32;
        let nodes: Vec<NodeIx> = view.nodes().collect();
        Ok(TmdResult {
            schema_version: 1,
            morphology_fingerprint: self.fingerprint().to_owned(),
            topology_fingerprint: self.topology_fingerprint().to_owned(),
            selection_fingerprint: view.fingerprint().to_owned(),
            selection: view.query.clone(),
            filtration: options.filtration,
            center,
            units: self.units().to_owned(),
            pairs,
            node_ids: nodes.iter().map(|node| self.id(*node).0).collect(),
            filtration_values: nodes
                .iter()
                .map(|node| filtration[node.0 as usize])
                .collect(),
            non_monotone_pair_count,
            provenance: TmdProvenance {
                definition_version: TMD_DEFINITION_VERSION,
                algorithm: "iterative-leaf-to-root-elder-rule-v1".to_owned(),
                elder_rule: "larger-terminal-filtration-survives".to_owned(),
                tie_rule: "lower-terminal-node-id-survives".to_owned(),
                root_connector_rule: match (options.filtration, center) {
                    (TmdFiltration::RootPathLength, None) => {
                        "selected-arbor-root-has-zero-path-distance".to_owned()
                    }
                    (TmdFiltration::RadialDistance, Some(TmdCenter::Soma)) => {
                        "euclidean-distance-from-soma-center".to_owned()
                    }
                    (TmdFiltration::RadialDistance, Some(TmdCenter::Root)) => {
                        "euclidean-distance-from-each-selected-arbor-root".to_owned()
                    }
                    _ => unreachable!("filtration and resolved center are validated above"),
                },
                pair_order: "arbor-root-ascending; essential-first; birth-descending; death-descending; terminal-node-ascending; merge-node-ascending".to_owned(),
                reference:
                    "Kanari et al. (2018), Neuroinformatics 16:3-13, doi:10.1007/s12021-017-9341-1"
                        .to_owned(),
            },
        })
    }
}

fn persistence_pair(
    morphology: &Morphology,
    component: ActiveComponent,
    merge: NodeIx,
    root: NodeIx,
    death: f64,
    essential: bool,
) -> PersistencePair {
    let birth = component.birth;
    PersistencePair {
        birth,
        death,
        persistence: (birth - death).abs(),
        start: birth.min(death),
        end: birth.max(death),
        terminal_node: morphology.id(component.terminal).0,
        merge_node: morphology.id(merge).0,
        arbor_root: morphology.id(root).0,
        essential,
        non_monotone: birth < death,
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnalysisDomain, ValidationProfile, parse_swc};

    use super::*;

    fn strict(source: &str) -> Morphology {
        parse_swc(source, ValidationProfile::IncfStrict)
            .morphology
            .unwrap()
    }

    #[test]
    fn path_tmd_applies_the_elder_rule_and_keeps_the_essential_bar() {
        let morphology =
            strict("1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 0 0 1 2\n4 3 3 0 0 1 3\n5 3 2 1 0 1 3\n");
        let result = morphology
            .tmd(&TmdOptions {
                selection: SelectionQuery {
                    domain: AnalysisDomain::Neurites,
                    ..Default::default()
                },
                filtration: TmdFiltration::RootPathLength,
                center: None,
            })
            .unwrap();
        assert_eq!(result.pairs.len(), 2);
        assert_eq!(result.pairs.iter().filter(|pair| pair.essential).count(), 1);
        assert_eq!(
            result
                .pairs
                .iter()
                .map(|pair| pair.terminal_node)
                .collect::<Vec<_>>(),
            vec![4, 5]
        );
        assert!(result.pairs[0].essential);
        let essential = result.pairs.iter().find(|pair| pair.essential).unwrap();
        assert_eq!(essential.terminal_node, 4);
        assert!((essential.birth - 2.0).abs() < 1e-12);
        let killed = result.pairs.iter().find(|pair| !pair.essential).unwrap();
        assert_eq!(killed.terminal_node, 5);
        assert!((killed.death - 1.0).abs() < 1e-12);
        assert_eq!(result.center, None);
        assert_eq!(result.provenance.definition_version, 2);
        assert_eq!(
            result.provenance.pair_order,
            "arbor-root-ascending; essential-first; birth-descending; death-descending; terminal-node-ascending; merge-node-ascending"
        );
    }

    #[test]
    fn radial_tmd_reports_non_monotone_bars_without_hiding_them() {
        let morphology = strict("1 1 0 0 0 1 -1\n2 3 2 0 0 1 1\n3 3 1 0 0 1 2\n4 3 3 0 0 1 2\n");
        let result = morphology.tmd(&TmdOptions::default()).unwrap();
        assert_eq!(result.non_monotone_pair_count, 1);
        assert!(result.pairs.iter().any(|pair| pair.non_monotone));
        assert_eq!(result.center, Some(TmdCenter::Soma));
    }

    #[test]
    fn root_centered_radial_tmd_uses_each_selected_arbor_root() {
        let morphology = strict("1 1 0 0 0 1 -1\n2 3 2 0 0 1 1\n3 3 5 0 0 1 2\n");
        let result = morphology
            .tmd(&TmdOptions {
                selection: SelectionQuery {
                    domain: AnalysisDomain::Neurites,
                    ..Default::default()
                },
                filtration: TmdFiltration::RadialDistance,
                center: Some(TmdCenter::Root),
            })
            .unwrap();
        assert_eq!(result.center, Some(TmdCenter::Root));
        assert_eq!(result.node_ids, vec![2, 3]);
        assert_eq!(result.filtration_values, vec![0.0, 3.0]);
        assert_eq!(
            result.provenance.root_connector_rule,
            "euclidean-distance-from-each-selected-arbor-root"
        );
    }

    #[test]
    fn path_tmd_rejects_a_center_instead_of_silently_changing_its_origin() {
        let morphology = strict("1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 0 0 1 2\n");
        let error = morphology
            .tmd(&TmdOptions {
                selection: SelectionQuery {
                    domain: AnalysisDomain::Neurites,
                    ..Default::default()
                },
                filtration: TmdFiltration::RootPathLength,
                center: Some(TmdCenter::Soma),
            })
            .unwrap_err();
        assert_eq!(error, TmdError::CenterNotApplicable);
    }
}
