#import "../lib.typ" as swc
#import "@preview/cetz:0.5.2"

#let source = "# CREATURE: synthetic\n1 1 0 0 0 2 -1\n2 3 0 5 0 1 1\n3 3 -4 10 0 0.8 2\n4 3 4 10 0 0.8 2\n"
#let cell = swc.from-text(source, profile: "incf-strict")
#let result = swc.analyze(cell)

#if sys.version >= version(0, 15, 0) {
  let path-cell = swc.load(
    path("../examples/data/AA0109.CNG.swc"),
    profile: "incf-strict",
  )
  assert.eq(path-cell.node-count, 2172)
}

#assert(cell.valid)
#assert.eq(cell.node-count, 4)
#assert(cell.fingerprint != none)
#assert(cell.at("source-fingerprint") != none)
#assert.eq(swc.metadata(cell).fields.at("creature"), ("synthetic",))
#assert.eq(result.summary.node_count, 3)
#assert.eq(result.summary.raw_node_count, 4)
#assert.eq(result.summary.edge_count, 2)
#assert.eq(result.summary.branch_point_count, 1)
#assert.eq(result.summary.terminal_count, 2)
#assert.eq(result.topology.root_ids, (2,))
#assert.eq(result.topology.branch_point_ids, (2,))
#assert.eq(result.branch_order.values, (1.0, 2.0, 2.0))
#assert.eq(result.strahler_order.values, (2.0, 1.0, 1.0))
#assert.eq(result.radial_distance.values.at(0), 0.0)
#assert(calc.abs(result.radial_distance.values.at(1) - calc.sqrt(41)) < 0.000001)
#assert.eq(swc.analyze(cell, domain: "raw").summary.node_count, 4)
#assert.eq(result.root_path_length.fingerprint, cell.fingerprint)

#let selected-nodes = swc.select-nodes(cell, nodes: (2, 3))
#assert.eq(selected-nodes.node-count, 2)
#assert.eq(swc.analyze(selected-nodes).topology.root_ids, (2,))
#let selected-kinds = swc.select-kinds(cell, kinds: (3,))
#assert.eq(selected-kinds.node-count, 3)
#let branch = swc.subtree(cell, node: 2)
#assert.eq(branch.node-count, 3)
#assert.eq(branch.at("source-fingerprint"), cell.at("source-fingerprint"))
#assert(branch.fingerprint != cell.fingerprint)
#let route = swc.path(cell, start: 3, end: 4)
#assert.eq(route.node-count, 3)
#let rerooted = swc.reroot(cell, node: 3)
#assert.eq(swc.analyze(rerooted).topology.root_ids, (3,))
#let pruned = swc.prune(cell, kinds: (3,))
#assert.eq(pruned.node-count, 1)
#let simplified = swc.simplify(cell, tolerance: 0.1)
#assert(simplified.node-count <= cell.node-count)
#assert(simplified.at("transform-report").operation == "simplify")
#let resampled = swc.resample(cell, step: 2)
#assert(resampled.lineage.len() > 0)
#assert(swc.export-swc(cell).contains("axodendron-canonical"))

#let invalid = swc.from-text(
  "1 1 0 0 0 1 -1\n1 3 1 0 0 1 1\n",
  fail-on-error: false,
)
#assert(not invalid.valid)
#assert(invalid.diagnostics.any(item => item.code == "SWC_DUPLICATE_ID"))

#let forest = swc.from-text(
  "10 1 0 0 0 1 -1\n20 9 10 0 0 1 -1\n",
  fail-on-error: false,
)
#assert(forest.valid)
#assert(forest.diagnostics.any(item => item.code == "SWC_CUSTOM_TYPE"))
#assert(forest.diagnostics.any(item => item.code == "SWC_DISCONNECTED_COMPONENT"))

#let crossings = swc.sholl(cell, radii: (2, 5, 8), center-node: 1)
#assert.eq(crossings.bins.len(), 3)
#let crossings-2d = swc.sholl-2d(cell, radii: (2, 5), projection: "xy", center-node: 1)
#assert.eq(crossings-2d.dimension, "two-dimensional")

#let offset-label = swc.label(node: 3, offset: (x: -40pt, y: -14pt), [terminal])
#assert.eq(offset-label.offset, (x: -40pt, y: -14pt))
#let spaced-color-bar = swc.color-bar(
  min: 0,
  max: 10,
  label: [path],
  label-gap: 5pt,
)
#assert.eq(spaced-color-bar.min, 0)
#assert.eq(spaced-color-bar.max, 10)
#assert.eq(spaced-color-bar.label-gap, 5pt)

#let anchored = swc.render(
  cell,
  width: 60mm,
  height: 45mm,
  canvas-width: 400,
  canvas-height: 300,
  display-tolerance: 0.1,
  anchor-nodes: (2, 3, 3),
  return-report: true,
)
#assert.eq(anchored.node-anchors.len(), 2)
#assert.eq(anchored.report.overlay_node_count, 2)
#assert.eq(anchored.rendered-node-count, 4)
#let projected-terminal = swc.node-anchor(anchored, node: 3)
#assert.eq(projected-terminal.node, 3)
#assert(projected-terminal.x >= 0pt and projected-terminal.x <= anchored.width)
#assert(projected-terminal.y >= 0pt and projected-terminal.y <= anchored.height)
#assert(projected-terminal.x-ratio >= 0 and projected-terminal.x-ratio <= 1)
#assert(projected-terminal.y-ratio >= 0 and projected-terminal.y-ratio <= 1)

#let leader = swc.cetz-label(
  node: 3,
  offset: (x: 12mm, y: -8mm),
  via: ((x: 5mm, y: -8mm),),
  target-gap: 1pt,
  [terminal node],
)
#assert.eq(leader.node, 3)
#assert.eq(leader.anchor, auto)
#assert.eq(leader.via.len(), 1)
#assert.eq(leader.controls.len(), 0)

#let curved-leader = swc.cetz-label(
  node: 4,
  offset: (x: -12mm, y: -8mm),
  controls: (
    (x: -9mm, y: -8mm),
    (x: -5mm, y: -3mm),
  ),
  [curved terminal label],
)
#assert.eq(curved-leader.via.len(), 0)
#assert.eq(curved-leader.controls.len(), 2)

#let quadratic-leader = swc.cetz-label(
  node: 2,
  offset: (x: 0mm, y: -12mm),
  controls: ((x: 4mm, y: -6mm),),
  [quadratic branch label],
)
#assert.eq(quadratic-leader.controls.len(), 1)

#let cetz-figure = swc.render(
  cell,
  width: 60mm,
  height: 45mm,
  canvas-width: 400,
  canvas-height: 300,
  cetz: cetz,
  cetz-labels: (leader, curved-leader, quadratic-leader),
  return-report: true,
)
#assert.eq(cetz-figure.node-anchors.len(), 3)
#assert.eq(cetz-figure.report.overlay_node_count, 3)

#let figure = swc.render(
  cell,
  projection: (
    direction: (x: 1, y: 1, z: 1),
    up: (x: 0, y: 0, z: 1),
  ),
  color-by: result.root_path_length,
  width: 90mm,
  height: 67.5mm,
  display-tolerance: 0.1,
  labels: (offset-label,),
  markers: (swc.marker(node: 4),),
  legend: swc.legend(entries: (
    (label: [dendrite], color: "#009e73"),
  )),
  color-bar: spaced-color-bar,
  scale-bar: swc.scale-bar(value: 5),
  return-report: true,
)
#assert.eq(figure.report.radius_mode, "readable")
#assert.eq(figure.report.overlay_node_count, 2)
#figure.body
#cetz-figure.body
