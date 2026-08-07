// This version of the code comes from:
// https://inspirnathan.com/posts/52-shadertoy-tutorial-part-6/
// from: Nathan Vaughn
// Conversion from glsl to wgsl was reasonably straight forward.
//

@group(0) @binding(0)
var<uniform> screen_xy: vec2u;

@group(0) @binding(1)
var<uniform> transformer: mat4x4f;

@group(0) @binding(2)
var<uniform> timer: f32;

@group(0) @binding(3)
var<uniform> scale: f32;

// @group(0) @binding(0)
// var<uniform> screen_x: u32;

// @group(0) @binding(1)
// var<uniform> screen_y: u32;

//////////////////////////////////////////////////////////////////////////
//
//  Vertex shader - normalizes screen coordinates in xy
//
//////////////////////////////////////////////////////////////////////////

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


    var out: VertexOutput;
    out.position = vec4f(pos[index], 0.0, 1.0);
    out.xy = xy[index];
    return out;
}

const MAX_MARCHING_STEPS = 255;
const MIN_DIST = 0.0;
const MAX_DIST = 100.0;
const PRECISION = 0.001;
const EPSILON = 0.0005;
const TWOPI = 2 * 3.14159265359;

// float sdSphere(vec3 p, float r )
fn sdSphere(p: vec3f, r: f32) -> f32
{
  let offset = vec3f(0, 0, -2);
  return length(p - offset) - r;
}

// distance from a box
fn box(p: vec3f, b: vec3f) -> f32 {
  let q = abs(p) - b;
  // let zero = vec3(0.0);
  return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

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

// Rotate around x with theta = timer
fn shape7(p: vec3f) -> f32 {
  let s = 0.5;
  let p1 = trans(p, scaling(s));
  return box(p1, vec3(1.0, 0.5, 0.5)) * s;
}

// Rotate around x with theta = timer
fn shape6(p: vec3f) -> f32 {
  let p1 = trans(p, rotx(timer));
  return box(p1, vec3(1.0, 0.5, 0.5));
}

// Rotate around y with theta = timer
fn shape5(p: vec3f) -> f32 {
  let p1 = trans(p, roty(timer));
  return box(p1, vec3(1.0, 0.5, 0.5));
}

// Rotate around z with theta = timer
fn shape4(p: vec3f) -> f32 {
  let p1 = trans(p, rotz(timer));
  return box(p1, vec3(1.0, 0.5, 0.5));
}

// Use transformer from cpu
fn shape3(p: vec3f) -> f32 {
  let p1 = trans(p, transformer);
  return box(p1, vec3(1.0, 0.5, 0.5)) * scale;
}

// Rotate and then translate a box
fn shape2(p: vec3f) -> f32 {
  // let p1 = trans(p, translate(1.0, 0.0, 0.0) * rotz(0.125 * TWOPI));
  let p1 = trans(p, rotz(0.125 * TWOPI) * translate(1.0, 0.0, 0.0));
  return box(p1, vec3(1.0, 0.5, 0.5));
}

// Translate and then rotate a box
fn shape1(p: vec3f) -> f32 {
  let p1 = trans(p, translate(1.0, 0.0, 0.0) * rotz(0.125 * TWOPI));
  // let p1 = trans(p, rotz(0.125 * TWOPI) * translate(1.0, 0.0, 0.0));
  return box(p1, vec3(1.0, 0.5, 0.5));
}

fn theShape(p: vec3f) -> f32 { return shape3(p); }

// float rayMarch(vec3 ro, vec3 rd, float start, float end) {
fn rayMarch(ro: vec3f, rd: vec3f, start: f32, end: f32) -> f32 {
  var depth = start;

  for (var i = 0; i < MAX_MARCHING_STEPS; i++) {
    let p = ro + depth * rd;
    let d = theShape(p);
    depth += d;
    if d < PRECISION || depth > end { break; }
  }

  return depth;
}

// vec3 calcNormal(vec3 p) {
fn calcNormal(p: vec3f) -> vec3f {
    let e = vec2f(1.0, -1.0) * 0.0005; // epsilon
    // let r = 1.; // radius of sphere
    return normalize(
      e.xyy * theShape(p + e.xyy) +
      e.yyx * theShape(p + e.yyx) +
      e.yxy * theShape(p + e.yxy) +
      e.xxx * theShape(p + e.xxx));
}

// void mainImage( out vec4 fragColor, in vec2 fragCoord ) {
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  // let uv = (fragCoord-.5*iResolution.xy)/iResolution.y;
  let backgroundColor = vec3f(0.835, 1, 1);

  var col = vec3f(0);
  let ro = vec3f(0, 0, 3); // ray origin that represents camera position
  let rd = normalize(vec3(in.xy, -1)); // ray direction

  let d = rayMarch(ro, rd, MIN_DIST, MAX_DIST); // distance to sphere

  if (d > MAX_DIST) {
    col = backgroundColor; // ray didn't hit anything
  } else {
    let p = ro + rd * d; // point on sphere we discovered from ray marching
    let normal = calcNormal(p);
    let lightPosition = vec3f(2, 2, 7);
    let lightDirection = normalize(lightPosition - p);

    // Calculate diffuse reflection by taking the dot product of
    // the normal and the light direction.
    let dif = clamp(dot(normal, lightDirection), 0.3, 1.);

    // Multiply the diffuse reflection value by an orange color and add a bit
    // of the background color to the sphere to blend it more with the background.
    col = dif * vec3(1, 0.58, 0.29) + backgroundColor * .2;
  }

  // Output to screen
  return vec4(col, 1.0);
}
