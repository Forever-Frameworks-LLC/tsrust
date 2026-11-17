//! Number-heavy phases: integer arrays, float records, nested arrays and calls.
use std::f64::consts::PI;

pub fn sieve(limit: usize) -> i64 {
    let mut composite = vec![false; limit + 1];
    let (mut count, mut sum) = (0i64, 0i64);
    for candidate in 2..=limit {
        if composite[candidate] {
            continue;
        }
        count += 1;
        sum = (sum + candidate as i64) % 1_000_000_007;
        let mut multiple = candidate * candidate;
        while multiple <= limit {
            composite[multiple] = true;
            multiple += candidate;
        }
    }
    count * 1_000_000_007 + sum
}

#[derive(Clone, Copy)]
struct Body {
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    mass: f64,
}

const SOLAR_MASS: f64 = 4.0 * PI * PI;
const DAYS_PER_YEAR: f64 = 365.24;

fn planets() -> Vec<Body> {
    let body = |x, y, z, vx: f64, vy: f64, vz: f64, mass: f64| Body {x, y, z, vx: vx * DAYS_PER_YEAR, vy: vy * DAYS_PER_YEAR, vz: vz * DAYS_PER_YEAR, mass: mass * SOLAR_MASS};
    vec![
        Body {x: 0.0, y: 0.0, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, mass: SOLAR_MASS},
        body(4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01, 1.66007664274403694e-03, 7.69901118419740425e-03, -6.90460016972063023e-05, 9.54791938424326609e-04),
        body(8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01, -2.76742510726862411e-03, 4.99852801234917238e-03, 2.30417297573763929e-05, 2.85885980666130812e-04),
        body(1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01, 2.96460137564761618e-03, 2.37847173959480950e-03, -2.96589568540237556e-05, 4.36624404335156298e-05),
        body(1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01, 2.68067772490389322e-03, 1.62824170038242295e-03, -9.51592254519715870e-05, 5.15138902046611451e-05),
    ]
}

fn advance(bodies: &mut [Body], step: f64) {
    let count = bodies.len();
    for i in 0..count {
        for j in i + 1..count {
            let dx = bodies[i].x - bodies[j].x;
            let dy = bodies[i].y - bodies[j].y;
            let dz = bodies[i].z - bodies[j].z;
            let distance2 = dx * dx + dy * dy + dz * dz;
            let magnitude = step / (distance2 * distance2.sqrt());
            let (mass_i, mass_j) = (bodies[i].mass, bodies[j].mass);
            bodies[i].vx -= dx * mass_j * magnitude;
            bodies[i].vy -= dy * mass_j * magnitude;
            bodies[i].vz -= dz * mass_j * magnitude;
            bodies[j].vx += dx * mass_i * magnitude;
            bodies[j].vy += dy * mass_i * magnitude;
            bodies[j].vz += dz * mass_i * magnitude;
        }
    }
    for body in bodies.iter_mut() {
        body.x += step * body.vx;
        body.y += step * body.vy;
        body.z += step * body.vz;
    }
}

fn energy(bodies: &[Body]) -> f64 {
    let mut total = 0.0;
    for (i, body) in bodies.iter().enumerate() {
        total += 0.5 * body.mass * (body.vx * body.vx + body.vy * body.vy + body.vz * body.vz);
        for other in &bodies[i + 1..] {
            let (dx, dy, dz) = (body.x - other.x, body.y - other.y, body.z - other.z);
            total -= body.mass * other.mass / (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    total
}

pub fn nbody(steps: usize) -> f64 {
    let mut bodies = planets();
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for body in &bodies {
        px += body.vx * body.mass;
        py += body.vy * body.mass;
        pz += body.vz * body.mass;
    }
    bodies[0].vx = -px / SOLAR_MASS;
    bodies[0].vy = -py / SOLAR_MASS;
    bodies[0].vz = -pz / SOLAR_MASS;
    for _ in 0..steps {
        advance(&mut bodies, 0.01);
    }
    energy(&bodies)
}

pub fn matrix(size: usize) -> f64 {
    let a: Vec<Vec<f64>> = (0..size).map(|row| (0..size).map(|column| ((row * size + column) % 17) as f64 / 7.0).collect()).collect();
    let b: Vec<Vec<f64>> = (0..size).map(|row| (0..size).map(|column| ((row + column * 3) % 13) as f64 / 5.0).collect()).collect();
    let mut trace = 0.0;
    for row in 0..size {
        for column in 0..size {
            let mut sum = 0.0;
            for k in 0..size {
                sum += a[row][k] * b[k][column];
            }
            if row == column {
                trace += sum;
            }
        }
    }
    trace
}

pub fn fib(n: i64) -> i64 {
    if n < 2 {n} else {fib(n - 1) + fib(n - 2)}
}
