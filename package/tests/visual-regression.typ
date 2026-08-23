#import "../lib.typ" as swc

#set page(width: 54mm, height: 54mm, margin: 3mm, fill: white)

#let cell = swc.from-text(
  "1 1 0 0 0 3 -1\n"
  + "2 3 0 4 0 1.4 1\n"
  + "3 3 -5 9 0 1.1 2\n"
  + "4 3 -8 15 0 0.7 3\n"
  + "5 3 -2 15 0 0.6 3\n"
  + "6 4 5 9 0 1.0 2\n"
  + "7 4 10 15 0 0.55 6\n"
  + "8 4 3 16 0 0.65 6\n"
  + "9 2 -4 -5 0 0.9 1\n"
  + "10 2 -9 -9 0 0.55 9\n"
  + "11 2 -14 -10 0 0.35 10\n",
  profile: "incf-strict",
)

#swc.render(
  cell,
  width: 48mm,
  height: 48mm,
  canvas-width: 480,
  canvas-height: 480,
  padding: 20,
  geometry: "tapered",
  radius-mode: "readable",
  color-by: "type",
)
