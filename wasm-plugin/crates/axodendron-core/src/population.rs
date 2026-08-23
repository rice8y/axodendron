use serde::{Deserialize, Serialize};

use crate::analysis::SectionBoundaryPolicy;
use crate::metrics::{
    MeasureOptions, MetricData, MetricDescriptor, MetricError, MetricResult, MetricSpec,
    MetricValue,
};
use crate::model::Morphology;
use crate::query::SelectionQuery;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldAggregate {
    Mean,
    Median,
    Minimum,
    Maximum,
    Sum,
    SampleVariance,
    PopulationVariance,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldMissingPolicy {
    /// Reject a field aggregate if any selected entity has an undefined value.
    #[default]
    Strict,
    /// Aggregate only defined values while retaining the policy in column metadata.
    Omit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureColumnSpec {
    #[serde(default)]
    pub name: Option<String>,
    pub metric: MetricSpec,
    #[serde(default)]
    pub aggregate: Option<FieldAggregate>,
    #[serde(default)]
    pub component: Option<FeatureComponent>,
    #[serde(default)]
    pub missing_policy: FieldMissingPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureComponent {
    X,
    Y,
    Z,
    MinX,
    MinY,
    MinZ,
    MaxX,
    MaxY,
    MaxZ,
    SpanX,
    SpanY,
    SpanZ,
    Major,
    Middle,
    Minor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureTableOptions {
    pub columns: Vec<FeatureColumnSpec>,
    #[serde(default)]
    pub selection: SelectionQuery,
    #[serde(default)]
    pub section_boundaries: SectionBoundaryPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PopulationMorphology {
    pub id: String,
    pub morphology: Morphology,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum FeatureCell {
    Value { value: f64 },
    Missing { reason: String, detail: String },
}

impl FeatureCell {
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Value { value } => Some(*value),
            Self::Missing { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureColumn {
    pub name: String,
    pub metric: MetricDescriptor,
    pub aggregate: Option<FieldAggregate>,
    pub component: Option<FeatureComponent>,
    pub missing_policy: FieldMissingPolicy,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureRow {
    pub id: String,
    pub morphology_fingerprint: String,
    pub values: Vec<FeatureCell>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureSummary {
    pub column: String,
    pub valid_count: u32,
    pub missing_count: u32,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub sample_variance: Option<f64>,
    pub population_variance: Option<f64>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureTable {
    pub schema_version: u16,
    pub columns: Vec<FeatureColumn>,
    pub rows: Vec<FeatureRow>,
    pub summaries: Vec<FeatureSummary>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PopulationError {
    Empty,
    EmptyColumns,
    DuplicateId(String),
    DuplicateColumn(String),
    IncompatibleColumn { column: String, detail: String },
    Metric(MetricError),
}

impl std::fmt::Display for PopulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("population must contain at least one morphology"),
            Self::EmptyColumns => f.write_str("feature table must contain at least one column"),
            Self::DuplicateId(id) => write!(f, "population morphology id {id:?} is duplicated"),
            Self::DuplicateColumn(name) => write!(f, "feature column name {name:?} is duplicated"),
            Self::IncompatibleColumn { column, detail } => {
                write!(f, "feature column {column:?} is not comparable: {detail}")
            }
            Self::Metric(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for PopulationError {}

impl From<MetricError> for PopulationError {
    fn from(value: MetricError) -> Self {
        Self::Metric(value)
    }
}

pub fn feature_table(
    population: &[PopulationMorphology],
    options: &FeatureTableOptions,
) -> Result<FeatureTable, PopulationError> {
    if population.is_empty() {
        return Err(PopulationError::Empty);
    }
    if options.columns.is_empty() {
        return Err(PopulationError::EmptyColumns);
    }
    let mut ids = std::collections::HashSet::new();
    for item in population {
        if !ids.insert(item.id.clone()) {
            return Err(PopulationError::DuplicateId(item.id.clone()));
        }
    }
    let names: Vec<String> = options.columns.iter().map(column_name).collect();
    let mut unique_names = std::collections::HashSet::new();
    for name in &names {
        if !unique_names.insert(name.clone()) {
            return Err(PopulationError::DuplicateColumn(name.clone()));
        }
    }

    let mut rows = Vec::with_capacity(population.len());
    let mut columns: Option<Vec<FeatureColumn>> = None;
    for item in population {
        let results = item.morphology.measure(&MeasureOptions {
            metrics: options
                .columns
                .iter()
                .map(|column| column.metric.clone())
                .collect(),
            selection: options.selection.clone(),
            section_boundaries: options.section_boundaries,
        })?;
        if columns.is_none() {
            columns = Some(
                results
                    .iter()
                    .zip(&options.columns)
                    .zip(&names)
                    .map(|((result, spec), name)| FeatureColumn {
                        name: name.clone(),
                        metric: result.metric.clone(),
                        aggregate: spec.aggregate,
                        component: spec.component,
                        missing_policy: spec.missing_policy,
                        units: result_units(result),
                    })
                    .collect(),
            );
        }
        let established = columns
            .as_ref()
            .expect("non-empty population initializes columns");
        for ((result, column), name) in results.iter().zip(established).zip(&names) {
            if result.metric != column.metric {
                return Err(PopulationError::IncompatibleColumn {
                    column: name.clone(),
                    detail: "resolved metric ID, definition version, or parameters differ"
                        .to_owned(),
                });
            }
            let units = result_units(result);
            if units != column.units {
                return Err(PopulationError::IncompatibleColumn {
                    column: name.clone(),
                    detail: format!("units differ ({:?} versus {:?})", column.units, units),
                });
            }
        }
        let values = results
            .iter()
            .zip(&options.columns)
            .zip(&names)
            .map(|((result, column), name)| feature_cell(result, column, name))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(FeatureRow {
            id: item.id.clone(),
            morphology_fingerprint: item.morphology.fingerprint().to_owned(),
            values,
        });
    }
    let columns = columns.expect("non-empty population initializes columns");
    let summaries = summarize(&columns, &rows);
    Ok(FeatureTable {
        schema_version: 1,
        columns,
        rows,
        summaries,
    })
}

pub fn feature_table_csv(table: &FeatureTable) -> String {
    let mut output = String::new();
    output.push_str("id,morphology-fingerprint");
    for column in &table.columns {
        output.push(',');
        output.push_str(&csv_escape(&column.name));
    }
    output.push('\n');
    for row in &table.rows {
        output.push_str(&csv_escape(&row.id));
        output.push(',');
        output.push_str(&csv_escape(&row.morphology_fingerprint));
        for value in &row.values {
            output.push(',');
            if let Some(value) = value.value() {
                output.push_str(&canonical_number(value));
            }
        }
        output.push('\n');
    }
    output
}

fn column_name(column: &FeatureColumnSpec) -> String {
    let base = column
        .name
        .clone()
        .unwrap_or_else(|| match column.aggregate {
            Some(aggregate) => format!("{}--{}", column.metric.id, enum_text(aggregate)),
            None => column.metric.id.clone(),
        });
    if column.name.is_none() {
        column.component.map_or(base.clone(), |component| {
            format!("{base}--{}", enum_text(component))
        })
    } else {
        base
    }
}

fn feature_cell(
    result: &MetricResult,
    column: &FeatureColumnSpec,
    name: &str,
) -> Result<FeatureCell, PopulationError> {
    if let Some(missing) = result.missing.first() {
        if matches!(result.data, MetricData::MorphologyMetric(_)) {
            return Ok(FeatureCell::Missing {
                reason: enum_text(missing.reason),
                detail: missing.detail.clone(),
            });
        }
    }
    let incompatible = |detail: &str| PopulationError::IncompatibleColumn {
        column: name.to_owned(),
        detail: detail.to_owned(),
    };
    match (&result.data, column.aggregate) {
        (MetricData::MorphologyMetric(metric), None) => match metric.value {
            Some(MetricValue::Scalar(value)) if column.component.is_none() => {
                Ok(FeatureCell::Value { value })
            }
            Some(MetricValue::Scalar(_)) => Err(incompatible(
                "a scalar morphology metric must not specify `component`",
            )),
            Some(MetricValue::Vector3(value)) => {
                let component = column.component.ok_or_else(|| {
                    incompatible("a vector morphology metric requires an explicit `component`")
                })?;
                let value = match (result.metric.id.as_str(), component) {
                    ("centroid", FeatureComponent::X) => value.x,
                    ("centroid", FeatureComponent::Y) => value.y,
                    ("centroid", FeatureComponent::Z) => value.z,
                    ("principal-extents", FeatureComponent::Major) => value.x,
                    ("principal-extents", FeatureComponent::Middle) => value.y,
                    ("principal-extents", FeatureComponent::Minor) => value.z,
                    ("centroid", _) => {
                        return Err(incompatible("centroid components are `x`, `y`, or `z`"));
                    }
                    ("principal-extents", _) => {
                        return Err(incompatible(
                            "principal-extents components are `major`, `middle`, or `minor`",
                        ));
                    }
                    _ => return Err(incompatible("unsupported vector metric component")),
                };
                Ok(FeatureCell::Value { value })
            }
            Some(MetricValue::Box3(value)) => {
                let component = column.component.ok_or_else(|| {
                    incompatible("a box morphology metric requires an explicit `component`")
                })?;
                let spans = value.spans();
                let value = match component {
                    FeatureComponent::MinX => value.min.x,
                    FeatureComponent::MinY => value.min.y,
                    FeatureComponent::MinZ => value.min.z,
                    FeatureComponent::MaxX => value.max.x,
                    FeatureComponent::MaxY => value.max.y,
                    FeatureComponent::MaxZ => value.max.z,
                    FeatureComponent::SpanX => spans.x,
                    FeatureComponent::SpanY => spans.y,
                    FeatureComponent::SpanZ => spans.z,
                    _ => {
                        return Err(incompatible(
                            "bounding-box components are `min-*`, `max-*`, or `span-*`",
                        ));
                    }
                };
                Ok(FeatureCell::Value { value })
            }
            None => Ok(FeatureCell::Missing {
                reason: "undefined".to_owned(),
                detail: result
                    .missing
                    .first()
                    .map(|item| item.detail.clone())
                    .unwrap_or_else(|| "metric has no value".to_owned()),
            }),
        },
        (MetricData::MorphologyMetric(_), Some(_)) => Err(incompatible(
            "a morphology metric must not specify a field aggregation",
        )),
        (MetricData::NodeField(field), Some(aggregate)) => {
            if column.component.is_some() {
                return Err(incompatible("an entity field must not specify `component`"));
            }
            Ok(aggregate_field(
                &field.values,
                result,
                aggregate,
                column.missing_policy,
            ))
        }
        (MetricData::SectionField(field), Some(aggregate)) => {
            if column.component.is_some() {
                return Err(incompatible("an entity field must not specify `component`"));
            }
            Ok(aggregate_field(
                &field.values,
                result,
                aggregate,
                column.missing_policy,
            ))
        }
        (MetricData::BifurcationField(field), Some(aggregate)) => {
            if column.component.is_some() {
                return Err(incompatible("an entity field must not specify `component`"));
            }
            Ok(aggregate_field(
                &field.values,
                result,
                aggregate,
                column.missing_policy,
            ))
        }
        (_, None) => Err(incompatible(
            "node, section, and bifurcation fields require an explicit aggregation",
        )),
    }
}

fn aggregate_field(
    values: &[f64],
    result: &MetricResult,
    aggregate: FieldAggregate,
    missing_policy: FieldMissingPolicy,
) -> FeatureCell {
    if missing_policy == FieldMissingPolicy::Strict && !result.missing.is_empty() {
        return FeatureCell::Missing {
            reason: "partial-field".to_owned(),
            detail: format!(
                "{} selected field entities are undefined; use missing-policy `omit` to aggregate only defined values",
                result.missing.len()
            ),
        };
    }
    aggregate_values(values, aggregate)
}

fn aggregate_values(values: &[f64], aggregate: FieldAggregate) -> FeatureCell {
    if values.is_empty() {
        return FeatureCell::Missing {
            reason: "empty-field".to_owned(),
            detail: "the selected field has no defined values".to_owned(),
        };
    }
    let value = match aggregate {
        FieldAggregate::Mean => values.iter().sum::<f64>() / values.len() as f64,
        FieldAggregate::Median => {
            let mut values = values.to_vec();
            values.sort_by(f64::total_cmp);
            quantile(&values, 0.5)
        }
        FieldAggregate::Minimum => values.iter().copied().fold(f64::INFINITY, f64::min),
        FieldAggregate::Maximum => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        FieldAggregate::Sum => values.iter().sum(),
        FieldAggregate::PopulationVariance => variance(values, false).unwrap_or(0.0),
        FieldAggregate::SampleVariance => match variance(values, true) {
            Some(value) => value,
            None => {
                return FeatureCell::Missing {
                    reason: "insufficient-sample".to_owned(),
                    detail: "sample variance requires at least two defined values".to_owned(),
                };
            }
        },
    };
    FeatureCell::Value { value }
}

fn result_units(result: &MetricResult) -> String {
    match &result.data {
        MetricData::MorphologyMetric(value) => value.units.clone(),
        MetricData::NodeField(value) => value.units.clone(),
        MetricData::SectionField(value) => value.units.clone(),
        MetricData::BifurcationField(value) => value.units.clone(),
    }
}

fn summarize(columns: &[FeatureColumn], rows: &[FeatureRow]) -> Vec<FeatureSummary> {
    columns
        .iter()
        .enumerate()
        .map(|(ix, column)| {
            let mut values: Vec<f64> = rows
                .iter()
                .filter_map(|row| row.values[ix].value())
                .collect();
            values.sort_by(f64::total_cmp);
            let valid = values.len();
            FeatureSummary {
                column: column.name.clone(),
                valid_count: valid as u32,
                missing_count: (rows.len() - valid) as u32,
                mean: (!values.is_empty())
                    .then(|| values.iter().sum::<f64>() / values.len() as f64),
                median: (!values.is_empty()).then(|| quantile(&values, 0.5)),
                sample_variance: variance(&values, true),
                population_variance: variance(&values, false),
                minimum: values.first().copied(),
                maximum: values.last().copied(),
            }
        })
        .collect()
}

fn variance(values: &[f64], sample: bool) -> Option<f64> {
    if values.is_empty() || (sample && values.len() < 2) {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let divisor = if sample {
        values.len() - 1
    } else {
        values.len()
    } as f64;
    Some(
        values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / divisor,
    )
}

fn quantile(sorted: &[f64], probability: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let position = probability * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = position - lower as f64;
    sorted[lower] + fraction * (sorted[upper] - sorted[lower])
}

fn enum_text<T: std::fmt::Debug>(value: T) -> String {
    let debug = format!("{value:?}");
    let mut text = String::new();
    for (ix, character) in debug.chars().enumerate() {
        if character.is_ascii_uppercase() && ix > 0 {
            text.push('-');
        }
        text.push(character.to_ascii_lowercase());
    }
    text
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn canonical_number(value: f64) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{AnalysisDomain, MetricParameters, ValidationProfile, parse_swc};

    use super::*;

    fn cell(id: &str, length: f64) -> PopulationMorphology {
        PopulationMorphology {
            id: id.to_owned(),
            morphology: parse_swc(
                &format!("1 1 0 0 0 1 -1\n2 3 {length} 0 0 1 1\n"),
                ValidationProfile::IncfStrict,
            )
            .morphology
            .unwrap(),
        }
    }

    #[test]
    fn feature_table_keeps_values_missingness_and_descriptive_statistics() {
        let table = feature_table(
            &[cell("a", 1.0), cell("b", 3.0)],
            &FeatureTableOptions {
                columns: vec![FeatureColumnSpec {
                    name: Some("x-span".to_owned()),
                    metric: MetricSpec {
                        id: "bounding-box".to_owned(),
                        parameters: MetricParameters::default(),
                    },
                    aggregate: None,
                    component: Some(FeatureComponent::SpanX),
                    missing_policy: FieldMissingPolicy::Strict,
                }],
                selection: SelectionQuery {
                    domain: AnalysisDomain::Raw,
                    ..Default::default()
                },
                section_boundaries: SectionBoundaryPolicy::TopologyOnly,
            },
        )
        .unwrap();
        assert_eq!(table.rows[0].values[0].value(), Some(1.0));
        assert_eq!(table.rows[1].values[0].value(), Some(3.0));
        assert_eq!(table.summaries[0].mean, Some(2.0));

        let csv = feature_table_csv(&table);
        assert!(csv.starts_with("id,morphology-fingerprint,x-span\n"));
    }

    #[test]
    fn field_columns_require_explicit_aggregation() {
        let population = [PopulationMorphology {
            id: "branch".to_owned(),
            morphology: parse_swc(
                "1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 1 1 0 1 1\n",
                ValidationProfile::IncfStrict,
            )
            .morphology
            .unwrap(),
        }];
        let make = |aggregate| FeatureTableOptions {
            columns: vec![FeatureColumnSpec {
                name: None,
                metric: MetricSpec {
                    id: "local-bifurcation-angle".to_owned(),
                    parameters: MetricParameters::default(),
                },
                aggregate,
                component: None,
                missing_policy: FieldMissingPolicy::Strict,
            }],
            selection: SelectionQuery {
                domain: AnalysisDomain::Raw,
                ..Default::default()
            },
            section_boundaries: SectionBoundaryPolicy::TopologyOnly,
        };
        assert!(matches!(
            feature_table(&population, &make(None)),
            Err(PopulationError::IncompatibleColumn { .. })
        ));
        let mean = feature_table(&population, &make(Some(FieldAggregate::Mean))).unwrap();
        assert!((mean.rows[0].values[0].value().unwrap() - 45.0).abs() < 1e-12);
    }

    #[test]
    fn partial_field_aggregation_is_strict_unless_omit_is_explicit() {
        let population = [PopulationMorphology {
            id: "partly-degenerate".to_owned(),
            morphology: parse_swc(
                "1 3 0 0 0 1 -1\n2 3 0 0 0 1 1\n3 3 1 0 0 1 1\n",
                ValidationProfile::IncfStrict,
            )
            .morphology
            .unwrap(),
        }];
        let make = |missing_policy| FeatureTableOptions {
            columns: vec![FeatureColumnSpec {
                name: None,
                metric: MetricSpec {
                    id: "local-bifurcation-angle".to_owned(),
                    parameters: MetricParameters::default(),
                },
                aggregate: Some(FieldAggregate::Mean),
                component: None,
                missing_policy,
            }],
            selection: SelectionQuery {
                domain: AnalysisDomain::Raw,
                ..Default::default()
            },
            section_boundaries: SectionBoundaryPolicy::TopologyOnly,
        };
        let strict = feature_table(&population, &make(FieldMissingPolicy::Strict)).unwrap();
        assert!(matches!(
            strict.rows[0].values[0],
            FeatureCell::Missing { ref reason, .. } if reason == "partial-field"
        ));
        let omit = feature_table(&population, &make(FieldMissingPolicy::Omit)).unwrap();
        assert!(matches!(
            omit.rows[0].values[0],
            FeatureCell::Missing { ref reason, .. } if reason == "empty-field"
        ));
        assert_eq!(omit.columns[0].missing_policy, FieldMissingPolicy::Omit);
    }
}
