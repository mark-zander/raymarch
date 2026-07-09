// This version of the code comes from:
// https://inspirnathan.com/posts/52-shadertoy-tutorial-part-6/
// from: Nathan Vaughn
// Conversion from glsl to wgsl was reasonably straight forward.
//

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
    let x = f32(screen_x) / f32(screen_y);
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

// float sdSphere(vec3 p, float r )
fn sdSphere(p: vec3f, r: f32) -> f32
{
  let offset = vec3f(0, 0, -2);
  return length(p - offset) - r;
}

// float rayMarch(vec3 ro, vec3 rd, float start, float end) {
fn rayMarch(ro: vec3f, rd: vec3f, start: f32, end: f32) -> f32 {
  var depth = start;

  for (var i = 0; i < MAX_MARCHING_STEPS; i++) {
    let p = ro + depth * rd;
    let d = sdSphere(p, 1.);
    depth += d;
    if d < PRECISION || depth > end { break; }
  }

  return depth;
}

// vec3 calcNormal(vec3 p) {
fn calcNormal(p: vec3f) -> vec3f {
    let e = vec2f(1.0, -1.0) * 0.0005; // epsilon
    let r = 1.; // radius of sphere
    return normalize(
      e.xyy * sdSphere(p + e.xyy, r) +
      e.yyx * sdSphere(p + e.yyx, r) +
      e.yxy * sdSphere(p + e.yxy, r) +
      e.xxx * sdSphere(p + e.xxx, r));
}

@group(0) @binding(0)
var<uniform> screen_x: u32;

@group(0) @binding(1)
var<uniform> screen_y: u32;

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
