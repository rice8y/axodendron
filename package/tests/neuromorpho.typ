#import "../lib.typ" as swc

// Optional local-only end-to-end check. The referenced SWCs are downloaded to
// the ignored target cache and are never part of the package or test source.
#let neuron = swc.load(
  read("../../target/neuromorpho/NMO_00001.swc", encoding: none),
  profile: "incf-strict",
)
#let glia = swc.load(
  read("../../target/neuromorpho/NMO_200000.swc", encoding: none),
  profile: "incf-strict",
)

#assert.eq(neuron.node-count, 1274)
#assert.eq(glia.node-count, 568)

#set page(width: 180mm, height: 180mm, margin: 8mm)
#grid(
  columns: (1fr, 1fr),
  rows: (1fr, 1fr),
  gutter: 4mm,
  swc.render(neuron, projection: "xy", width: 80mm, height: 80mm, canvas-width: 800, canvas-height: 800),
  swc.render(neuron, projection: "xz", color-by: "#111827", width: 80mm, height: 80mm, canvas-width: 800, canvas-height: 800),
  swc.render(glia, projection: "xy", width: 80mm, height: 80mm, canvas-width: 800, canvas-height: 800),
  swc.render(glia, projection: "yz", color-by: "#111827", width: 80mm, height: 80mm, canvas-width: 800, canvas-height: 800),
)
