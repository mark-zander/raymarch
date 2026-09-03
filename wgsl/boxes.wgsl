fn theShape(p: vec3f) -> Surface {
  let id = u32(0);
  let p1 = trans(p, transformer[id]);
  var bb = both_box(p1);
  bb.dist = bb.dist * scale[id];
  return bb;
}

// fn theShape(p: vec3f) -> Surface {
//   return Surface(
//     box(p, vec3(1.0, 0.5, 0.5)),
//     0,
//     ORANGE
//   );
// }

// Rotate around x with theta = timer
fn box_timer(p: vec3f) -> f32 {
  let p1 = trans(p, rotx(timer));
  return box(p1, vec3(1.0, 0.5, 0.5));
}

// fn box1(p: vec3f) -> f32 {
//   let p1 = trans(p, transformer[1] * translate(1.0, 1.0, 0.0));
//   return box(p1, vec3(1.0, 0.5, 0.5)) * scale[1];
// }

fn box1(p: vec3f) -> Surface {
  let id = u32(1);
  let p1 = trans(p, transformer[id] * translate(1.0, 1.0, 0.0));
  return Surface(
    box(p1, vec3(1.0, 0.5, 0.5)) * scale[id],
    id,
    ORANGE
  );
}

// fn box2(p: vec3f) -> f32 {
//   let p1 = trans(p, transformer[2] * translate(-1.0, -1.0, 0.0));
//   return box(p1, vec3(1.0, 0.5, 0.5)) * scale[2];
// }

fn box2(p: vec3f) -> Surface {
  let id = u32(2);
  let p1 = trans(p, transformer[id] * translate(-1.0, -1.0, 0.0));
  return Surface(
    box(p1, vec3(1.0, 0.5, 0.5)) * scale[id],
    id,
    CYAN
  );
}

fn both_box(p: vec3f) -> Surface {
  return unions(box1(p), box2(p));
}

