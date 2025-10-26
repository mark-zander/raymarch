// // Vertex shader

// struct VertexOutput {
//     @builtin(position) clip_position: vec4f,
// };

// @vertex
// fn vs_main(
//     @builtin(vertex_index) in_vertex_index: u32,
// ) -> VertexOutput {
//     var out: VertexOutput;
//     let x = f32(1 - i32(in_vertex_index)) * 0.5;
//     let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
//     out.clip_position = vec4f(x, y, 0.0, 1.0);
//     return out;
// }

// // Fragment shader

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4f {
//     return vec4f(0.3, 0.2, 0.1, 1.0);
// }

// Vertex shader

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



@fragment
fn fs_fill(in: VertexOutput) -> @location(0) vec4f {
    let x = in.xy.x;
    let y = in.xy.y;

    var r = abs(x);
    var g = 0.0;
    var b = 0.0;
    if y > 0 {
        g = y;
    } else {
        b = -y;
    }

    return vec4f(r, g, b, 1.0);
}

// @fragment
// fn fs_sdf(in: VertexOutput) -> @location(0) vec4f {
//     let epsilon = 6.0E-8;
//     var dnext: Result = Result(0.0, vec4f(0.0));
//     var hit: vec3f = vec3f(0.0, 0.0, 0.0);
//     var color: vec4f = vec4f(0.0, 0.0, 0.0, 0.0);
//     var count: u32 = 0u;
//     for (var z: f32 = 1.0; z >= -1.0; z -= dnext.dist) {
//         dnext = shape6(vec3f(in.xy, z));
//         if abs(dnext.dist) <= epsilon {
//             // z -= znext;
//             hit = vec3f(in.xy, z);
//             // color = vec4f((z + 1.0) / 2.0, 1.0, 1.0, 1.0);
//             color = dnext.color; // + (z + 1.0) / 2.0;
//             break;
//         }
//         count++;
//     }
//     return color;
// }

// alias ptrMaterial = ptr<private, Material>;

// struct Result {
//     dist: f32,
//     aMaterial: ptr<private, Material>,
// }

// var<private> black =   Material(vec4f(0.0, 0.0, 0.0, 1.0));    // black
// var<private> red =     Material(vec4f(1.0, 0.0, 0.0, 1.0));    // red
// var<private> green =   Material(vec4f(0.0, 1.0, 0.0, 1.0));    // green
// var<private> blue =    Material(vec4f(0.0, 0.0, 1.0, 1.0));    // blue

// const black = 0;
// const red = 1;
// const green = 2;
// const blue = 3;

// Material could be returned using function argument pointers if
// performance is an issue.
struct Material {
    color: vec4f,
}

struct Result {
    dist: f32,
    aMaterial: Material,
}

const black = Material(vec4f(0.0, 0.0, 0.0, 1.0));    // black
const red = Material(vec4f(1.0, 0.0, 0.0, 1.0));    // red
const green = Material(vec4f(0.0, 1.0, 0.0, 1.0));    // green
const blue = Material(vec4f(0.0, 0.0, 1.0, 1.0));    // blue

// get gradient in the world
const grad_step = 0.02;
fn gradient(pos: vec3f) -> vec3f {
	let dx = vec3f( grad_step, 0.0, 0.0 );
	let dy = vec3f( 0.0, grad_step, 0.0 );
	let dz = vec3f( 0.0, 0.0, grad_step );    
	return normalize (
		vec3f(
			theshape( pos + dx ).dist - theshape( pos - dx ).dist,
			theshape( pos + dy ).dist - theshape( pos - dy ).dist,
			theshape( pos + dz ).dist - theshape( pos - dz ).dist			
		)
	);
}

fn fresnel(F0: vec3f, h: vec3f, l: vec3f) -> vec3f {
	return F0 + ( 1.0 - F0 ) * pow( clamp( 1.0 - dot( h, l ), 0.0, 1.0 ), 5.0 );
}

// phong shading
fn shading(v: vec3f, n: vec3f, dir: vec3f, eye: vec3f) -> vec3f {
	// ...add lights here...
	
	let shininess = 16.0;
	
	var fin = vec3( 0.0 );
	
	let refl = reflect( dir, n );
    
    let Ks = vec3( 0.5 );
    let Kd = vec3( 1.0 );
	
	// light 0
	{
		let light_pos   = vec3( 20.0, 20.0, 20.0 );
		let light_color = vec3( 1.0, 0.7, 0.7 );
	
		let vl = normalize( light_pos - v );
	
		let diffuse  = Kd * vec3( max( 0.0, dot( vl, n ) ) );
		var specular = vec3( max( 0.0, dot( vl, refl ) ) );
		
        let F = fresnel( Ks, normalize( vl - dir ), vl );
		specular = pow( specular, vec3( shininess ) );
		
		fin += light_color * mix( diffuse, specular, F ); 
	}
    // fin += texture( iChannel0, ref ).rgb * fresnel( Ks, n, -dir );
    
	return fin;
}

fn phong(v: vec3f, n: vec3f, dir: vec3f, eye: vec3f) -> vec3f {
    let light_pos   = vec3( 20.0, 20.0, 20.0 );
    let light_color = vec3( 1.0, 0.7, 0.7 );
    return light_pos;
}

// fragment shader, perspective signed distance function
const epsilon = 0.001;
const zfar = -1.0;      // clip range for z
const znear = 1.0;

const background = Result(-1.0, black);

// fn ray_march(in: VertexOutput) -> vec4f {
fn ray_march(
    xyz: vec3f,     // virtual screen position
    eye: vec3f,     // camera location
    dir: vec3f,     // ray direction
) -> Result {
    let diff = eye - xyz;
    var t = length(diff);       // total depth
    // var t = 0.0;
    var dt = 0.0;               // change in depth
    var d = 0.0;

    for (var i = 0; i < 128; i++) {
        let v : vec3f = eye - t * dir;
        if v.z < zfar { break; }  // past clip range
        d = theshape(v).dist;
        if d < epsilon { break; }
        dt = min(abs(d), 0.1);
        t += dt;
    }

    if d >= epsilon { return background; }

    t -= dt;
    var s: Result;
    for (var i = 0; i < 4; i++) {
        dt *= 0.5;
        let v = eye - dir * (t + dt);
        s = theshape(v);
        if s.dist >= epsilon {
            t += dt;
        }
    }


    return Result(t, s.aMaterial);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let xyz: vec3f = vec3f(in.xy, znear);
    let eye = vec3f(0.0, 0.0, 2.0);
    let dir = normalize(eye - xyz);
    let result = ray_march(xyz, eye, dir);
    // let result = ray_march(eye, dir);

    var pos = eye - result.dist * dir;
    var n = normalize(gradient(pos));
    var col: vec3f;
    // let color = shading(pos, n, dir, eye);
    if result.dist == -1.0 {
        col = vec3f(0.30, 0.36, 0.60) - dir.y * 0.4;
    } else {
        let shapeColor = vec3f(0.4, 0.8, 0.1);
        let l = normalize(vec3(1.0, 0.5, -0.5));
        // L is vector from surface point to light, N is surface normal. N and L must be normalized!
        let nol = max(dot(n, l), 0.0);
        let lDir = vec3(1.80,1.27,0.99) * nol;
        let lAmbient = vec3(0.03, 0.04, 0.1);
        let diffuse = shapeColor * (lDir + lAmbient);
		
        col = diffuse;
        
        var shadow = 0.0;
        let shadowOrigin = pos + n * 0.01;
        let shadowDir = l;
        let t = ray_march(shadowOrigin, shadowOrigin, shadowDir).dist;
        if t >= -1.0 { shadow = 1.0; }
        col = mix(col, col*0.8, shadow);    }

    // let color = n * vec3(0.5) + vec3(0.5);
    return vec4(pow(col, vec3(1.0/1.2)), 1.0);
}

//  Ray direction
//  znear = +1
//  zfar = -1
// fn ray_dir(eye: vec3f, fov: f32, aspect: f32) -> vec3f {

// }

const pi: f32 = 3.14159265359;

fn theshape(p: vec3f) -> Result { return shape7(p); }

fn shape1(p: vec3f) -> Result {
    let p2 = (translate( 0.25,  0.0,  0.0) * vec4(p, 1.0)).xyz;
    let p1 = (translate(-0.25,  0.0, -0.0) * vec4(p, 1.0)).xyz;
    return unions(
        Result(sphere(p1, 0.5), red),
        Result(sphere(p2, 0.5), green)
    );
}

fn shape7(p: vec3f) -> Result {
    let p2 = (translate( 0.5,  0.0,  0.5) * vec4(p, 1.0)).xyz;
    let p1 = (translate(-0.5,  0.0, -0.5) * vec4(p, 1.0)).xyz;
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

// matrix operations
fn translate(x: f32, y: f32, z: f32) -> mat4x4<f32> {
    return mat4x4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        x,   y,   z,   1.0
    );
}

fn rotx(theta: f32) -> mat4x4<f32> {
    return mat4x4(
        1.0, 0.0,        0.0,         0.0,
        0.0, cos(theta), -sin(theta), 0.0,
        0.0, sin(theta), cos(theta),  0.0,
        0.0, 0.0,        0.0,         1.0
    );
}

fn roty(theta: f32) -> mat4x4<f32> {
    return mat4x4(
        cos(theta),  0.0, sin(theta), 0.0,
        0.0,         1.0, 0.0,        0.0,
        -sin(theta), 0.0, cos(theta), 0.0,
        0.0,         0.0, 0.0,        1.0
    );
}

fn rotz(theta: f32) -> mat4x4<f32> {
    return mat4x4(
        cos(theta), -sin(theta), 0.0, 0.0,
        sin(theta), cos(theta),  0.0, 0.0,
        0.0,        0.0,         1.0, 0.0,
        0.0,        0.0,         0.0, 1.0
    );
}

