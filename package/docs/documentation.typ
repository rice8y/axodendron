#import "@preview/mantys:1.0.2": *
#import "@preview/codly:1.3.0"
#import "@preview/cetz:0.5.2"
#import "../lib.typ" as swc

#let infos = toml("../typst.toml")

#let aa-bytes = read("../examples/data/AA0109.CNG.swc", encoding: none)
#let nr5a1-bytes = read("../examples/data/Nr5a1-Cre_Ai14-187777-05-02-01_491392821_m.kp1.swc", encoding: none)
#let sst-bytes = read("../examples/data/Sst-IRES-Cre_Ai14-188740-03-02-01_491119369_m.kp12.swc", encoding: none)
#let vipr2-bytes = read("../examples/data/Vipr2-IRES2-Cre_Ai14-310513-05-02-01_637021223_m.CNG.swc", encoding: none)

#let aa-cell = swc.load(aa-bytes, profile: "incf-strict")
#let nr5a1-cell = swc.load(nr5a1-bytes, profile: "incf-strict")
#let sst-cell = swc.load(sst-bytes, profile: "incf-strict")
#let vipr2-cell = swc.load(vipr2-bytes, profile: "incf-strict")

#let aa-analysis = swc.analyze(aa-cell)
#let nr5a1-analysis = swc.analyze(nr5a1-cell)
#let sst-analysis = swc.analyze(sst-cell)
#let vipr2-analysis = swc.analyze(vipr2-cell)

#let aa-local-angle = swc.measure(aa-cell, metrics: "local-bifurcation-angle").first()
#let aa-angle-nodes = swc.field-to-nodes(
  aa-cell,
  field: aa-local-angle,
  placement: "bifurcation-branch",
  reducer: "mean",
)
#let aa-angle-min = calc.min(..aa-angle-nodes.data.value.values)
#let aa-angle-max = calc.max(..aa-angle-nodes.data.value.values)
#let sst-tmd = swc.tmd(sst-cell, filtration: "root-path-length")
#let sst-tmd-scale = swc.persistence-scale(sst-tmd)
#let example-population = swc.population((
  swc.population-entry("AA0109", cell: aa-cell),
  swc.population-entry("Nr5a1", cell: nr5a1-cell),
  swc.population-entry("Sst", cell: sst-cell),
))
#let example-features = swc.feature-table(
  example-population,
  columns: (
    swc.feature-column("fractional-anisotropy", name: "FA"),
    swc.feature-column(
      "local-bifurcation-angle",
      name: "mean local angle",
      aggregate: "mean",
      missing-policy: "omit",
    ),
    swc.feature-column("centroid", name: "centroid x", component: "x"),
  ),
)

#let field-maximum(field) = field.values.fold(0, (current, value) => calc.max(current, value))
#let aa-branch-maximum = field-maximum(aa-analysis.branch_order)
#let aa-path-maximum = calc.ceil(field-maximum(aa-analysis.root_path_length))

#let source-note(record, filename, archive, study) = block(width: 100%, inset: (top: 3pt))[
  #set text(size: 7.4pt, fill: luma(38%))
  *Data:* #raw(filename); NeuroMorpho.Org record #record; #archive; #study; CC BY 4.0. Cite NeuroMorpho.Org, RRID:SCR_002145.
]

#let aa-source = source-note(
  "85226",
  "AA0109.CNG.swc",
  [MouseLight],
  [doi:10.1002/jnr.23978 and reconstruction deposit doi:10.25378/janelia.5526706],
)

#let nr5a1-source = source-note(
  "62390",
  "Nr5a1-Cre_Ai14-187777-05-02-01_491392821_m.kp1.swc",
  [Allen Cell Types],
  [doi:10.1016/j.neuron.2015.02.022],
)

#let sst-source = source-note(
  "62495",
  "Sst-IRES-Cre_Ai14-188740-03-02-01_491119369_m.kp12.swc",
  [Allen Cell Types],
  [doi:10.1016/j.neuron.2015.02.022],
)

#let vipr2-source = source-note(
  "102520",
  "Vipr2-IRES2-Cre_Ai14-310513-05-02-01_637021223_m.CNG.swc",
  [Allen Cell Types],
  [doi:10.1016/j.neuron.2015.02.022],
)

#let attributed(body, source) = stack(dir: ttb, spacing: 2mm, body, source)

#let breakable-mono(value, size: 8pt) = {
  set text(font: ("Menlo", "Courier New"), size: size)
  value
    .replace("_", "_\u{200b}")
    .replace("-", "-\u{200b}")
    .replace(".", ".\u{200b}")
}

#let manual-theme = create-theme(
  fonts: (
    serif: ("Times New Roman", "Georgia"),
    sans: ("Helvetica Neue", "Arial"),
    mono: ("Menlo", "Courier New"),
  ),
  text: (
    size: 11pt,
    font: ("Times New Roman", "Georgia"),
    fill: rgb(35, 31, 32),
  ),
  heading: (
    font: ("Helvetica Neue", "Arial"),
    fill: rgb(35, 31, 32),
  ),
  emph: (
    link: rgb("#1f4f73"),
  ),
  code: (
    size: 9pt,
    font: ("Menlo", "Courier New"),
    fill: rgb("#555555"),
  ),
)

#show: mantys(
  ..infos,
  title: [#infos.package.name],
  subtitle: [Deterministic neuronal morphology analysis and rendering from SWC],
  date: datetime.today(),
  abstract: [
    `Axodendron` validates, analyzes, transforms, exports, and renders neuronal morphology data in Typst. A deterministic Rust/WebAssembly core owns SWC parsing, topology, morphometrics, transformations, and SVG geometry; Typst owns document layout and native annotations.
  ],
  wrap-snippets: true,
  examples-scope: (
    scope: (
      swc: swc,
      cetz: cetz,
      aa-cell: aa-cell,
      nr5a1-cell: nr5a1-cell,
      sst-cell: sst-cell,
      vipr2-cell: vipr2-cell,
      aa-analysis: aa-analysis,
      nr5a1-analysis: nr5a1-analysis,
      sst-analysis: sst-analysis,
      vipr2-analysis: vipr2-analysis,
      aa-source: aa-source,
      nr5a1-source: nr5a1-source,
      sst-source: sst-source,
      vipr2-source: vipr2-source,
      attributed: attributed,
      field-maximum: field-maximum,
      aa-branch-maximum: aa-branch-maximum,
      aa-path-maximum: aa-path-maximum,
      aa-local-angle: aa-local-angle,
      aa-angle-nodes: aa-angle-nodes,
      aa-angle-min: aa-angle-min,
      aa-angle-max: aa-angle-max,
      sst-tmd: sst-tmd,
      sst-tmd-scale: sst-tmd-scale,
      example-population: example-population,
      example-features: example-features,
    ),
  ),
  theme: manual-theme,
)

#let example = example.with(side-by-side: false, breakable: true)
#let doc-code(..args, body) = frame(
  breakable: true,
  codly.local(number-format: none, breakable: true, ..args, body),
)

= Getting Started

Import `axodendron`, read an SWC file as bytes, validate it, and pass the resulting immutable cell to @cmd:render[-]. The filename in a user document is resolved by the calling document, not by the package.

#example[
  ```typ
  #import "@preview/axodendron:0.1.1" as swc

  #let cell = swc.load(
    read("Sst-IRES-Cre_Ai14-188740-03-02-01_491119369_m.kp12.swc", encoding: none),
    profile: "incf-strict",
  )

  #swc.render(cell, width: 120mm, height: 90mm)
  ```
][
  #attributed(
    swc.render(sst-cell, width: 120mm, height: 90mm, display-tolerance: 0.5),
    sst-source,
  )
]

The examples below assume `#import "@preview/axodendron:0.1.1" as swc`. Code snippets treat the directory containing the selected `.swc` file as the caller root. This manual keeps its Typst sources and data separate, so its executable setup reads the same files from `examples/data/`.

On Typst 0.15.0 and later, @cmd:load[-] can receive `path("AA0109.CNG.swc")` directly. Axodendron reads the path inside the package call while retaining caller-relative resolution. This manual uses `read(..., encoding: none)` because the minimum supported compiler is Typst 0.14.0 and the documentation toolchain is pinned to Typst 0.14.2.

SWC coordinates and radii are plain numbers in `cell.units`, conventionally micrometres. Typst lengths such as `120mm`, `4pt`, and `8pt` control document layout only and are never mixed into morphology calculations.

== Choosing a Validation Profile

Use `profile: "incf-strict"` for publication inputs expected to satisfy INCF SWC 1.0 ordering. It requires sequential positive IDs, parent-before-child order, a first root with parent `-1`, and one connected root. Use `profile: "permissive"` when a structurally valid archive contains arbitrary positive IDs, out-of-order rows, or a rooted forest that must be preserved exactly.

#warning-alert[
  Neither profile repairs morphology. Duplicate IDs, missing parents, self-parenting, cycles, malformed rows, and non-finite geometry remain errors. Permissive mode relaxes ordering and connectedness only.
]

== Cell Values

A successful load returns a dictionary containing `valid`, `diagnostics`, `fingerprint`, `source-fingerprint`, `node-count`, `units`, `metadata`, and a private canonical `payload`. The semantic fingerprint ignores harmless text formatting but changes when morphology semantics change. The source fingerprint tracks the exact input bytes.

#example[
  ```typ
  #let cell = swc.load(read("AA0109.CNG.swc", encoding: none))

  #table(
    columns: 2,
    [Valid], [#cell.valid],
    [Nodes], [#cell.node-count],
    [Units], [#cell.units],
    [Semantic fingerprint], [#cell.fingerprint],
  )
  ```
][
  #attributed(
    table(
      columns: (1fr, 2.2fr),
      inset: 5pt,
      [*Valid*], [#aa-cell.valid],
      [*Nodes*], [#aa-cell.node-count],
      [*Units*], [#aa-cell.units],
      [*Semantic fingerprint*], [#text(size: 7pt, aa-cell.fingerprint)],
    ),
    aa-source,
  )
]

= Compatibility and Architecture

#{
  set par(justify: false)
  table(
    columns: (1.45fr, 0.75fr, 2.15fr),
    inset: 6pt,
    align: left,
    table.header([*Surface*], [*Minimum*], [*Coverage*]),
    [Typst package with string or bytes input],
    [`0.14.0`],
    [CI compiles the complete package and smoke suite on the minimum and latest configured Typst versions],
    [Typst `path(...)` input],
    [`0.15.0`],
    [A version-gated smoke assertion loads a real file through `path(...)`],
    [Rust workspace],
    [`1.85.0`],
    [Ubuntu minimum plus stable Rust on Ubuntu, macOS, and Windows],
    [Detailed manual],
    [`0.14.2`],
    [Built reproducibly with `just docs` and the pinned Nixpkgs revision],
  )
}

The package is split into a format-independent Rust core, an SVG renderer, and a thin Typst minimal-protocol adapter. The WASM plugin is pure and stateless: it cannot access files, clocks, randomness, environment variables, or the network. Equal request bytes produce equal response bytes.

The wire boundary uses deterministic CBOR with `protocol_version: 2`. Morphology payload schema 2 serializes canonical arrays, provenance, and fingerprints; derived children, roots, components, lookup maps, and soma classification are rebuilt and validated on every decode.

= Example Data, Attribution, and License

This manual uses four standardized real SWC reconstructions from NeuroMorpho.Org. They are distributed under #link("https://creativecommons.org/licenses/by/4.0/")[CC BY 4.0]. Reuse requires attribution to the original study and NeuroMorpho.Org, RRID:SCR_002145. NeuroMorpho.Org also requests citation of its database publications, including Tecuatl, Ljungquist, and Ascoli (2024), #link("https://doi.org/10.1096/fba.2024-00048")[doi:10.1096/fba.2024-00048].

#{
  set text(size: 8.3pt)
  set par(justify: false)
  table(
    columns: (1.6fr, 0.55fr, 0.8fr, 1.45fr),
    inset: 4pt,
    align: left,
    table.header([*File*], [*Record*], [*Archive*], [*Original study*]),
    [#breakable-mono("AA0109.CNG.swc")], [#link("https://neuromorpho.org/api/neuron/id/85226")[85226]], [MouseLight], [#link("https://doi.org/10.1002/jnr.23978")[10.1002/jnr.23978]; #link("https://doi.org/10.25378/janelia.5526706")[deposit]],
    [#breakable-mono("Nr5a1-Cre_Ai14-187777-05-02-01_491392821_m.kp1.swc")], [#link("https://neuromorpho.org/api/neuron/id/62390")[62390]], [Allen Cell Types], [#link("https://doi.org/10.1016/j.neuron.2015.02.022")[10.1016/j.neuron.2015.02.022]],
    [#breakable-mono("Sst-IRES-Cre_Ai14-188740-03-02-01_491119369_m.kp12.swc")], [#link("https://neuromorpho.org/api/neuron/id/62495")[62495]], [Allen Cell Types], [#link("https://doi.org/10.1016/j.neuron.2015.02.022")[10.1016/j.neuron.2015.02.022]],
    [#breakable-mono("Vipr2-IRES2-Cre_Ai14-310513-05-02-01_637021223_m.CNG.swc")], [#link("https://neuromorpho.org/api/neuron/id/102520")[102520]], [Allen Cell Types], [#link("https://doi.org/10.1016/j.neuron.2015.02.022")[10.1016/j.neuron.2015.02.022]],
  )
}

#example[
  ```typ
  #grid(
    columns: 2,
    gutter: 5mm,
    row-gutter: 5mm,
    swc.render(aa, width: 65mm, height: 48.75mm),
    swc.render(nr5a1, width: 65mm, height: 48.75mm),
    swc.render(sst, width: 65mm, height: 48.75mm),
    swc.render(vipr2, width: 65mm, height: 48.75mm),
  )
  ```
][
  #grid(
    columns: 2,
    gutter: 5mm,
    row-gutter: 5mm,
    attributed(swc.render(aa-cell, width: 65mm, height: 48.75mm, canvas-width: 800, canvas-height: 600, display-tolerance: 1), aa-source),
    attributed(swc.render(nr5a1-cell, width: 65mm, height: 48.75mm, canvas-width: 800, canvas-height: 600, display-tolerance: 1), nr5a1-source),
    attributed(swc.render(sst-cell, width: 65mm, height: 48.75mm, canvas-width: 800, canvas-height: 600, display-tolerance: 1), sst-source),
    attributed(swc.render(vipr2-cell, width: 65mm, height: 48.75mm, canvas-width: 800, canvas-height: 600, display-tolerance: 1), vipr2-source),
  )
]

#info-alert[
  The real files are included only as attributed documentation and README examples. Their source URLs, SHA-256 checksums, archive names, original studies, and license are also recorded in `THIRD_PARTY_NOTICES.md`. The separate 30-case regression corpus remains an ignored, checksum-verified private cache and is not distributed.
]

= Validation and Provenance

@cmd:diagnostics[-] returns every retained diagnostic as a dictionary with `code`, `severity`, `message`, optional `line`, optional `column`, and optional `node-id`. Set #arg[fail-on-error] to `false` when a document should inspect invalid input instead of stopping immediately.

#example[
  ```typ
  #let malformed = swc.from-text(
    "1 1 0 0 0 2 -1\n1 3 0 5 0 1 1\n",
    fail-on-error: false,
  )

  #for item in swc.diagnostics(malformed) [
    #item.severity: #item.code - #item.message
  ]
  ```
][
  #let malformed = swc.from-text(
    "1 1 0 0 0 2 -1\n1 3 0 5 0 1 1\n",
    fail-on-error: false,
  )
  #table(
    columns: (0.65fr, 1.35fr, 2.25fr),
    inset: 5pt,
    align: left + top,
    table.header([*Severity*], [*Code*], [*Message*]),
    ..swc.diagnostics(malformed).map(item => (
      item.severity,
      breakable-mono(item.code, size: 8.5pt),
      item.message,
    )).flatten(),
  )
]

@cmd:metadata[-] exposes recognized header fields without applying them. Scale, shrinkage correction, species, region, and free-form comments remain provenance. Axodendron never changes coordinates or radii because a header suggests a correction.

== Profiles in Practice

`incf-strict` is appropriate for standardized NeuroMorpho.Org files and reproducible interchange. `permissive` is appropriate for structurally valid rooted forests or legacy tools that do not preserve row ordering. Exporting a single-root transformed cell through @cmd:export-swc[-] produces sequential, topologically ordered IDs that can be loaded strictly again.

= Morphometric Analysis

@cmd:analyze[-] computes one versioned bundle containing a summary, topology, sections, tortuosity, and four node-aligned fields. The default `neurites` domain excludes type-1 soma nodes and every soma-incident edge; `raw` includes every encoded node and edge.

#example[
  ```typ
  #let metrics = swc.analyze(cell)
  #let summary = metrics.summary

  #table(
    columns: 2,
    [Nodes], [#summary.node_count],
    [Cable length], [#summary.total_cable_length],
    [Branch points], [#summary.branch_point_count],
    [Terminals], [#summary.terminal_count],
  )
  ```
][
  #let summary = aa-analysis.summary
  #attributed(
    table(
      columns: (1.5fr, 1fr),
      inset: 5pt,
      [*Domain*], [#summary.domain],
      [*Neurite nodes*], [#summary.node_count],
      [*Cable length*], [#calc.round(summary.total_cable_length, digits: 2) #summary.units],
      [*Branch points*], [#summary.branch_point_count],
      [*Terminals*], [#summary.terminal_count],
      [*Sections*], [#summary.section_count],
      [*Maximum root path*], [#calc.round(summary.max_root_path_length, digits: 2) #summary.units],
    ),
    aa-source,
  )
]

== Domains and Topology

Each non-soma node whose parent is excluded becomes an independent arbor root in the `neurites` domain. A node with more than one included child is a branch point; a node with no included child is terminal. `topology.node-ids`, `parent-ids`, `root-ids`, `terminal-ids`, `branch-point-ids`, and `component-ids` are aligned and deterministic.

Use `domain: "raw"` only when a measurement should follow the encoded graph convention, including soma nodes and soma connectors. The distinction matters when comparing cable length or path distance with external tools.

== Sections, Paths, and Tortuosity

The default `section-boundaries: "topology-and-type"` starts or ends a section at roots, branch points, terminals, and SWC type changes. The transition edge belongs to the proximal section, so section lengths sum exactly to total cable length. `"topology-only"` ignores type changes.

Root path length accumulates 3D Euclidean segment lengths from each arbor root. Radial distance is the straight-line 3D distance from that root. Section tortuosity is path length divided by endpoint distance; zero-span sections are excluded and counted separately.

== Branch and Strahler Order

Centrifugal branch order is `1` at each arbor root and increments on children of a branch point. Strahler order is computed independently per arbor: terminals are `1`, and a parent increments the maximum child order only when that maximum occurs at least twice.

#example[
  ```typ
  #let metrics = swc.analyze(cell)

  #swc.render(
    cell,
    color-by: metrics.branch_order,
    colormap: "viridis",
    minimum: 1,
    maximum: 20,
    color-bar: swc.color-bar(
      min: 1,
      max: 20,
      label: [branch order],
    ),
  )
  ```
][
  #attributed(
    swc.render(
      aa-cell,
      color-by: aa-analysis.branch_order,
      minimum: 1,
      maximum: aa-branch-maximum,
      width: 130mm,
      height: 97.5mm,
      display-tolerance: 1,
      color-bar: swc.color-bar(
        min: 1,
        max: aa-branch-maximum,
        label: [branch order],
      ),
    ),
    aa-source,
  )
]

Node fields include `name`, `node-ids`, `values`, `units`, `fingerprint`, `domain`, and `definition-version`. Passing a field from another cell to @cmd:render[-] is rejected instead of silently assigning values to the wrong geometry.

== Radius-Dependent Metrics

Each neurite segment is modeled as an uncapped circular frustum with axial length $L$ and endpoint radii $r_0$, $r_1$:

$ "area" = pi (r_0 + r_1) sqrt(L^2 + (r_0 - r_1)^2) $

$ "volume" = pi L (r_0^2 + r_0 r_1 + r_1^2) / 3 $

No end caps are added. A non-positive endpoint radius makes aggregate surface area and volume unavailable, and the offending node IDs are returned. Input radii are never repaired silently.

A single-point soma exposes sphere area and volume. A valid NeuroMorpho three-point soma exposes equivalent-sphere values and the encoded cylinder's lateral area and volume. Equal radii, endpoint distance, and endpoint opposition use a 1% scale-relative tolerance. Other soma encodings do not receive invented scalar metrics.

= Versioned Metric System

@cmd:measure[-] complements the fixed @cmd:analyze[-] bundle with individually versioned metrics. A request uses one common selection query, and each returned `MetricResult` records the stable metric ID, an independent definition version, all resolved parameters, morphology/topology/selection fingerprints, implementation provenance, a tagged data container, and explicit missing values.

The data tag is one of `morphology-metric`, `node-field`, `section-field`, or `bifurcation-field`. A bifurcation key records the branch node and the participating child sections, so pairwise values remain unambiguous at multifurcations. A section key is intentionally transient: it includes the topology fingerprint, selection fingerprint, section-definition revision, boundary policy, and proximal/distal node IDs. It must not be persisted as an identity independent of those fields.

#warning-alert[
  Missing values are not encoded as an unexplained `none`. Every undefined entity carries a reason such as `non-binary-bifurcation`, `zero-length`, `non-positive-radius`, or `degenerate`, plus a detail string. Do not discard this information when aggregating results.
]

== Selection and Explicit Field Conversion

The common selection query combines `domain`, `kinds`, subtree `roots`, and explicit `nodes`. The result is always an induced forest: an original edge is included only when both endpoints survive. Topology-aware selectors return roots, branch points, terminals, soma nodes, or nodes constrained by centrifugal branch order or Strahler order under exactly the same query semantics. Orders are computed after applying the query, so they describe the selected induced forest rather than a hidden full-cell topology.

@cmd:branch-order[-] and @cmd:strahler-order[-] construct selectors with one exact positive order or an inclusive `min`/`max` interval. Their convenience wrappers @cmd:branch-order-nodes[-] and @cmd:strahler-order-nodes[-] return IDs directly. Exact and range constraints are mutually exclusive; zero, negative, and inverted ranges are rejected.

Section and bifurcation values are not node fields. Use @cmd:field-to-nodes[-] with an explicit placement and collision reducer before passing them to `color-by`. The default reducer is `error`; this prevents pairwise bifurcation values from being silently collapsed at a multifurcation.

#example[
  ```typ
  #let angle = swc.measure(
    cell,
    metrics: swc.metric(
      "local-bifurcation-angle",
      multifurcation: "pairwise",
    ),
  ).first()
  #let at-branches = swc.field-to-nodes(
    cell,
    field: angle,
    placement: "bifurcation-branch",
    reducer: "mean",
  )

  #swc.render(cell, color-by: at-branches, colormap: "magma")
  ```
][
  #attributed(
    swc.render(
      aa-cell,
      color-by: aa-angle-nodes,
      colormap: "magma",
      minimum: aa-angle-min,
      maximum: aa-angle-max,
      width: 130mm,
      height: 97.5mm,
      display-tolerance: 1,
      color-bar: swc.color-bar(
        min: aa-angle-min,
        max: aa-angle-max,
        label: [local bifurcation angle (degrees)],
        position: top + right,
      ),
    ),
    aa-source,
  )
]

== Branch and Section Definitions

#{
  set par(justify: false)
  set text(size: 8.2pt)
  table(
    columns: (1.45fr, 2.75fr),
    inset: 4.5pt,
    align: left,
    table.header([*Metric ID*], [*Definition*]),
    [#breakable-mono("local-bifurcation-angle")], [Angle between the first non-zero outgoing child-segment vectors.],
    [#breakable-mono("remote-bifurcation-angle")], [Angle between vectors from the branch node to the distal endpoints of the outgoing sections.],
    [#breakable-mono("sibling-ratio")], [$min(d_1,d_2) / max(d_1,d_2)$ for positive child diameters; the sampling rule is recorded.],
    [#breakable-mono("partition-asymmetry-terminal")], [$abs(N_1-N_2)/(N_1+N_2-2)$ using selected-subtree terminal counts; two terminal children are defined as zero.],
    [#breakable-mono("diameter-power-ratio")], [$sum_i d_i^p / d_0^p$ for every outgoing child; `p` and diameter sampling are recorded.],
    [#breakable-mono("rall-ratio")], [Alias of the diameter-power ratio with default $p=3/2$. A value of one satisfies that parameterized relation.],
    [#breakable-mono("taper-rate")], [Signed slope of radius or diameter against cumulative section path length; ordinary least squares is the default and endpoint difference is optional.],
    [#breakable-mono("segment-meander-angle")], [Turning angle between consecutive non-zero outward segment vectors: zero is locally straight and 180 degrees is reversal.],
  )
}

Angles are serialized in degrees on $[0,180]$. Pairwise processing is the default for multifurcations and is recorded in the key; choose `multifurcation: "exclude"` when a binary-only definition is required. Sibling ratio follows the smaller-over-larger convention on $[0,1]$. Diameter power ratios include all outgoing children and require positive parent and child diameters.

The taper result is signed: negative means narrowing from root toward tip. `taper-quantity: "diameter"` is the default, while `"radius"` is a distinct resolved definition. The fitted abscissa is cumulative three-dimensional path distance at the original SWC samples; no resampling or smoothing occurs.

NeuroM's `section_meander_angles` uses the supplementary opening-angle convention, where a straight continuation is $pi$ radians. Axodendron's turning angle is therefore 180 degrees minus the NeuroM opening angle after conversion to degrees. This convention transform and NeuroM's radians-to-degrees transform are recorded explicitly in the cross-validation fixture rather than described as exact definition equality.

== Basic Versioned Metrics

#{
  set par(justify: false)
  set text(size: 8.2pt)
  table(
    columns: (1.45fr, 0.75fr, 2fr),
    inset: 4.5pt,
    align: left,
    table.header([*Metric IDs*], [*Support*], [*Definition*]),
    [#breakable-mono("node-count, branch-point-count, terminal-count, section-count")], [Morphology], [Counts in the selected induced forest.],
    [#breakable-mono("total-cable-length, maximum-root-path-length")], [Morphology], [Stable three-dimensional edge sum and maximum selected-root path distance.],
    [#breakable-mono("neurite-surface-area, neurite-volume")], [Morphology], [Uncapped-frustum aggregates; any non-positive endpoint radius yields explicit missing records rather than a partial total.],
    [#breakable-mono("root-path-length, radial-distance")], [Node], [Distance from each selected induced-arbor root.],
    [#breakable-mono("branch-order, strahler-order")], [Node], [Discrete topology fields on the selected forest.],
    [#breakable-mono("section-length, section-contraction")], [Section], [Section path length and endpoint distance divided by path length.],
  )
}

These metrics use the same tagged `MetricResult`, selection fingerprints, definition versions, missing reasons, and population aggregation rules as definition-sensitive branch metrics. The fixed @cmd:analyze[-] bundle remains convenient for a broad report, while @cmd:measure[-] is the comparable and provenance-bearing interface.

== Spatial Metrics and Principal Frames

#{
  set par(justify: false)
  set text(size: 8.2pt)
  table(
    columns: (1.4fr, 2.8fr),
    inset: 4.5pt,
    align: left,
    table.header([*Metric ID*], [*Support and interpretation*]),
    [#breakable-mono("centroid")], [Node mean or exact continuous cable-length centroid.],
    [#breakable-mono("bounding-box")], [Axis-aligned minimum and maximum of selected node centers; radii are excluded.],
    [#breakable-mono("principal-extents")], [Full min-to-max spans of selected node centers in a deterministic principal frame.],
    [#breakable-mono("fractional-anisotropy")], [Fractional anisotropy of the three spatial covariance eigenvalues.],
    [#breakable-mono("convex-hull-2d-area")], [Projected node-center hull area in a coordinate or principal plane.],
    [#breakable-mono("convex-hull-3d-surface-area")], [Surface area of the three-dimensional node-center hull.],
    [#breakable-mono("convex-hull-3d-volume")], [Volume of the three-dimensional node-center hull.],
    [#breakable-mono("volume-density")], [Selected uncapped-frustum volume divided by the three-dimensional node-center hull volume.],
  )
}

Cable-length weighting analytically integrates the first and second moments of every selected line segment, rather than treating sample points as independent observations. Consequently, inserting collinear samples leaves the resulting centroid and covariance unchanged up to floating-point rounding. Node weighting is available when the sampling distribution itself is intentionally part of the statistic.

@cmd:principal-frame[-] is a first-class result rather than a loose metric. It returns origin, centroid, ordered axes, covariance eigenvalues, extents, numerical rank, ambiguous-axis flags, and complete tolerance/sign/handedness provenance. Eigenvalues are ordered from largest to smallest. Axis signs use a deterministic farthest-projection rule with node-ID tie breaking, and the final basis is right-handed.

Repeated or nearly repeated eigenvalues make individual axes scientifically non-identifiable. Axodendron reports that degeneracy using $epsilon_a + epsilon_r max(lambda_i,lambda_j,1)$; it does not imply a biologically meaningful orientation merely because a reproducible numerical basis can be chosen. @cmd:pca-align[-] rejects such a frame unless `allow-degenerate: true` is explicit.

Named render projections `"principal-xy"`, `"principal-xz"`, and `"principal-yz"` compute the default cable-weighted frame and project without modifying the morphology. Use `swc.pca-align` only when a transformed SWC-compatible coordinate set is actually required.

= Sholl Analysis

@cmd:sholl[-] intersects original 3D segments with spheres. @cmd:sholl-2d[-] first applies the chosen physical orthographic projection, then intersects the projected segments with circles. Display simplification is never used for either calculation.

#example[
  ```typ
  #let result = swc.sholl(
    cell,
    radii: range(20, 220, step: 20),
  )

  #table(
    columns: 2,
    table.header([Radius], [Intersections]),
    ..result.bins.map(bin => (bin.radius, bin.intersections)).flatten(),
  )
  ```
][
  #let result = swc.sholl(sst-cell, radii: range(20, 220, step: 20))
  #attributed(
    table(
      columns: (1fr, 1fr),
      inset: 4pt,
      table.header([*Radius (#result.units)*], [*Intersections*]),
      ..result.bins.map(bin => ([#bin.radius], [#bin.intersections])).flatten(),
    ),
    sst-source,
  )
]

The default center is the represented soma center, or the sole root when no soma exists. Soma-free forests and disconnected soma subgraphs have no unique default; provide #arg[center] or #arg[center-node]. Intersections at segment parameter `t` in `(0, 1]` count, so a shared vertex is counted once and a tangency counts once.

= Pure Transformations

Every transformation returns a new cell. The input remains unchanged. Results include `transform-report`, `mapping`, and `lineage`; the report records source/result fingerprints, node counts, removed and inserted IDs, cable-length change, and any guaranteed deviation bound.

== SWC-Compatible Geometry

@cmd:translate[-], @cmd:rotate[-], @cmd:uniform-scale[-], @cmd:reflect[-], @cmd:center-morphology[-], and @cmd:pca-align[-] preserve circular SWC radius semantics. Translation, rotation, reflection, centering, and principal alignment keep radii unchanged. Uniform scaling multiplies coordinates and radii by the same positive factor. Topology, node IDs, kinds, and parent links are preserved.

#example[
  ```typ
  #let centered = swc.center-morphology(cell, by: "centroid")
  #let frame = swc.principal-frame(centered, origin: "centroid")
  #let aligned = swc.pca-align(centered, frame: frame)
  #let rotated = swc.rotate(
    aligned,
    axis: (0, 0, 1),
    angle: 15deg,
  )
  ```
][
  #let centered = swc.center-morphology(aa-cell, by: "centroid")
  #let frame = swc.principal-frame(centered, origin: "centroid")
  #let aligned = swc.pca-align(centered, frame: frame)
  #let rotated = swc.rotate(aligned, axis: (0, 0, 1), angle: 15deg)
  #attributed(
    swc.render(rotated, width: 130mm, height: 97.5mm, display-tolerance: 1),
    aa-source,
  )
]

== General Affine Geometry

@cmd:affine-transform[-] is deliberately separate. A non-similarity affine map turns a circular cross-section into an ellipse, which SWC cannot encode. The caller must choose `radius-policy: "preserve"` or `"volume-equivalent"`; the transform report records the matrix, determinant, applied radius scale, policy, and `radius-representation-lossy: true`. Singular and non-finite transforms are rejected.

#warning-alert[
  General affine output is a centerline transform with an explicitly approximate radius representation. Do not describe it as an exact geometric transform of neuronal volume or membrane surface.
]

== Selection and Extraction

@cmd:select-nodes[-] and @cmd:select-kinds[-] return the induced forest over retained original edges. They never invent an edge across a removed node. @cmd:subtree[-] extracts a node and every descendant. @cmd:path[-] extracts the unique undirected route between two nodes in one component.

#example[
  ```typ
  #let selected = swc.select-kinds(cell, kinds: (3, 4))
  #let branch = swc.subtree(cell, node: 12)
  #let route = swc.path(cell, start: 4, end: 11)
  ```
][
  #let selected = swc.select-kinds(aa-cell, kinds: (3, 4))
  #let branch = swc.subtree(aa-cell, node: 12)
  #let route = swc.path(aa-cell, start: 4, end: 11)
  #attributed(
    table(
      columns: (1.4fr, 1fr),
      inset: 5pt,
      [*Selected dendrites*], [#selected.node-count nodes],
      [*Subtree at node 12*], [#branch.node-count nodes],
      [*Path 4 to 11*], [#route.node-count nodes],
    ),
    aa-source,
  )
]

== Rerooting and Pruning

@cmd:reroot[-] reverses the parent chain so the requested node becomes the root of its component. @cmd:prune[-] removes every node of the listed kinds and each removed node's complete descendant subtree.

#example[
  ```typ
  #let metrics = swc.analyze(cell)
  #let dendrites = swc.prune(cell, kinds: (2,))
  #let dendrite-metrics = swc.analyze(dendrites)

  #grid(
    columns: 2,
    gutter: 5mm,
    swc.render(cell, color-by: metrics.branch_order),
    swc.render(dendrites, color-by: dendrite-metrics.branch_order),
  )
  ```
][
  #let dendrites = swc.prune(aa-cell, kinds: (2,))
  #let dendrite-metrics = swc.analyze(dendrites)
  #attributed(
    grid(
      columns: 2,
      gutter: 5mm,
      swc.render(aa-cell, color-by: aa-analysis.branch_order, width: 68mm, height: 51mm, display-tolerance: 1),
      swc.render(dendrites, color-by: dendrite-metrics.branch_order, width: 68mm, height: 51mm, display-tolerance: 1),
    ),
    aa-source,
  )
]

== Simplification

@cmd:simplify[-] applies topology-preserving 3D Ramer-Douglas-Peucker simplification independently between mandatory section points. Roots, branch points, terminals, protected IDs, soma nodes, and type changes are retained according to the options. The report's `guaranteed-max-deviation` is the requested tolerance.

Simplification is also available as the display-only #arg[display-tolerance] option of @cmd:render[-]. Display simplification never changes the source cell, analysis results, or export; requested native annotations, CeTZ labels, and explicit node anchors are protected automatically.

#pagebreak(weak: true)

== Resampling

@cmd:resample[-] inserts equal-arc-length samples within non-soma, same-type topological sections. Roots, soma nodes, branch points, terminals, type boundaries, and protected IDs retain their original IDs. Radius interpolation is linear in arc length, and every inserted node receives proximal/distal lineage with a distal fraction.

#example[
  ```typ
  #let sampled = swc.resample(cell, step: 5)

  #table(
    columns: 2,
    [Source nodes], [#cell.node-count],
    [Result nodes], [#sampled.node-count],
    [Inserted nodes], [#sampled.lineage.len()],
  )
  ```
][
  #let sampled = swc.resample(aa-cell, step: 5)
  #attributed(
    table(
      columns: (1.5fr, 1fr),
      inset: 5pt,
      [*Source nodes*], [#aa-cell.node-count],
      [*Result nodes*], [#sampled.node-count],
      [*Inserted nodes*], [#sampled.lineage.len()],
      [*Cable-length change*], [#calc.round(sampled.transform-report.cable_length_change, digits: 8)],
    ),
    aa-source,
  )
]

== Deterministic Export

@cmd:export-swc[-] emits canonical SWC with a deterministic header, topological row order, and sequential IDs. Single-root results satisfy `incf-strict`; forests retain every root and must be reloaded with `permissive` unless they are connected separately by the caller.

= Topology Visualization

@cmd:render-tree[-] draws the selected rooted forest independently of physical projection. Horizontal placement follows stable input/child order and leaf allocation. Vertical depth is one of topological edge depth, root path length, or Euclidean radial distance from each selected arbor root.

Strahler order and centrifugal branch order are discrete attributes, not interchangeable continuous distances. They belong in `color-by`; they are deliberately not accepted as `depth` modes.

#example[
  ```typ
  #let metrics = swc.analyze(cell)

  #swc.render-tree(
    cell,
    depth: "path-length",
    color-by: metrics.strahler_order,
  )
  ```
][
  #attributed(
    swc.render-tree(
      sst-cell,
      depth: "path-length",
      color-by: sst-analysis.strahler_order,
      width: 130mm,
      height: 97.5mm,
      canvas-width: 800,
      canvas-height: 600,
    ),
    sst-source,
  )
]

The tree report names its depth rule explicitly. The renderer uses the same query layer and node-field fingerprint checks as physical rendering, and its returned node anchors can be used with the same CeTZ leader-label API.

= Topological Morphology Descriptor

@cmd:tmd[-] computes terminal-to-merge persistence pairs with the elder rule of the Topological Morphology Descriptor. `filtration: "root-path-length"` is monotone away from each selected arbor root, which has path distance zero; `center` is therefore inapplicable and rejected. `"radial-distance"` uses Euclidean distance from `center: "soma"` (the default) or `center: "root"` and can be non-monotone along a traced branch.

For each merge, the component with the larger terminal filtration survives; an exact tie is resolved by the lower terminal node ID. One essential survivor is retained per selected arbor. Every pair stores birth, death, absolute persistence, sorted display endpoints, terminal, merge, arbor root, essential status, and a `non_monotone` flag.

The returned pair array, and therefore barcode rows, is deterministic: arbor-root ID ascending, essential survivor first, birth descending, death descending, terminal-node ID ascending, then merge-node ID ascending. The exact rule is also stored in `provenance.pair_order`; the original TMD paper otherwise treats the barcode's vertical order as arbitrary.

#warning-alert[
  A radial terminal may lie closer to the origin than an upstream merge. Axodendron reports such pairs through `non-monotone-pair-count` and preserves signed birth/death ordering instead of clamping, discarding, or silently redefining the filtration. Treat these as disclosed terminal–branch TMD pairs, not as evidence that radial distance was monotone.
]

#example[
  ```typ
  #let descriptor = swc.tmd(
    cell,
    filtration: "root-path-length",
  )
  #let scale = swc.persistence-scale(descriptor)

  #grid(
    columns: 2,
    swc.persistence-barcode(descriptor, scale: scale),
    swc.persistence-diagram(descriptor, scale: scale),
  )
  #swc.persistence-legend()
  ```
][
  #block(breakable: false)[
    #attributed(
      stack(
        dir: ttb,
        spacing: 2mm,
        grid(
          columns: (1fr, auto),
          gutter: 6mm,
          swc.persistence-barcode(sst-tmd, width: 80mm, height: 43mm, scale: sst-tmd-scale),
          swc.persistence-diagram(sst-tmd, size: 43mm, scale: sst-tmd-scale),
        ),
        align(center, swc.persistence-legend()),
      ),
      sst-source,
    )
  ]
]

Birth is horizontal and death vertical, matching the original TMD publication. Both plots use the same explicit physical scale, label it in the morphology's units, and apply blue to ordinary pairs and red only to the essential survivor retained for each selected arbor. Native renderers inset endpoint strokes and points from their clipped bounds. Essential bars remain finite root-to-terminal intervals rather than infinity. These conventions are recorded in provenance and must be matched across TMD implementations.

= Population Features

@cmd:population[-] retains named morphologies, while @cmd:feature-table[-] evaluates a common query and feature specification for every row. The primary result is Typst-native arrays and dictionaries. @cmd:feature-table-csv[-] is a separate text export; missing CSV cells are empty, so the Typst result remains the authoritative source for missing reasons.

Morphology scalars must not specify an aggregate. Node, section, and bifurcation fields must specify one. Vector and box morphology metrics require one explicit `component`: `x`, `y`, or `z` for a centroid; `major`, `middle`, or `minor` for principal extents; and `min-x` through `span-z` for a bounding box. This avoids silently flattening structured scientific results. Field aggregation is strict by default: if any selected entity is undefined, the cell is missing with reason `partial-field`. `missing-policy: "omit"` is an explicit instruction to aggregate only defined entities. Columns are rejected if their resolved metric definition, parameters, units, support, aggregate, or component differ across rows.

#example[
  ```typ
  #let features = swc.feature-table(
    cells,
    columns: (
      swc.feature-column("fractional-anisotropy", name: "FA"),
      swc.feature-column("local-bifurcation-angle",
        aggregate: "mean", missing-policy: "omit"),
      swc.feature-column("centroid", component: "x"),
    ),
  )
  ```
][
  #let show-feature(cell) = if cell.status == "value" {
    str(calc.round(cell.value, digits: 3))
  } else {
    "missing: " + cell.reason
  }
  #attributed(
    table(
      columns: (1.15fr, 0.65fr, 1.35fr, 0.85fr),
      inset: 5pt,
      table.header([*Morphology*], [*FA*], [*Mean local angle*], [*Centroid x*]),
      ..example-features.rows.map(row => (
        [#row.id],
        [#show-feature(row.values.at(0))],
        [#show-feature(row.values.at(1))],
        [#show-feature(row.values.at(2))],
      )).flatten(),
    ),
    [#aa-source #nr5a1-source #sst-source],
  )
]

Summaries report valid and missing row counts, mean, median, minimum, maximum, and both sample and population variance. No inferential statistic, imputation, unit conversion, or correction for repeated measurements is performed automatically.

= Rendering

@cmd:render[-] produces a compact deterministic SVG in WASM and places it in a Typst block. Labels, markers, legends, color bars, and scale bars are then composed as native Typst content.

== Projections

Named projections are `"xy"`, `"xz"`, and `"yz"`. An arbitrary orthographic camera is a dictionary containing `direction` and `up` vectors. The vectors must be finite, non-zero, and non-collinear.

#example[
  ```typ
  #grid(
    columns: 3,
    gutter: 4mm,
    swc.render(cell, projection: "xy"),
    swc.render(cell, projection: "xz"),
    swc.render(
      cell,
      projection: (
        direction: (x: 1, y: 1, z: 1),
        up: (x: 0, y: 0, z: 1),
      ),
    ),
  )
  ```
][
  #attributed(
    grid(
      columns: 3,
      gutter: 3mm,
      swc.render(vipr2-cell, projection: "xy", width: 41mm, height: 36mm, canvas-width: 820, canvas-height: 720, display-tolerance: 1),
      swc.render(vipr2-cell, projection: "xz", width: 41mm, height: 36mm, canvas-width: 820, canvas-height: 720, display-tolerance: 1),
      swc.render(vipr2-cell, projection: (direction: (x: 1, y: 1, z: 1), up: (x: 0, y: 0, z: 1)), width: 41mm, height: 36mm, canvas-width: 820, canvas-height: 720, display-tolerance: 1),
    ),
    vipr2-source,
  )
]

#pagebreak(weak: true)

== Geometry and Radius Modes

`geometry: "tapered"` draws radius-aware frusta with round distal joins. `"skeleton"` draws constant-width centerlines. `radius-mode: "physical"` maps projected radii linearly. `"readable"` applies #arg[radius-exponent] and the screen-space #arg[minimum-radius] and #arg[maximum-radius] bounds.

#example[
  ```typ
  #grid(
    columns: 2,
    gutter: 5mm,
    swc.render(cell, geometry: "tapered", radius-mode: "readable"),
    swc.render(cell, geometry: "skeleton"),
  )
  ```
][
  #attributed(
    grid(
      columns: 2,
      gutter: 5mm,
      swc.render(nr5a1-cell, geometry: "tapered", radius-mode: "readable", width: 68mm, height: 51mm, display-tolerance: 1),
      swc.render(nr5a1-cell, geometry: "skeleton", width: 68mm, height: 51mm, display-tolerance: 1),
    ),
    nr5a1-source,
  )
]

== Soma Modes

`soma-mode: "equivalent-sphere"` is the default and displays one body at the projected soma centroid with the representative encoded radius. `"encoded"` uses cylinder geometry for a geometrically valid three-point soma. `"raw-points"` displays every encoded type-1 node and edge. Display policy does not invent soma area or volume metrics.

== Type and Scalar Color

The default `color-by: "type"` mapping uses red `#d62728` for soma, blue `#0072b2` for axon, and green `#009e73` for basal and apical dendrites. Types 5, 6, and 7 use `#e69f00`, `#56b4e9`, and `#cc79a7`; all other kinds use `#4d4d4d`.

Passing any other color string produces a uniform rendering. Passing a node field produces scalar color. Viridis is the default continuous map and Magma is also supported. Finite values are normalized linearly between #arg[minimum] and #arg[maximum], inferred from the field when omitted; values outside the range are clamped. Missing or non-finite values use neutral gray `#9ca3af`.

#example[
  ```typ
  #let metrics = swc.analyze(cell)

  #grid(
    columns: 2,
    gutter: 5mm,
    swc.render(cell, color-by: metrics.root_path_length),
    swc.render(
      cell,
      color-by: metrics.root_path_length,
      colormap: "magma",
    ),
  )
  ```
][
  #attributed(
    grid(
      columns: 2,
      gutter: 5mm,
      swc.render(aa-cell, color-by: aa-analysis.root_path_length, minimum: 0, maximum: aa-path-maximum, width: 68mm, height: 51mm, display-tolerance: 1),
      swc.render(aa-cell, color-by: aa-analysis.root_path_length, colormap: "magma", minimum: 0, maximum: aa-path-maximum, width: 68mm, height: 51mm, display-tolerance: 1),
    ),
    aa-source,
  )
]

== Fitting and Aspect Ratio

Canvas fitting includes painted radii, outlines, and soma extent. It must not clip painted geometry inside #arg[padding]. The Typst ratio `width / height` must exactly match `canvas-width / canvas-height`; Axodendron rejects a mismatch so overlays and physical scale bars remain calibrated.

`background: none` keeps the SVG transparent. Set #arg[outline-color] to add a halo around segments and soma; #arg[outline-width] has no effect when the color is `none`. Style strings accept a bounded safe subset of CSS color syntax and reject external URLs.

= Native Typst Annotations

@cmd:label[-] and @cmd:marker[-] anchor native Typst content to projected node IDs; #arg[offset] then shifts it by `x` and `y` lengths without changing the selected node. @cmd:legend[-], @cmd:color-bar[-], and @cmd:scale-bar[-] place publication-oriented overlays inside the render block. These overlays remain Typst content rather than being flattened into SVG text; #arg[label-gap] controls the vertical separation between a color-bar label and its palette strip.

#example[
  ```typ
  #let metrics = swc.analyze(cell)
  #let path-maximum = calc.ceil(calc.max(..metrics.root_path_length.values))

  #swc.render(
    cell,
    color-by: metrics.root_path_length,
    minimum: 0,
    maximum: path-maximum,
    labels: (swc.label(
      node: 243, offset: (x: 14pt, y: 12pt),
      text(size: 7pt)[basal dendrite #linebreak() terminal (node 243)],
    ),),
    markers: (swc.marker(node: 243),),
    color-bar: swc.color-bar(
      min: 0, max: path-maximum,
      label: [root path length (µm)], label-gap: 5pt, position: top + right,
    ),
    scale-bar: swc.scale-bar(value: 100),
  )
  ```
][
  #block(breakable: false)[
    #attributed(
      align(center, swc.render(
          aa-cell,
          color-by: aa-analysis.root_path_length,
          minimum: 0,
          maximum: aa-path-maximum,
          width: 92mm,
          height: 69mm,
          labels: (swc.label(
            node: 243,
            offset: (x: 14pt, y: 12pt),
            text(size: 7pt)[basal dendrite #linebreak() terminal (node 243)],
          ),),
          markers: (swc.marker(node: 243),),
          color-bar: swc.color-bar(
            min: 0,
            max: aa-path-maximum,
            label: [root path length (µm)],
            label-gap: 5pt,
            position: top + right,
          ),
          scale-bar: swc.scale-bar(value: 100),
        ),
      ),
      aa-source,
    )
  ]
]

Unknown annotation IDs are errors by default. Set #arg[strict-node-ids] to `false` only when optional annotations may legitimately target nodes removed by an earlier selection. Requested native labels, markers, CeTZ labels, and #arg[anchor-nodes] are protected from display simplification.

Label offsets and legend or color-bar positions are explicit layout choices rather than automatic collision-avoidance requests. Tune a node label with `offset: (x: ..., y: ...)`, choose a clear overlay corner with #arg[position], and use #arg[label-gap] when the color-bar title needs more or less separation from the palette; this example labels SWC type-3 terminal node 243, marks the same node with a circle, uses a 14 pt rightward and 12 pt downward label shift, places the color bar in the clear top-right corner, and applies a 5 pt label gap.

== CeTZ Leader Labels

CeTZ leader labels keep the text box away from morphology geometry while an arrow identifies the exact projected node. Import CeTZ separately and pass its module through #arg[cetz]; Axodendron has no mandatory CeTZ dependency and performs no second projection. @cmd:render[-] requests and protects every #arg[cetz-labels] target in the same WASM call, converts its final fitted screen coordinate to the CeTZ bottom-left coordinate system, registers it as a named CeTZ anchor, and composes the SVG, leader, and label in one canvas.

#example[
  ```typ
  #import "@preview/axodendron:0.1.1" as swc
  #import "@preview/cetz:0.5.2"

  #swc.render(
    cell,
    width: 120mm,
    height: 90mm,
    cetz: cetz,
    cetz-labels: (swc.cetz-label(
      node: 447,
      offset: (x: 17mm, y: -9mm),
      controls: (
        (x: 12mm, y: -10mm),
        (x: 5mm, y: -5mm),
      ),
      text(size: 8pt)[basal dendrite terminal],
    ),),
  )
  ```
][
  #attributed(
    align(center, swc.render(
      aa-cell,
      width: 110mm,
      height: 82.5mm,
      cetz: cetz,
      cetz-labels: (swc.cetz-label(
        node: 447,
        offset: (x: 17mm, y: -9mm),
        controls: (
          (x: 12mm, y: -10mm),
          (x: 5mm, y: -5mm),
        ),
        text(size: 8pt)[basal dendrite terminal],
      ),),
    )),
    aa-source,
  )
]

The #arg[offset], each #arg[via] point, and each #arg[controls] point are relative to the selected node in Typst screen coordinates: positive `x` moves right and positive `y` moves down. With `anchor: auto`, Axodendron places the label by the edge or corner facing the node. For a label beside the node, the leader itself starts at the `mid-west` or `mid-east` content anchor, so its initial `y` coordinate is the text's typographic vertical center rather than a corner. The default leader is straight; #arg[via] inserts explicit straight segments. One #arg[controls] point selects a quadratic CeTZ Bezier and two select a cubic Bezier. Axodendron never introduces curvature on its own. The default white fill and 2 pt padding keep morphology lines out of the text without a visible border. Set #arg[target-gap] only when the arrow tip should stop short of the node.

A leader path is not an automatic obstacle-avoidance solver. A direct arrow can still cross an unrelated branch in a dense arbor, so use #arg[via] for a deliberate polyline or #arg[controls] for a deliberate curve and verify the final PDF. The two explicit control points in this example produce the displayed cubic Bezier. Node 447 is an encoded type-3 terminal with no child; the leader reaches that exact rendered sample without crossing the neighboring arbor.

For a custom CeTZ composition, request sparse coordinates with `anchor-nodes: (447, ...)` and `return-report: true`, retrieve a record with @cmd:node-anchor[-], or pass the result to @cmd:cetz-annotate[-]. `x` and `y` in each returned record are lengths from the top-left of the base render block; `x-ratio` and `y-ratio` are normalized screen coordinates, and `screen-x` and `screen-y` retain the fitted SVG coordinates.

== Render Reports

Set #arg[return-report] to `true` to receive `body`, dimensions, `node-anchors`, `report`, `pixels-per-unit`, `source-node-count`, and `rendered-node-count`. The report records radius and soma modes, counts of radii floored or capped, the simplified node count, and the protected overlay node count.

#{
  set par(justify: false)
  table(
    columns: (1.15fr, 2.8fr),
    inset: 5pt,
    table.header([*Field*], [*Interpretation*]),
    [#breakable-mono("pixels-per-unit")], [Projected SVG pixels per morphology coordinate unit after fitting and padding],
    [#breakable-mono("source-node-count")], [Canonical nodes in the input cell],
    [#breakable-mono("rendered-node-count")], [Nodes retained for SVG geometry after display-only simplification],
    [#breakable-mono("node-anchors")], [Sparse projected-node records requested by labels, markers, CeTZ labels, or `anchor-nodes`; all rendered nodes when `include-nodes` is true],
    [#breakable-mono("floored-radius-count")], [Rendered radii raised to `minimum-radius`],
    [#breakable-mono("capped-radius-count")], [Rendered radii limited by `maximum-radius` or `maximum-soma-radius`],
    [#breakable-mono("simplified-node-count")], [Source nodes omitted only from display geometry],
    [#breakable-mono("overlay-node-count")], [Unique valid node IDs protected for native labels, markers, CeTZ labels, or explicit anchors],
  )
}

The report is disclosure metadata, not a transformed morphology. Analysis and export still use the input cell. For a reproducible publication figure, retain the projection, page and canvas dimensions, display tolerance, radius and soma modes, scalar range, palette, and the report alongside the input fingerprint.

= Publication Figure Guidance

Use the type palette when compartment identity matters and a sequential map only for an ordered scalar. Do not use branch-order color without a color bar in a figure intended for quantitative interpretation. State the analysis domain and units in the caption.

Use `radius-mode: "physical"` only when literal projected thickness is legible at the final size. The default readable mode is more robust for overview figures, but its exponent and screen-space caps must be reported when thickness is interpreted.

Keep labels sparse and outside dense arbors. Add a light outline only when a morphology crosses a colored or photographic background. Verify the final PDF at publication size; deterministic SVG structure does not guarantee identical rasterization across font and PDF toolchains.

#warning-alert[
  Display simplification, readable radius scaling, equivalent-sphere soma display, and projection are presentation operations. They never change the cell or scientific analysis, but a publication caption should disclose them when they affect interpretation.
]

= Error Model and Resource Limits

Plugin errors are structured before the Typst wrapper turns them into readable panics. Stable error families are:

#{
  set par(justify: false)
  table(
    columns: (1.1fr, 2.7fr),
    inset: 5pt,
    table.header([*Family*], [*Meaning*]),
    [#breakable-mono("PROTOCOL_*")], [Malformed CBOR or incompatible protocol],
    [#breakable-mono("PAYLOAD_*")], [Incompatible, forged, or internally inconsistent morphology payload],
    [#breakable-mono("SWC_*")], [Lexical, structural, or validation failure],
    [#breakable-mono("SHOLL_*")], [Invalid center, radius, domain, or projection],
    [#breakable-mono("MEASURE_*")], [Unknown metric, invalid parameter, selection, or request size],
    [#breakable-mono("PRINCIPAL_FRAME_*")], [Invalid or degenerate spatial frame],
    [#breakable-mono("CENTER_*")], [Invalid or ambiguous center selection or geometry],
    [#breakable-mono("QUERY_*")], [Invalid topology selector or selection query],
    [#breakable-mono("FIELD_TO_NODES_*")], [Invalid explicit field conversion],
    [#breakable-mono("TMD_*")], [Invalid filtration center, selection, or soma geometry],
    [#breakable-mono("POPULATION_*")], [Invalid population size, column count, or comparability],
    [#breakable-mono("TRANSFORM_*")], [Invalid node, component, tolerance, step, or ID space],
    [#breakable-mono("RENDER_*")], [Invalid canvas, style, scalar field, projection, or overlay],
    [#breakable-mono("LIMIT_*")], [Explicit bounded-resource failure],
  )
}

#{
  set par(justify: false)
  table(
    columns: (1.5fr, 1fr),
    inset: 5pt,
    table.header([*Resource*], [*Limit*]),
    [SWC source / encoded request], [64 MiB / 64 MiB],
    [Morphology nodes], [250,000],
    [Decoded payload / encoded response], [128 MiB / 128 MiB],
    [Sholl radii], [10,000],
    [SVG result], [64 MiB],
    [WASM artifact], [1.25 MiB],
    [Source package bundle], [4 MiB],
  )
}

Limits are checked before unbounded output allocation. The normal quality gate also exercises the full 250,000-node parser boundary, bounded resampling expansion, a 100,000-node performance case, two clean byte-identical WASM builds, and package import compilation.

= API Reference

#custom-type("cell", color: aqua)
#custom-type("analysis", color: blue)
#custom-type("node-field", color: green)
#custom-type("annotation", color: orange)
#custom-type("render-result", color: purple)

The public string `version` reports the embedded plugin version. The public `swc` dictionary mirrors every function below for selective dictionary imports, but dictionary-stored functions require parenthesized calls such as `(swc.analyze)(cell)`. Importing the package as a module and calling `swc.analyze(cell)` remains the recommended interface.

== Loading and Inspection

#command(
  "load",
  arg("source"),
  arg(profile: "permissive"),
  arg(fail-on-error: true),
  ret: "cell",
)[
  Parse and validate SWC input.

  #argument("source", types: (str, bytes, "path"))[
    SWC text or bytes. Typst 0.15.0 and later may pass `path(...)`; Typst 0.14.x should pass `read(..., encoding: none)` output.
  ]

  #argument("profile", types: str, default: "permissive")[
    `"permissive"` or `"incf-strict"`.
  ]

  #argument("fail-on-error", types: bool, default: true)[
    Panic on validation errors when `true`; return an invalid diagnostic-bearing cell when `false`.
  ]
]

#command("from-text", arg("source"), arg(profile: "permissive"), arg(fail-on-error: true), ret: "cell")[
  Parse SWC source already held in a string or bytes value. Arguments and validation behavior match @cmd:load[-] except that `path` input is not read.
]

#command("diagnostics", arg("cell"), ret: array)[
  Return the retained diagnostic dictionaries.
]

#command("metadata", arg("cell"), ret: dictionary)[
  Return retained comments and recognized header fields without applying corrections.
]

== Analysis

#command(
  "analyze",
  arg("cell"),
  arg(domain: "neurites"),
  arg(section-boundaries: "topology-and-type"),
  ret: "analysis",
)[
  Compute the complete versioned morphometrics bundle.

  #argument("domain", types: str, default: "neurites")[
    `"neurites"` excludes soma nodes and soma-incident edges; `"raw"` includes the encoded graph.
  ]

  #argument("section-boundaries", types: str, default: "topology-and-type")[
    `"topology-and-type"` or `"topology-only"`.
  ]
]

#command(
  "sholl",
  arg("cell"),
  arg(radii: none),
  arg(center: none),
  arg(center-node: none),
  arg(domain: "neurites"),
  arg(projection: none),
  ret: dictionary,
)[
  Compute exact 3D sphere intersections. Supplying #arg[projection] switches to physical 2D projected intersections and is normally exposed through @cmd:sholl-2d[-].
]

#command(
  "sholl-2d",
  arg("cell"),
  arg(radii: none),
  arg(projection: "xy"),
  arg(center: none),
  arg(center-node: none),
  arg(domain: "neurites"),
  ret: dictionary,
)[
  Compute exact 2D circle intersections after the requested orthographic projection.
]

#command("available-metrics", ret: array)[
  Return the metric registry with stable IDs, independent definition versions, support entities, units, summaries, reference URLs, and machine-readable parameter schemas. Each parameter record includes its name, value type, default, allowed choices or numerical bound, conditional applicability, and summary.
]

#command(
  "metric",
  arg("id"),
  arg(p: none),
  arg(diameter-sampling: none),
  arg(taper-quantity: none),
  arg(taper-method: none),
  arg(multifurcation: none),
  arg(weighting: none),
  arg(plane: none),
  ret: dictionary,
)[
  Construct a parameterized metric specification. Supplied parameters that are unknown or irrelevant to the selected metric are rejected; accepted defaults and explicit values are serialized into the resolved metric identity.
]

#command(
  "measure",
  arg("cell"),
  arg(metrics: none),
  arg(domain: "neurites"),
  arg(kinds: ()),
  arg(roots: ()),
  arg(nodes: ()),
  arg(section-boundaries: "topology-and-type"),
  ret: array,
)[
  Compute one or more versioned metrics over a common induced-forest selection and return tagged `MetricResult` dictionaries.
]

#command(
  "principal-frame",
  arg("cell"),
  arg(origin: "centroid"),
  arg(weighting: "cable-length"),
  arg(domain: "neurites"),
  arg(kinds: ()),
  arg(roots: ()),
  arg(nodes: ()),
  arg(relative-tolerance: 1e-10),
  arg(absolute-tolerance: 1e-12),
  ret: dictionary,
)[
  Compute a deterministic, provenance-bearing principal coordinate frame and report rank or repeated-eigenvalue ambiguity.
]

#command("select-node-ids", arg("cell"), arg(selector: "all"), ret: array)[
  Return IDs matching `all`, `roots`, `branch-points`, `terminals`, `soma`, or an order-selector dictionary under the common query arguments. `branch-points`, `terminals`, and `soma-nodes` are convenience wrappers.
]

#command("branch-order", arg(exact: none), arg(min: none), arg(max: none), ret: dictionary)[
  Construct an exact or inclusive-range centrifugal branch-order selector. @cmd:branch-order-nodes[-] applies it directly to one morphology.
]

#command("strahler-order", arg(exact: none), arg(min: none), arg(max: none), ret: dictionary)[
  Construct an exact or inclusive-range Strahler-order selector. @cmd:strahler-order-nodes[-] applies it directly to one morphology.
]

#command("branch-order-nodes", arg("cell"), arg(exact: none), arg(min: none), arg(max: none), ret: array)[
  Return IDs matching a centrifugal branch-order constraint under the common query arguments.
]

#command("strahler-order-nodes", arg("cell"), arg(exact: none), arg(min: none), arg(max: none), ret: array)[
  Return IDs matching a Strahler-order constraint under the common query arguments.
]

#command("field-to-nodes", arg("cell"), arg(field: none), arg(placement: none), arg(reducer: "error"), ret: dictionary)[
  Explicitly project a section or bifurcation field to nodes. Valid placements are section proximal/distal/broadcast and bifurcation branch/children; reducers are `error`, `mean`, `minimum`, `maximum`, or `sum`.
]

#command("tmd", arg("cell"), arg(filtration: "radial-distance"), arg(center: auto), ret: dictionary)[
  Compute versioned terminal–merge TMD pairs under radial-distance or root-path-length filtration. Radial distance defaults to the soma and accepts `center: "soma"` or `center: "root"`; root-path length is measured from each selected arbor root and rejects a center. The common domain/kind/root/node selection arguments are also accepted.
]

#command("persistence-scale", arg("descriptor"), arg(min: none), arg(max: none), ret: dictionary)[
  Resolve a physical scale that can be passed to both persistence plots. Explicit bounds must contain every birth and death value.
]

#command("persistence-barcode", arg("descriptor"), arg(width: 80mm), arg(height: 70mm), arg(row-height: none), arg(scale: none), ret: content)[
  Render every TMD pair in provenance-defined row order with a labeled filtration axis. The default distributes rows within #arg[height]; an explicit #arg[row-height] requests fixed spacing and determines total height.
]

#command("persistence-diagram", arg("descriptor"), arg(size: 70mm), arg(scale: none), arg(radius: 1.4pt), ret: content)[
  Render TMD birth/death points with Birth on the horizontal axis, Death on the vertical axis, and physical units on both axes.
]

#command("persistence-legend", ret: content)[
  Render the shared plot key: blue for ordinary pairs and red for the essential survivor retained once per selected arbor.
]

== Population Tables

#command("population-entry", arg("id"), arg(cell: none), ret: dictionary)[
  Construct one named population row.
]

#command("population", arg("entries"), ret: array)[
  Validate a Typst array of population entries without performing analysis.
]

#command("feature-column", arg("metric"), arg(name: none), arg(aggregate: none), arg(component: none), arg(missing-policy: "strict"), ret: dictionary)[
  Construct a comparable scalar feature column. Structured morphology metrics require an explicit component; field aggregates are `mean`, `median`, `minimum`, `maximum`, `sum`, `sample-variance`, or `population-variance`; missing policy is `strict` or explicit `omit`.
]

#command("feature-table", arg("population"), arg(columns: none), ret: dictionary)[
  Evaluate common metric/query definitions, enforce parameter and unit compatibility, retain missing reasons, and compute descriptive summaries.
]

#command("feature-table-csv", arg("table"), ret: str)[
  Serialize the scalar table as RFC 4180-style CSV text without performing file I/O.
]

== Selection and Transformations

#command("translate", arg("cell"), arg(offset: none), ret: "cell")[
  Translate coordinates by a three-dimensional morphology-unit offset.
]

#command("rotate", arg("cell"), arg(axis: none), arg(angle: none), arg(center: (0, 0, 0)), ret: "cell")[
  Apply a right-handed axis-angle rotation; numeric angles are radians and Typst angle values are accepted directly.
]

#command("uniform-scale", arg("cell"), arg(factor: none), arg(center: (0, 0, 0)), ret: "cell")[
  Apply one positive factor to coordinates and radii about a fixed center.
]

#command("reflect", arg("cell"), arg(normal: none), arg(point: (0, 0, 0)), ret: "cell")[
  Reflect coordinates across a plane while preserving circular radii.
]

#command("center-morphology", arg("cell"), arg(by: "soma"), arg(target: (0, 0, 0)), ret: "cell")[
  Translate a soma center or selected centroid to a target coordinate.
]

#command("pca-align", arg("cell"), arg(frame: none), arg(allow-degenerate: false), ret: "cell")[
  Express coordinates in a matching principal frame; ambiguous frames require an explicit opt-in.
]

#command("affine-transform", arg("cell"), arg(matrix: none), arg(translation: (0, 0, 0)), arg(radius-policy: "preserve"), ret: "cell")[
  Apply an invertible general affine centerline transform with explicit lossy SWC radius policy.
]

#command("select-nodes", arg("cell"), arg(nodes: none), ret: "cell")[
  Select exactly the listed node IDs as an induced forest.
]

#command("select-kinds", arg("cell"), arg(kinds: none), ret: "cell")[
  Select nodes with the listed SWC kinds as an induced forest.
]

#command("subtree", arg("cell"), arg(node: none), ret: "cell")[
  Extract a node and all descendants.
]

#command("path", arg("cell"), arg(start: none), arg(end: none), ret: "cell")[
  Extract the unique undirected path between two nodes in one component.
]

#command("reroot", arg("cell"), arg(node: none), ret: "cell")[
  Reverse a component's parent chain so #arg[node] becomes its root.
]

#command("prune", arg("cell"), arg(kinds: none), ret: "cell")[
  Remove listed SWC kinds and their complete descendant subtrees.
]

#command(
  "simplify",
  arg("cell"),
  arg(tolerance: none),
  arg(preserve-type-changes: true),
  arg(preserve-soma: true),
  arg(protected-nodes: ()),
  ret: "cell",
)[
  Apply topology-preserving 3D Ramer-Douglas-Peucker simplification.
]

#command("resample", arg("cell"), arg(step: none), arg(protected-nodes: ()), ret: "cell")[
  Resample eligible sections at equal arc-length spacing and return interpolation lineage.
]

#command("export-swc", arg("cell"), ret: str)[
  Return deterministic canonical SWC text with sequential IDs and topological row order.
]

== Annotation Builders

#command("label", arg("body"), arg(node: none), arg(offset: (x: 4pt, y: -4pt)), ret: "annotation")[
  Construct a Typst-native label anchored at a node. #arg[offset] contains Typst `x` and `y` lengths applied after projection; positive `x` moves right and positive `y` moves down.
]

#command(
  "marker",
  arg(node: none),
  arg(body: none),
  arg(offset: (x: 0pt, y: 0pt)),
  arg(size: 5pt),
  arg(fill: white),
  arg(stroke: 0.8pt + black),
  ret: "annotation",
)[
  Construct a node marker. When #arg[body] is omitted, a circle is drawn.
]

#command(
  "cetz-label",
  arg("body"),
  arg(node: none),
  arg(offset: (x: 16mm, y: -10mm)),
  arg(via: ()),
  arg(controls: ()),
  arg(anchor: auto),
  arg(padding: 2pt),
  arg(fill: white),
  arg(label-stroke: none),
  arg(arrow-stroke: 0.7pt + black),
  arg(arrow-fill: black),
  arg(mark: "stealth"),
  arg(mark-scale: 0.7),
  arg(target-gap: 0pt),
  ret: "annotation",
)[
  Construct an optional CeTZ leader label. #arg[offset], every #arg[via] dictionary, and every #arg[controls] dictionary contain Typst `x` and `y` lengths relative to the node; positive `x` moves right and positive `y` moves down. #arg[anchor] accepts `auto` or a CeTZ content-anchor string for label placement. A side leader starts at the text's typographic vertical center. With no #arg[controls] it is a straight line or a #arg[via] polyline; one control selects a quadratic CeTZ Bezier and two select a cubic Bezier. #arg[controls] and #arg[via] cannot be combined. #arg[padding] creates space around the text, #arg[fill] and #arg[label-stroke] style its rectangular background, and the remaining arguments style or shorten the leader and arrow tip.
]

#command("node-anchor", arg("render-result"), arg(node: none), ret: dictionary)[
  Retrieve one projected-node record from a render result. Request the ID with #arg[anchor-nodes], a native annotation, or a CeTZ label and set #arg[return-report] to `true`.
]

#command(
  "cetz-annotate",
  arg("render-result"),
  arg(cetz: none),
  arg(labels: ()),
  arg(length: 1pt),
  arg(strict: true),
  ret: content,
)[
  Compose values returned by @cmd:cetz-label[-] over an existing render result. This two-stage form is useful when the same projected coordinates also drive caller-owned CeTZ elements; the one-stage #arg[cetz-labels] renderer argument is preferred for ordinary leader labels.
]

#command("legend", arg(entries: none), arg(position: top + right), arg(inset: 8pt), ret: dictionary)[
  Construct a compact categorical legend. Each entry has `label` and `color`.
]

#command(
  "color-bar",
  arg(min: none),
  arg(max: none),
  arg(label: none),
  arg(label-gap: 4pt),
  arg(colormap: "viridis"),
  arg(position: bottom + right),
  arg(inset: 8pt),
  ret: dictionary,
)[
  Construct a scalar color bar using the renderer's named palette. #arg[min] and #arg[max] are aligned to the exact left and right ends of the palette strip. #arg[label-gap] is the non-negative vertical space between #arg[label] and the palette strip.
]

#command("scale-bar", arg(value: none), arg(label: none), arg(inset: 8pt), arg(stroke: 1pt), ret: dictionary)[
  Construct a physical scale bar. #arg[value] is expressed in `cell.units`.
]

== Renderer

#command(
  "render",
  arg("cell"),
  arg(projection: "xy"),
  arg(color-by: "type"),
  arg(colormap: "viridis"),
  arg(minimum: none),
  arg(maximum: none),
  arg(width: 120mm),
  arg(height: 90mm),
  arg(geometry: "tapered"),
  arg(radius-mode: "readable"),
  arg(soma-mode: "equivalent-sphere"),
  arg(canvas-width: 800),
  arg(canvas-height: 600),
  arg(padding: 24),
  arg(stroke-width: 2),
  arg(minimum-radius: 1),
  arg(maximum-radius: 18),
  arg(maximum-soma-radius: 96),
  arg(radius-scale: 1),
  arg(radius-exponent: 0.5),
  arg(soma-scale: 1),
  arg(background: none),
  arg(outline-color: none),
  arg(outline-width: 1),
  arg(display-tolerance: none),
  arg(include-nodes: false),
  arg(anchor-nodes: ()),
  arg(labels: ()),
  arg(markers: ()),
  arg(cetz: none),
  arg(cetz-labels: ()),
  arg(legend: none),
  arg(color-bar: none),
  arg(scale-bar: none),
  arg(strict-node-ids: true),
  arg(return-report: false),
  ret: content,
)[
  Render deterministic SVG and compose optional native Typst overlays.

  #argument("projection", types: (str, dictionary), default: "xy")[
    `"xy"`, `"xz"`, `"yz"`, or `(direction:, up:)`.
  ]

  #argument("color-by", types: (str, "node-field"), default: "type")[
    `"type"`, a safe CSS color string, or a fingerprint-bound scalar node field.
  ]

  #argument("colormap", types: str, default: "viridis")[
    `"viridis"` or `"magma"` for scalar fields.
  ]

  #argument("minimum", types: (float, int, none), default: none)[
    Scalar range minimum; inferred from finite field values when omitted.
  ]

  #argument("maximum", types: (float, int, none), default: none)[
    Scalar range maximum; inferred from finite field values when omitted.
  ]

  #argument("width", types: length, default: 120mm)[
    Typst output width. Its ratio with #arg[height] must match the numeric canvas ratio.
  ]

  #argument("height", types: length, default: 90mm)[
    Typst output height.
  ]

  #argument("geometry", types: str, default: "tapered")[
    `"tapered"` or `"skeleton"`.
  ]

  #argument("radius-mode", types: str, default: "readable")[
    `"readable"` or `"physical"`.
  ]

  #argument("soma-mode", types: str, default: "equivalent-sphere")[
    `"equivalent-sphere"`, `"encoded"`, or `"raw-points"`.
  ]

  #argument("canvas-width", types: (float, int), default: 800)[
    Numeric SVG width in pixels. Its ratio with #arg[canvas-height] must equal the Typst output ratio.
  ]

  #argument("canvas-height", types: (float, int), default: 600)[
    Numeric SVG height in pixels.
  ]

  #argument("padding", types: (float, int), default: 24)[
    Minimum fitted clearance in SVG pixels, including painted radii, soma extent, and outlines.
  ]

  #argument("stroke-width", types: (float, int), default: 2)[
    Skeleton stroke width and tapered-geometry distal join allowance in SVG pixels.
  ]

  #argument("minimum-radius", types: (float, int), default: 1)[
    Screen-space lower bound for rendered radii in readable mode.
  ]

  #argument("maximum-radius", types: (float, int), default: 18)[
    Screen-space upper bound for non-soma radii in readable mode.
  ]

  #argument("maximum-soma-radius", types: (float, int), default: 96)[
    Independent screen-space upper bound for soma radii in readable mode.
  ]

  #argument("radius-scale", types: (float, int), default: 1)[
    Multiplicative scale applied to neurite radii before screen-space bounds.
  ]

  #argument("radius-exponent", types: (float, int), default: 0.5)[
    Readable-mode exponent applied to projected radii; `1` preserves linear scaling before bounds.
  ]

  #argument("soma-scale", types: (float, int), default: 1)[
    Multiplicative scale applied to the displayed soma radius.
  ]

  #argument("background", types: (str, none), default: none)[
    Safe CSS color string for the SVG canvas, or `none` for transparency.
  ]

  #argument("outline-color", types: (str, none), default: none)[
    Safe CSS color string for a segment and soma halo, or `none` to disable it.
  ]

  #argument("outline-width", types: (float, int), default: 1)[
    Outline width in SVG pixels; ignored when #arg[outline-color] is `none`.
  ]

  #argument("display-tolerance", types: (float, int, none), default: none)[
    Optional morphology-unit tolerance for display-only topology-preserving simplification.
  ]

  #argument("include-nodes", types: bool, default: false)[
    Render every retained sample node as an SVG marker and include every projected node in #arg[node-anchors]. Leave this off for ordinary figures and request sparse coordinates with #arg[anchor-nodes].
  ]

  #argument("anchor-nodes", types: array, default: ())[
    Integer node IDs whose final projected coordinates should be protected from display simplification and included in the render result without drawing markers.
  ]

  #argument("labels", types: array, default: ())[
    Sequence returned by @cmd:label[-]. Target IDs are protected from display simplification.
  ]

  #argument("markers", types: array, default: ())[
    Sequence returned by @cmd:marker[-]. Target IDs are protected from display simplification.
  ]

  #argument("cetz", types: (module, none), default: none)[
    Caller-imported CeTZ module. Required only when #arg[cetz-labels] is non-empty; Axodendron does not import CeTZ itself.
  ]

  #argument("cetz-labels", types: array, default: ())[
    Sequence returned by @cmd:cetz-label[-]. Targets are protected, projected, registered as CeTZ anchors, and composed over the base render.
  ]

  #argument("legend", types: (dictionary, none), default: none)[
    Overlay returned by @cmd:legend[-].
  ]

  #argument("color-bar", types: (dictionary, none), default: none)[
    Overlay returned by @cmd:color-bar[-].
  ]

  #argument("scale-bar", types: (dictionary, none), default: none)[
    Overlay returned by @cmd:scale-bar[-].
  ]

  #argument("strict-node-ids", types: bool, default: true)[
    Reject unknown native-label, marker, CeTZ-label, or explicit anchor IDs. If `false`, missing optional annotations are skipped.
  ]

  #argument("return-report", types: bool, default: false)[
    Return a dictionary containing `body`, base dimensions, projected node anchors, and fit, simplification, radius, and overlay metadata instead of content alone.
  ]
]

#command(
  "render-tree",
  arg("cell"),
  arg(depth: "topological"),
  arg(color-by: "type"),
  arg(domain: "neurites"),
  arg(width: 120mm),
  arg(height: 90mm),
  arg(return-report: false),
  ret: content,
)[
  Render an abstract rooted forest. Depth is `topological`, `path-length`, or `radial-distance`; branch and Strahler orders are supplied as discrete color fields. The full function also accepts the common selection, canvas, node-anchor, and CeTZ-label arguments documented for physical rendering.
]

= Result Schemas

== Analysis Bundle

The analysis bundle contains `schema-version`, `definition-version`, `fingerprint`, `domain`, `summary`, `topology`, `sections`, `tortuosity`, `root-path-length`, `radial-distance`, `branch-order`, and `strahler-order`. The current definition version is `axodendron-morphometrics-1`.

The summary includes raw/domain node counts, edges, roots, components, branches, terminals, sections, total cable length, maximum path and radial distances, bounding box, type counts and metrics, per-arbor metrics, radius metrics, soma metrics and class, and units.

== Metric Result

A `MetricResult` contains `schema_version`, `metric`, `selection`, `source`, `provenance`, `data`, and `missing`. `metric` separates `id`, `definition_version`, and resolved `parameters`. `source` contains morphology, topology, and selection fingerprints. `data.kind` is a tagged support type; its `value` contains aligned entity keys, values, and units. `missing` contains explicit entity/reason/detail records.

== Principal Frame

A principal frame contains matching morphology and topology fingerprints, the resolved selection and weighting, origin mode and coordinate, centroid, three ordered axes and eigenvalues, min/max extents, rank, ambiguous-axis flags, and provenance containing definition version, covariance model, sign and handedness rules, tolerances, and selection fingerprint.

== TMD Result

A TMD result contains schema and definition provenance, matching fingerprints, query, filtration, optional radial center, units, node-aligned filtration values, persistence pairs, and the non-monotone pair count. Definition version `2` makes each selected arbor root zero for root-path length and forbids a center for that filtration. `provenance.pair_order` records the deterministic barcode order. Essential and radial non-monotone conventions are explicit fields rather than implicit plotting behavior.

== Feature Table

A feature table contains versioned column metadata, rows with morphology fingerprints and tagged value/missing cells, and descriptive summaries. Column metadata retains the metric descriptor, aggregate, structured component, missing policy, and units.

== Transform Result

Transformed cells add `transform-report`, `mapping`, and `lineage`. `mapping` contains `old-id` and optional `new-id`; `lineage` contains `new-id`, `proximal-old-id`, `distal-old-id`, and `distal-fraction` for inserted samples. Geometry operations additionally record API class, affine matrix and translation, determinant, radius scale and policy, and whether SWC radius representation is lossy.

== Render Result

With `return-report: true`, the renderer returns `body`, `width`, `height`, `canvas-width`, `canvas-height`, `node-anchors`, `report`, `pixels-per-unit`, `source-node-count`, and `rendered-node-count`. A node-anchor record contains `node`, top-left-relative `x` and `y` lengths, normalized `x-ratio` and `y-ratio`, SVG `screen-x` and `screen-y`, and projected `depth`. Use `body` as content and retain the result when a reproducible figure must disclose display simplification, radius clamping, or annotation coordinates.

= Limitations

- Axodendron renders orthographic views only; it does not provide perspective projection, lighting, or volumetric microscopy rendering.
- SWC expresses centerlines and sample radii, not membrane meshes, synapses, confidence intervals, or image registration. Axodendron does not infer information absent from the file.
- The default readable radius mode is a display policy, not a literal physical cross-section. Use physical mode and report it when width carries quantitative meaning.
- Equivalent-sphere soma display is a visual summary. Only supported soma encodings receive documented area and volume metrics.
- Scalar color is linear and supports Viridis or Magma. Axodendron does not automatically choose diverging, logarithmic, or categorical maps for a scientific hypothesis.
- Native and CeTZ labels use exact projected node anchors and explicit offsets; CeTZ `via` points define polylines and `controls` define Bezier curves, but collision-free placement is not automatic.
- Rendering tests guarantee deterministic SVG structure and broad real-data coverage. CI additionally hashes the exact decoded RGBA dimensions and pixels of a PNG baseline under Typst 0.14.2, so PNG metadata and compression do not create false regressions; this fixed-toolchain baseline does not claim identical raster output across unrelated PDF engines, fonts, or operating systems.

= License, Dependencies, and Data Citations

Axodendron source is distributed under the MIT license. The compiled WASM includes `ciborium`, `serde`, `half`, `cfg-if`, `zerocopy`, and `wasm-minimal-protocol`; their selected licenses and exact versions are recorded in `THIRD_PARTY_NOTICES.md` and `wasm-plugin/Cargo.lock`. CeTZ 0.5.2 is an optional LGPL-3.0-or-later example and documentation dependency supplied by the caller; its code is not embedded in Axodendron or `plugin.wasm`.

The four real SWC files used throughout this manual remain CC BY 4.0 material. Their per-file record links, archive names, original studies, and SHA-256 checksums are recorded in `THIRD_PARTY_NOTICES.md`. Every rendered occurrence in this manual also carries an adjacent attribution line.

When reusing a real-data figure, preserve its adjacent attribution, cite the original study, cite NeuroMorpho.Org with RRID:SCR_002145, and follow the NeuroMorpho.Org terms of use. The database citation used by this manual is Tecuatl C, Ljungquist B, Ascoli GA (2024), _Accelerating the continuous community sharing of digital neuromorphology data_, FASEB BioAdvances 6(7):207-221, #link("https://doi.org/10.1096/fba.2024-00048")[doi:10.1096/fba.2024-00048].

Metric fixtures are recomputed against NeuroM 4.0.5's #link("https://neurom.readthedocs.io/en/stable/_neurom_build/neurom.features.bifurcation.html")[bifurcation] and #link("https://neurom.readthedocs.io/en/stable/_neurom_build/neurom.features.section.html")[section] functions and L-Measure 5.2 revision 510 `Pk_classic`. The committed fixture records upstream callable, native units and values, Axodendron values, tolerance, compatibility class, and every known convention or precision difference. TMD provenance cites Kanari L et al. (2018), _A Topological Representation of Branching Neuronal Morphologies_, Neuroinformatics 16:3–13, #link("https://doi.org/10.1007/s12021-017-9341-1")[doi:10.1007/s12021-017-9341-1]. Diameter-power interpretation follows the recorded exponent rather than treating $p=3/2$ as a universal biological constant.

#info-alert[
  The detailed file-level notice is authoritative for bundled third-party material. This manual summarizes it for figure readers but does not replace `THIRD_PARTY_NOTICES.md`.
]
