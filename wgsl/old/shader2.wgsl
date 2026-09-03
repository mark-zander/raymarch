//  Start from github.com/electricsquare/raymarching-workshop
//  This is an glsl code meant to work with shadertoy so it needs to be
//  converted into wgsl.


//////////////////////////////////////////////////////////////////////////
//
//  Vertex shader - normalizes screen coordinates in xy
//
//////////////////////////////////////////////////////////////////////////

struct VertexOutput {
    // screen position in pixels from upper left
    @builtin(position) position: vec4f,
    // screen position in view coordinates
    @location(0) xy: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
) -> VertexOutput {

    // let pos = ...
    // May only be indexed by a constant where as:
    // var pos = ...
    // May be indexed by a varialble. Now fixed.
    let pos = array<vec2<f32>,6>(
        vec2<f32>( -1.0, -1.0),
        vec2<f32>(  1.0, -1.0),
        vec2<f32>( -1.0,  1.0),

        vec2<f32>( -1.0,  1.0),
        vec2<f32>(  1.0, -1.0),
        vec2<f32>(  1.0,  1.0),
    );

    // might make more sense for this to go from 0 to 1 for easier calcs
    // or could use vertex buffer to load in extent rather than calculate
    let xy = array<vec2<f32>,6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),

        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );


    var out: VertexOutput;
    out.position = vec4f(pos[index], 0.0, 1.0);
    out.xy = xy[index];
    return out;
}

//////////////////////////////////////////////////////////////////////////
//
//  Fragment shader - displays color for each pixel and shape
//
//////////////////////////////////////////////////////////////////////////

// Material could be returned using function argument pointers if
// performance is an issue.
struct Material {
    color: vec4f,
}

struct Result {
    dist: f32,
    aMaterial: Material,
}

const black = Material(vec4f(0.0, 0.0, 0.0, 1.0));      // black
const red =   Material(vec4f(0.2, 0.0, 0.0, 1.0));      // red
const green = Material(vec4f(0.0, 0.2, 0.0, 1.0));      // green
const blue =  Material(vec4f(0.0, 0.0, 0.2, 1.0));      // blue

const background = Result(-1.0, black);

fn calcNormal(pos: vec3f) -> vec3f {
	// Center sample
    let c = theShape(pos).dist;
	// Use offset samples to compute gradient / normal
    var eps_zero: vec2f = vec2f(0.001, 0.0);
    return normalize(vec3f(
        theShape(pos + eps_zero.xyy).dist,
        theShape(pos + eps_zero.yxy).dist,
        theShape(pos + eps_zero.yyx).dist) - c);
}

// get gradient in the world
// const grad_step = 0.02;
// fn calcNormal(pos: vec3f) -> vec3f {
// 	let dx = vec3f( grad_step, 0.0, 0.0 );
// 	let dy = vec3f( 0.0, grad_step, 0.0 );
// 	let dz = vec3f( 0.0, 0.0, grad_step );    
// 	return normalize (
// 		vec3f(
// 			theShape( pos + dx ).dist - theShape( pos - dx ).dist,
// 			theShape( pos + dy ).dist - theShape( pos - dy ).dist,
// 			theShape( pos + dz ).dist - theShape( pos - dz ).dist			
// 		)
// 	);
// }

const maxSteps = 128;
const epsilon = 0.001;

fn ray_march(
    rayOrigin: vec3f,     // camera location
    rayDir: vec3f,     // ray direction
) -> Result {
    var t = 1.0;                // total depth

    for (var i = 0; i < maxSteps; i++) {
        var res = theShape(rayOrigin - rayDir * t);
        if res.dist < epsilon * t { return Result(t, res.aMaterial); }
        t += res.dist;
    }

    return background;
}

const iTime = pi * 0.25;

fn render(rayOrigin: vec3f, rayDir: vec3f) -> vec3f {
    var color: vec3f;
	var t = ray_march(rayOrigin, rayDir);

    // vec3 L = normalize(vec3(sin(iTime)*1.0, cos(iTime*0.5)+0.5, -0.5));
    var L = normalize(vec3(sin(iTime)*1.0, cos(iTime*0.5)+0.5, -0.5));

	if t.dist == -1.0 {
        // color = vec3(0.30, 0.36, 0.60) - rayDir.y * 0.4;
        color = t.aMaterial.color.xyz;
    } else {   
        // vec3 pos = rayOrigin + rayDir * t;
        var pos = rayOrigin + rayDir * t.dist;
        // vec3 N = calcNormal(pos);
        var n = calcNormal(pos);

        // vec3 objectSurfaceColour = vec3(0.4, 0.8, 0.1);
        var objectSurfaceColour = t.aMaterial.color.xyz;
        // L is vector from surface point to light, N is surface normal. N and L must be normalized!
        var NoL = max(dot(n, L), 0.0);
        var LDirectional = vec3f(1.80,1.27,0.99) * NoL;
        var LAmbient = vec3f(0.03, 0.04, 0.1);
        var diffuse = objectSurfaceColour * (LDirectional + LAmbient);
		
        color = diffuse;
        
        
        var shadow = 0.0f;
        var shadowRayOrigin = pos + n * 0.01;
        var shadowRayDir = L;
        t = ray_march(shadowRayOrigin, shadowRayDir);
        if t.dist >= -1.0 { shadow = 1.0; }
        color = mix(color, color*0.8, shadow);
        
        // Visualize normals:
        // color = n * vec3(0.5) + vec3(0.5);
    }
    
    return color;
}

fn getCameraRayDir(uv: vec2f, camPos: vec3f, camTarget: vec3f) -> vec3f
{
	let camForward: vec3f = normalize(camPos - camTarget);
	let camRight: vec3f = normalize(cross(vec3(0.0, 1.0, 0.0), camForward));
	let camUp: vec3f = normalize(cross(camForward, camRight));

    // fPersp controls the camera's field of view. Try changing it!
    let fPersp = 1.0f;
	let vDir: vec3f =
        // normalize(uv.x * camRight + uv.y * camUp + camForward * fPersp);
        normalize(-uv.x * camRight - uv.y * camUp + camForward * fPersp);

	return vDir;
}

const zscreen = 1.0;

fn camera_dir(uv: vec2f, camPos: vec3f, camTarget: vec3f) -> vec3f {
    let xyz = vec3f(uv, zscreen);
    let dir = normalize(camPos - xyz);
    return dir;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var camPos = vec3f(0, 0, 2);
    // var at = vec3f(0, 0, 1);
    var at = vec3f(0, 0, 0);
    
    // vec2 uv = normalizeScreenCoords(fragCoord);
    var rayDir = getCameraRayDir(in.xy, camPos, at);  
    // var rayDir = camera_dir(in.xy, camPos, at);  
    
    var color = render(camPos, rayDir);
    
    color = pow(color, vec3f(0.4545)); // Gamma correction (1.0 / 2.2)
    
    return vec4f(color, 1.0); // Output to screen
}


//////////////////////////////////////////////////////////////////////////
//
//  Shapes and shape operators
//
//////////////////////////////////////////////////////////////////////////

const pi: f32 = 3.14159265359;

// fn theShape(p: vec3f) -> Result { return Result(sphere(p, 1), blue); }
fn theShape(p: vec3f) -> Result { return shape7(p); }

fn shape1(p: vec3f) -> Result {
    let p2 = (translate( 0.25,  0.0,  0.0) * vec4(p, 1.0)).xyz;
    let p1 = (translate(-0.25,  0.0, -0.0) * vec4(p, 1.0)).xyz;
    return unions(
        Result(sphere(p1, 0.5), red),
        Result(sphere(p2, 0.5), green)
    );
}

fn shape7(p: vec3f) -> Result {
    let p1 = (translate(-0.5, -0.5, -0.5) * vec4(p, 1.0)).xyz;
    let p2 = (translate( 0.5,  0.5,  0.5) * vec4(p, 1.0)).xyz;
    return unions(
        Result(sphere(p1, 0.5), red),
        Result(sphere(p2, 0.5), green)
    );
}

fn shape2(p: vec3f) -> Result {
    return intersect(
        Result(box(p, vec3(0.38)), red),
        Result(sphere(p, 0.5), blue)
    );
}

fn shape3(p: vec3f) -> Result {
    let p1 = (rotz(pi / 2.0) * vec4(p, 1.0)).xyz;
    let p2 = (rotx(pi / 2.0) * vec4(p, 1.0)).xyz;
    return Result(
        mind(
            cappedCylinder(p2, 0.4, 0.1),
            mind(
                cappedCylinder(p, 0.4, 0.1),
                cappedCylinder(p1, 0.4, 0.1),
            ),
        ),
        green
    );
}

fn shape4(p: vec3f) -> Result {
    return subtract(shape3(p), shape2(p));
}

fn shape5(p: vec3f) -> Result {
    return Result(box(p, vec3(0.5)), red);
}

fn shape6(p: vec3f) -> Result {
    let rotxy = roty(pi / 4.0) * rotx(pi / 4.0);
    let p1 = (rotxy * vec4(p, 1.0)).xyz;
    return subtract(shape3(p1), shape2(p1));
}

// distance from sphere
fn sphere(p: vec3f, radius: f32) -> f32 {
    return length(p) - radius;
    // return -radius - length(p);
}

// distance from a box
fn box(p: vec3f, b: vec3f) -> f32 {
  let q = abs(p) - b;
  // let zero = vec3(0.0);
  return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

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


fn trans(p: vec3f, m: mat4x4<f32>) -> vec3f {
    return (m * vec4(p, 1.0)).xyz;
}

// struct Result {
//     dist: f32,
//     color: vec4f,
// }

// type MatIndex = i32;

// color is just result
// fn color(dist: f32, material: matIdx) -> Result {
//     return Result(dist, material);
// }

fn recolor(in: Result, material: Material) -> Result {
    return Result(in.dist, material);
}

fn unions(c1: Result, c2: Result) -> Result {
    if c1.dist < c2.dist { return c1; }
    return c2;
}
fn intersect(c1: Result, c2: Result) -> Result {
    if c1.dist > c2.dist { return c1; }
    return c2;
}
fn subtract(c1: Result, c2: Result) -> Result {
    if -c1.dist > c2.dist { return Result(-c1.dist, c1.aMaterial); }
    return c2;
}
fn invert(c1: Result) -> Result { return Result(-c1.dist, c1.aMaterial); }
// fn xord(c1: Result, c2: Result) -> Result {
//     return maxd(mind(c1, c2), negd(maxd(c1, c2)));
// }
fn first(c1: Result, c2: Result) -> Result { return c1; }
fn second(c1: Result, c2: Result) -> Result { return c2; }

// fn unions(d1: f32, d2: f32) -> f32 { return min(d1, d2); }
// fn intersect(d1: f32, d2: f32) -> f32 { return max(d1, d2); }
// fn subtract(d1: f32, d2: f32) -> f32 { return max(-d1, d2); }

// matrix operations - inverted
fn translate(x: f32, y: f32, z: f32) -> mat4x4<f32> {
    return mat4x4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -x,   -y,   -z,   1.0
    );
}

fn rotx(theta: f32) -> mat4x4<f32> {
    let inv_theta = -theta;
    return mat4x4(
        1.0, 0.0,        0.0,         0.0,
        0.0, cos(inv_theta), -sin(inv_theta), 0.0,
        0.0, sin(inv_theta), cos(inv_theta),  0.0,
        0.0, 0.0,        0.0,         1.0
    );
}

fn roty(theta: f32) -> mat4x4<f32> {
    let inv_theta = -theta;
    return mat4x4(
        cos(inv_theta),  0.0, sin(inv_theta), 0.0,
        0.0,         1.0, 0.0,        0.0,
        -sin(inv_theta), 0.0, cos(inv_theta), 0.0,
        0.0,         0.0, 0.0,        1.0
    );
}

fn rotz(theta: f32) -> mat4x4<f32> {
    let inv_theta = -theta;
    return mat4x4(
        cos(inv_theta), -sin(inv_theta), 0.0, 0.0,
        sin(inv_theta), cos(inv_theta),  0.0, 0.0,
        0.0,        0.0,         1.0, 0.0,
        0.0,        0.0,         0.0, 1.0
    );
}


