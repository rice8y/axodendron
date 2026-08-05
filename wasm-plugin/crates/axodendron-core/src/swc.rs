use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::diagnostic::{Diagnostic, Severity, ValidationProfile};
use crate::geometry::Vec3;
use crate::model::{Morphology, NONE_NODE, SomaClass, SwcMetadata, fingerprint_bytes};

pub const MAX_NODE_COUNT: usize = 250_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParseResult {
    pub morphology: Option<Morphology>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    pub fn is_valid(&self) -> bool {
        self.morphology.is_some()
            && !self
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error)
    }
}

#[derive(Clone, Debug)]
struct Row {
    id: i64,
    kind: i32,
    position: Vec3,
    radius: f64,
    parent_id: i64,
    line: u32,
}

pub fn parse_swc(source: &str, profile: ValidationProfile) -> ParseResult {
    let mut rows = Vec::new();
    let mut diagnostics = Vec::new();
    let mut metadata = SwcMetadata::default();

    for (zero_line, raw_line) in source.lines().enumerate() {
        let line = (zero_line + 1) as u32;
        if let Some(comment) = raw_line.split_once('#').map(|(_, comment)| comment.trim()) {
            if !comment.is_empty() {
                metadata.comments.push(comment.to_owned());
                if let Some((key, value)) = metadata_field(comment) {
                    metadata.fields.entry(key).or_default().push(value);
                }
            }
        }
        let content = raw_line.split('#').next().unwrap_or_default().trim();
        if content.is_empty() {
            continue;
        }
        let columns: Vec<(&str, u32)> = content
            .split_whitespace()
            .map(|token| {
                let byte_offset = token.as_ptr() as usize - raw_line.as_ptr() as usize;
                let column = raw_line[..byte_offset].chars().count() + 1;
                (token, column as u32)
            })
            .collect();
        if columns.len() != 7 {
            let column = columns
                .get(7)
                .map_or(raw_line.chars().count() as u32 + 1, |item| item.1);
            diagnostics.push(
                Diagnostic::error(
                    "SWC_COLUMN_COUNT",
                    format!("expected 7 columns, found {}", columns.len()),
                )
                .at_line(line)
                .at_column(column),
            );
            continue;
        }

        let id = parse_integer(
            columns[0].0,
            "node id",
            line,
            columns[0].1,
            &mut diagnostics,
        );
        let kind = parse_integer(columns[1].0, "type", line, columns[1].1, &mut diagnostics)
            .and_then(|value| {
                i32::try_from(value)
                    .map_err(|_| {
                        diagnostics.push(
                            Diagnostic::error(
                                "SWC_TYPE_RANGE",
                                "type is outside the signed 32-bit range",
                            )
                            .at_line(line)
                            .at_column(columns[1].1),
                        );
                    })
                    .ok()
            });
        let x = parse_number(columns[2].0, "x", line, columns[2].1, &mut diagnostics);
        let y = parse_number(columns[3].0, "y", line, columns[3].1, &mut diagnostics);
        let z = parse_number(columns[4].0, "z", line, columns[4].1, &mut diagnostics);
        let radius = parse_number(columns[5].0, "radius", line, columns[5].1, &mut diagnostics);
        let parent_id = parse_integer(
            columns[6].0,
            "parent id",
            line,
            columns[6].1,
            &mut diagnostics,
        );
        if let (Some(id), Some(kind), Some(x), Some(y), Some(z), Some(radius), Some(parent_id)) =
            (id, kind, x, y, z, radius, parent_id)
        {
            if rows.len() == MAX_NODE_COUNT {
                diagnostics.push(
                    Diagnostic::error(
                        "SWC_NODE_LIMIT",
                        format!("morphology exceeds the {MAX_NODE_COUNT}-node limit"),
                    )
                    .at_line(line),
                );
                break;
            }
            if id <= 0 {
                diagnostics.push(
                    Diagnostic::error("SWC_ID_NONPOSITIVE", "node id must be a positive integer")
                        .at_line(line)
                        .at_column(columns[0].1)
                        .for_node(id),
                );
            }
            if kind < 0 {
                diagnostics.push(
                    Diagnostic::error(
                        "SWC_TYPE_NEGATIVE",
                        "type must be zero or a positive integer",
                    )
                    .at_line(line)
                    .at_column(columns[1].1)
                    .for_node(id),
                );
            }
            if radius <= 0.0 {
                diagnostics.push(
                    Diagnostic::warning("SWC_RADIUS_NONPOSITIVE", "radius should be positive")
                        .at_line(line)
                        .at_column(columns[5].1)
                        .for_node(id),
                );
            }
            rows.push(Row {
                id,
                kind,
                position: Vec3::new(x, y, z),
                radius,
                parent_id,
                line,
            });
        }
    }

    if rows.is_empty() {
        diagnostics.push(Diagnostic::error(
            "SWC_EMPTY",
            "no SWC data rows were found",
        ));
        return ParseResult {
            morphology: None,
            diagnostics,
        };
    }

    let mut custom_types = BTreeMap::<i32, (u32, usize)>::new();
    for (ix, row) in rows.iter().enumerate() {
        if row.kind > 7 {
            let entry = custom_types.entry(row.kind).or_insert((0, ix));
            entry.0 += 1;
        }
    }
    for (kind, (count, first)) in custom_types {
        let row = &rows[first];
        diagnostics.push(
            Diagnostic::info(
                "SWC_CUSTOM_TYPE",
                format!("preserving custom SWC type {kind} on {count} node(s)"),
            )
            .at_line(row.line)
            .for_node(row.id),
        );
    }

    let mut by_id = HashMap::with_capacity(rows.len());
    for (ix, row) in rows.iter().enumerate() {
        if let Some(previous) = by_id.insert(row.id, ix as u32) {
            diagnostics.push(
                Diagnostic::error(
                    "SWC_DUPLICATE_ID",
                    format!(
                        "node id {} duplicates line {}",
                        row.id, rows[previous as usize].line
                    ),
                )
                .at_line(row.line)
                .for_node(row.id),
            );
        }
    }

    if profile == ValidationProfile::IncfStrict {
        for (ix, row) in rows.iter().enumerate() {
            let expected = ix as i64 + 1;
            if row.id != expected {
                diagnostics.push(
                    Diagnostic::error(
                        "SWC_STRICT_ID_SEQUENCE",
                        format!(
                            "strict profile expects node id {expected} at data row {}",
                            ix + 1
                        ),
                    )
                    .at_line(row.line)
                    .for_node(row.id),
                );
            }
            if ix == 0 && row.parent_id != -1 {
                diagnostics.push(
                    Diagnostic::error(
                        "SWC_STRICT_FIRST_ROOT",
                        "first data row must have parent -1",
                    )
                    .at_line(row.line)
                    .for_node(row.id),
                );
            }
            if row.parent_id >= 0 {
                match by_id.get(&row.parent_id) {
                    Some(parent_ix) if (*parent_ix as usize) < ix => {}
                    Some(_) => diagnostics.push(
                        Diagnostic::error(
                            "SWC_STRICT_PARENT_ORDER",
                            "parent must occur before its child in strict profile",
                        )
                        .at_line(row.line)
                        .for_node(row.id),
                    ),
                    None => {}
                }
            } else if row.parent_id != -1 {
                diagnostics.push(
                    Diagnostic::error("SWC_STRICT_ROOT_SENTINEL", "root parent must be exactly -1")
                        .at_line(row.line)
                        .for_node(row.id),
                );
            }
        }
    }

    let mut parents = Vec::with_capacity(rows.len());
    for row in &rows {
        if row.parent_id < 0 {
            if profile == ValidationProfile::Permissive && row.parent_id != -1 {
                diagnostics.push(
                    Diagnostic::warning(
                        "SWC_NONSTANDARD_ROOT_SENTINEL",
                        format!("treating parent {} as a root sentinel", row.parent_id),
                    )
                    .at_line(row.line)
                    .for_node(row.id),
                );
            }
            parents.push(NONE_NODE);
        } else if row.parent_id == row.id {
            diagnostics.push(
                Diagnostic::error("SWC_SELF_PARENT", "node cannot be its own parent")
                    .at_line(row.line)
                    .for_node(row.id),
            );
            parents.push(NONE_NODE);
        } else if let Some(parent) = by_id.get(&row.parent_id) {
            parents.push(*parent);
        } else {
            diagnostics.push(
                Diagnostic::error(
                    "SWC_MISSING_PARENT",
                    format!("parent node {} does not exist", row.parent_id),
                )
                .at_line(row.line)
                .for_node(row.id),
            );
            parents.push(NONE_NODE);
        }
    }

    detect_cycles(&rows, &parents, &mut diagnostics);
    for (child, parent) in parents.iter().copied().enumerate() {
        if parent != NONE_NODE
            && rows[child]
                .position
                .distance(rows[parent as usize].position)
                == 0.0
        {
            diagnostics.push(
                Diagnostic::warning(
                    "SWC_ZERO_LENGTH_EDGE",
                    "edge has identical endpoint coordinates",
                )
                .at_line(rows[child].line)
                .for_node(rows[child].id),
            );
        }
    }

    let roots: Vec<usize> = parents
        .iter()
        .enumerate()
        .filter_map(|(ix, parent)| (*parent == NONE_NODE).then_some(ix))
        .collect();
    if roots.len() != 1 {
        let message = format!("morphology has {} roots; expected one", roots.len());
        let diagnostic = match profile {
            ValidationProfile::IncfStrict => Diagnostic::error("SWC_MULTIPLE_ROOTS", message),
            ValidationProfile::Permissive => Diagnostic::warning("SWC_MULTIPLE_ROOTS", message),
        };
        diagnostics.push(diagnostic);
        let message = format!(
            "morphology contains {} disconnected components",
            roots.len()
        );
        diagnostics.push(match profile {
            ValidationProfile::IncfStrict => {
                Diagnostic::error("SWC_DISCONNECTED_COMPONENT", message)
            }
            ValidationProfile::Permissive => {
                Diagnostic::warning("SWC_DISCONNECTED_COMPONENT", message)
            }
        });
    }

    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return ParseResult {
            morphology: None,
            diagnostics,
        };
    }

    let morphology = Morphology::from_parts(
        rows.iter().map(|row| row.id).collect(),
        rows.iter().map(|row| row.kind).collect(),
        rows.iter().map(|row| row.position).collect(),
        rows.iter().map(|row| row.radius).collect(),
        parents,
        rows.iter().map(|row| row.line).collect(),
        "um".to_owned(),
        Some(fingerprint_bytes(source.as_bytes())),
        metadata,
    );
    match morphology.soma_class() {
        SomaClass::Ambiguous => diagnostics.push(Diagnostic::warning(
            "SWC_SOMA_AMBIGUOUS",
            "three type-1 nodes resemble a three-point soma but fail its geometric invariants",
        )),
        SomaClass::Disconnected => diagnostics.push(Diagnostic::warning(
            "SWC_SOMA_DISCONNECTED",
            "type-1 nodes form more than one soma subgraph",
        )),
        SomaClass::MultiPointChain | SomaClass::Branched => diagnostics.push(Diagnostic::info(
            "SWC_SOMA_NONSTANDARD",
            "preserving a non-single-point, non-three-point soma representation",
        )),
        SomaClass::Absent | SomaClass::SinglePoint | SomaClass::ThreePoint => {}
    }
    ParseResult {
        morphology: Some(morphology),
        diagnostics,
    }
}

fn metadata_field(comment: &str) -> Option<(String, String)> {
    const KNOWN: &[&str] = &[
        "original_source",
        "creature",
        "region",
        "field/layer",
        "type",
        "class",
        "contributor",
        "reference",
        "raw",
        "extras",
        "soma_area",
        "shrinkage_correction",
        "version_number",
        "version_date",
        "scale",
        "sex",
        "age",
        "condition",
        "label",
        "slicing",
        "microscopy",
        "coordinate",
        "brainspace",
    ];
    let (raw_key, raw_value) = comment
        .split_once(':')
        .or_else(|| comment.split_once('='))
        .or_else(|| comment.split_once(char::is_whitespace))?;
    let normalized = raw_key.trim().to_ascii_lowercase().replace('-', "_");
    if !KNOWN.contains(&normalized.as_str()) {
        return None;
    }
    let value = raw_value.trim();
    (!value.is_empty()).then(|| (normalized.replace('_', "-"), value.to_owned()))
}

fn parse_integer(
    token: &str,
    label: &str,
    line: u32,
    column: u32,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match token.parse::<i64>() {
        Ok(value) => Some(value),
        Err(_) => {
            diagnostics.push(
                Diagnostic::error("SWC_INVALID_INTEGER", format!("{label} is not an integer"))
                    .at_line(line)
                    .at_column(column),
            );
            None
        }
    }
}

fn parse_number(
    token: &str,
    label: &str,
    line: u32,
    column: u32,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<f64> {
    match token.parse::<f64>() {
        Ok(value) if value.is_finite() => Some(value),
        Ok(_) => {
            diagnostics.push(
                Diagnostic::error("SWC_NONFINITE_NUMBER", format!("{label} must be finite"))
                    .at_line(line)
                    .at_column(column),
            );
            None
        }
        Err(_) => {
            diagnostics.push(
                Diagnostic::error("SWC_INVALID_NUMBER", format!("{label} is not a number"))
                    .at_line(line)
                    .at_column(column),
            );
            None
        }
    }
}

fn detect_cycles(rows: &[Row], parents: &[u32], diagnostics: &mut Vec<Diagnostic>) {
    let mut state = vec![0_u8; rows.len()];
    for start in 0..rows.len() {
        if state[start] == 2 {
            continue;
        }
        let mut path = Vec::new();
        let mut cursor = start as u32;
        while cursor != NONE_NODE && state[cursor as usize] == 0 {
            state[cursor as usize] = 1;
            path.push(cursor);
            cursor = parents[cursor as usize];
        }
        if cursor != NONE_NODE && state[cursor as usize] == 1 {
            diagnostics.push(
                Diagnostic::error("SWC_CYCLE", "parent references contain a cycle")
                    .at_line(rows[cursor as usize].line)
                    .for_node(rows[cursor as usize].id),
            );
        }
        for ix in path {
            state[ix as usize] = 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;

    const SIMPLE: &str =
        "# example\n1 1 0 0 0 2 -1\n2 3 1 0 0 1 1\n3 3 2 1 0 0.8 2\n4 3 2 -1 0 0.8 2\n";

    #[test]
    fn parses_strict_tree() {
        let result = parse_swc(SIMPLE, ValidationProfile::IncfStrict);
        assert!(result.is_valid(), "{:?}", result.diagnostics);
        let morphology = result.morphology.unwrap();
        assert_eq!(morphology.len(), 4);
        assert_eq!(
            morphology.child_count(morphology.index_of(NodeId(2)).unwrap()),
            2
        );
    }

    #[test]
    fn rejects_missing_parent() {
        let result = parse_swc(
            "1 1 0 0 0 1 -1\n2 3 1 0 0 1 99\n",
            ValidationProfile::Permissive,
        );
        assert!(!result.is_valid());
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "SWC_MISSING_PARENT")
        );
    }

    #[test]
    fn permissive_accepts_out_of_order_ids() {
        let result = parse_swc(
            "20 3 1 0 0 1 10\n10 1 0 0 0 1 -1\n",
            ValidationProfile::Permissive,
        );
        assert!(result.is_valid(), "{:?}", result.diagnostics);
    }

    #[test]
    fn permissive_preserves_custom_types_and_forests() {
        let result = parse_swc(
            "10 1 0 0 0 1 -1\n20 42 1 0 0 1 -1\n",
            ValidationProfile::Permissive,
        );
        assert!(result.is_valid(), "{:?}", result.diagnostics);
        assert_eq!(result.morphology.unwrap().kinds(), &[1, 42]);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "SWC_CUSTOM_TYPE")
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "SWC_DISCONNECTED_COMPONENT")
        );
    }

    #[test]
    fn rejects_parent_cycles() {
        let result = parse_swc(
            "1 3 0 0 0 1 2\n2 3 1 0 0 1 1\n",
            ValidationProfile::Permissive,
        );
        assert!(!result.is_valid());
        assert!(result.diagnostics.iter().any(|d| d.code == "SWC_CYCLE"));
    }
}
