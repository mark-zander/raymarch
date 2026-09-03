// This version of the code comes from:
// https://inspirnathan.com/posts/52-shadertoy-tutorial-part-6/
// from: Nathan Vaughn
// Conversion from glsl to wgsl was reasonably straight forward.
//

@group(0) @binding(0)
var<uniform> screen_xy: vec2u;

@group(0) @binding(1)
var<uniform> timer: f32;

// arrayLength(&transformer)
@group(0) @binding(2)
var<storage, read> transformer: array<mat4x4f>;

@group(0) @binding(3)
var<storage, read> scale: array<f32>;

@group(0) @binding(4)
var<uniform> mouse_hit: u32;

@group(0) @binding(5)
var<uniform> cursor_xy: vec2u;

@group(0) @binding(6)
var<storage, read_write> node_id: u32;

@group(0) @binding(7)
var<uniform> mouse_pos: vec2u;

const ID4X4F = mat4x4<f32>(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 1.0, 0.0,
  0.0, 0.0, 0.0, 1.0,
);

////////////////////////////////////////////////////////////////////
//
//  Vertex shader - normalizes screen coordinates in xy
//
////////////////////////////////////////////////////////////////////

struct VertexOutput {
    // screen position in pixels from upper left
    @builtin(position) position: vec4f,
    // screen position in view coordinates
    @location(0) xy: vec2f,
};

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
) -> VertexOutput {

    // let pos = ...
    // May only be indexed by a constant where as:
    // var pos = ...
    // May be indexed by a varialble. Now fixed.
    let pos = array<vec2f,6>(
        vec2f( -1.0, -1.0),
        vec2f(  1.0, -1.0),
        vec2f( -1.0,  1.0),

        vec2f( -1.0,  1.0),
        vec2f(  1.0, -1.0),
        vec2f(  1.0,  1.0),
    );

    // might make more sense for this to go from 0 to 1 for easier calcs
    // or could use vertex buffer to load in extent rather than calculate
    let x = f32(screen_xy.x) / f32(screen_xy.y);
    let xy = array<vec2f,6>(
        vec2f(-x, -1.0),
        vec2f( x, -1.0),
        vec2f(-x,  1.0),

        vec2f(-x,  1.0),
        vec2f( x, -1.0),
        vec2f( x,  1.0),
    );

    // let xy = array<vec2f,6>(
    //     vec2f(-1.0, -1.0),
    //     vec2f( 1.0, -1.0),
    //     vec2f(-1.0,  1.0),

    //     vec2f(-1.0,  1.0),
    //     vec2f( 1.0, -1.0),
    //     vec2f( 1.0,  1.0),
    // );

    // node_trans[0] = transformer;

    var out: VertexOutput;
    out.position = vec4f(pos[index], 0.0, 1.0);
    out.xy = xy[index];
    return out;
}

////////////////////////////////////////////////////////////////////
//
//  Fragment shader - displays color for each pixel and shape
//
////////////////////////////////////////////////////////////////////

const MAX_MARCHING_STEPS = 255;
const MIN_DIST = 0.0;
const MAX_DIST = 100.0;
const PRECISION = 0.001;
const EPSILON = 0.0005;
const TWOPI = 2 * 3.14159265359;

// float rayMarch(vec3 ro, vec3 rd, float start, float end) {
fn rayMarch(ro: vec3f, rd: vec3f, start: f32, end: f32) -> Surface {
  var depth = start;
  var surf: Surface;

  for (var i = 0; i < MAX_MARCHING_STEPS; i++) {
    let p = ro + depth * rd;
    surf = theShape(p);
    let d = surf.dist;
    depth += d;
    if d < PRECISION || depth > end { break; }
  }

  surf.dist = depth;

  return surf;
}

// vec3 calcNormal(vec3 p) {
fn calcNormal(p: vec3f) -> vec3f {
    let e = vec2f(1.0, -1.0) * 0.0005; // epsilon
    // let r = 1.; // radius of sphere
    return normalize(
      e.xyy * theShape(p + e.xyy).dist +
      e.yyx * theShape(p + e.yyx).dist +
      e.yxy * theShape(p + e.yxy).dist +
      e.xxx * theShape(p + e.xxx).dist);
}

// Material could be returned using function argument pointers if
// performance is an issue.
struct Material {
    color: vec4f,
}

struct Surface {
    dist: f32,
    node_id: u32,
    material: Material,
}

const BLACK = Material(vec4f(0.0, 0.0, 0.0, 1.0)); 
const RED = Material(vec4f(1.0, 0.0, 0.0, 1.0)); 
const CYAN = Material(vec4f(0.0, 0.8, 0.8, 1.0));
const ORANGE = Material(vec4f(1.0, 0.58, 0.29, 1.0));
const BRIGHT_CYAN = Material(vec4f(0.835, 1.0, 1.0, 1.0));
const BACKGROUND = BRIGHT_CYAN;

// void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  // let uv = (fragCoord-.5*iResolution.xy)/iResolution.y;
  // let backgroundColor = vec3f(0.835, 1, 1);

  var col = BACKGROUND.color;
  let ro = vec3f(0, 0, 3); // ray origin that represents camera position
  let rd = normalize(vec3(in.xy, -1)); // ray direction

  let surf = rayMarch(ro, rd, MIN_DIST, MAX_DIST); // distance to shape
  let d = surf.dist;
  let pos = vec2u(u32(in.position.x), u32(in.position.y));

  if (d > MAX_DIST) {
    col = BACKGROUND.color; // ray didn't hit anything
    if all(pos.xy == mouse_pos) { node_id = 0; }
  } else {
    let p = ro + rd * d; // point on sphere we discovered from ray marching
    let normal = calcNormal(p);
    let lightPosition = vec3f(2, 2, 7);
    let lightDirection = normalize(lightPosition - p);

    // Calculate diffuse reflection by taking the dot product of
    // the normal and the light direction.
    // let dif = clamp(dot(normal, lightDirection), 0.3, 1.);
    let dif = clamp(dot(normal, lightDirection), 0.3, 1.);

    // Multiply the diffuse reflection value by an orange color and add a bit
    // of the background color to the sphere to blend it more with the background.
    // col = dif * vec4(1, 0.58, 0.29, 1.0) + BACKGROUND.color * .2;
    col = dif * surf.material.color + BACKGROUND.color * .2;
    if node_id == surf.node_id {
      col = dif * RED.color + BACKGROUND.color * .2;
    }
    if all(pos.xy == mouse_pos) { node_id = surf.node_id; }
  }

  // if all(pos.xy == mouse_pos) {
  //   node_id = surf.node_id;
  //   if node_id != 0 {
  //     col = dif * RED.color + BACKGROUND.color * .2;
  //   }
  // }
  // if all(pos.xy == cursor_xy) { node_id = surf.node_id; }

  // Output to screen
  return col;
}

/////////////////////////////////////////////////
// some shapes to us in creating theShape

fn unions(c1: Surface, c2: Surface) -> Surface {
    if c1.dist < c2.dist { return c1; }
    return c2;
}

// float sdSphere(vec3 p, float r )
fn sdSphere(p: vec3f, r: f32) -> f32 {
  return length(p) - r;
}

// distance from a box
fn box(p: vec3f, b: vec3f) -> f32 {
  let q = abs(p) - b;
  return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// float sdPlane( vec3 p, vec3 n, float h )
// {
//   // n must be normalized
//   return dot(p,n) + h;
// }

// fn sdCappedCylinder( vec3 p, float h, float r ) -> f32 {
fn cappedCylinder(p: vec3f, h: f32, r: f32) -> f32 {
  let d = abs(vec2(length(p.xz),p.y)) - vec2(r,h);
  return min(max(d.x, d.y), 0.0) + length(max(d, vec2(0.0)));
}

// Union without the material
fn mind(c1: f32, c2: f32) -> f32 {
    if c1 < c2 { return c1; }
    return c2;
}

// Intersection without the material
fn maxd(c1: f32, c2: f32) -> f32 {
    if c1 > c2 { return c1; }
    return c2;
}

// Subtract without the material
fn minusd(c1: f32, c2: f32) -> f32 {
    if -c1 > c2 { return -c1; }
    return c2;
}

// Invert without the material
fn negd(c1: f32) -> f32 { return -c1; }

// Will transformations typically take place in the cpu?
// transform point by a matrix
fn trans(p: vec3f, m: mat4x4<f32>) -> vec3f {
    return (m * vec4(p, 1.0)).xyz;
}

fn translate(x: f32, y: f32, z: f32) -> mat4x4<f32> {
    return mat4x4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -x,  -y,  -z,  1.0
    );
}

// matrix to rotate around X axis
// accidentally works, for signed distance functions we need to
// invert the matrix but these are already inverted since the 
// matrix is given in row major order but should be in column
// major order. For rotations on axis the transpose is the inverse.
fn rotx(theta: f32) -> mat4x4<f32> {
  let c = cos(theta);
  let s = sin(theta);
    return mat4x4(
        1.0, 0.0, 0.0, 0.0,
        0.0, c,   -s,  0.0,
        0.0, s,   c,   0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

// matrix to rotate around Y axis
fn roty(theta: f32) -> mat4x4<f32> {
  let c = cos(theta);
  let s = sin(theta);
    return mat4x4(
        c,   0.0, s,   0.0,
        0.0, 1.0, 0.0, 0.0,
        -s,  0.0, c,   0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

// matrix to rotate around Z axis
fn rotz(theta: f32) -> mat4x4<f32> {
  let c = cos(theta);
  let s = sin(theta);
    return mat4x4(
        c,   -s,  0.0, 0.0,
        s,   c,   0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

// matrix for scaling, all axis must be scaled the same
// must divide by scale factor on return from shape
fn scaling(s: f32) -> mat4x4f {
    let s1 = 1.0 / s;
    return mat4x4(
        s1,  0.0, 0.0, 0.0,
        0.0, s1,  0.0, 0.0,
        0.0, 0.0, s1,  0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

