#![cfg_attr(feature = "nightly", feature(test))]
#![cfg(feature = "nightly")]
extern crate test;

extern crate korome;

#[cfg(all(feature = "nightly", test))]
mod tests {
    use test::{Bencher, black_box};
    use super::*;

    const M1: f32 = 10.;
    const M2: f32 = 20.;

    const V1: Vect = Vector2(10., 10.);
    const V2: Vect = Vector2(0., 10.);

    const P: Vect = Vector2(10., 0.);

    #[bench]
    fn bench_normal(bench: &mut Bencher){
        bench
            .iter(|| black_box(
                calc_velocity_after_collision(M1, M2, V1, V2, P)
            ))
    }

    #[bench]
    fn bench_alt(bench: &mut Bencher){
        bench
            .iter(|| black_box(
                call_alt(M1, M2, V1, V2, P.direction())
            ))
    }
    #[bench]
    fn bench_polar(bench: &mut Bencher){
        let a = (V1.length(), V2.length(), V1.direction(), V2.direction(), P.direction());
        bench
            .iter(|| black_box(
                calc_velocity_after_collision_alt(M1, M2, a.0, a.1, a.2, a.3, a.4)
            ))
    }
    #[bench]
    fn bench_cartesian(bench: &mut Bencher){
        let a = (V1.length(), V2.length(), V1.direction(), V2.direction());
        bench
            .iter(|| black_box(
                calc_velocity_after_collision(M1, M2, a.0 * Vector2::unit_vector(a.2), a.1 * Vector2::unit_vector(a.3), P)
            ))
    }
}

pub use korome::Vector2;

pub type Vect = Vector2<f32>;

pub fn calc_velocity_after_collision(m1: f32, m2: f32, v1: Vect, v2: Vect, dist: Vect) -> Vect{
    v1 - (2. * m2)/(m1 + m2) * (v1 - v2).dot(dist) / dist.length().powi(2) * dist
}

use std::f32::consts::FRAC_PI_2;

#[inline]
pub fn call_alt(m1: f32, m2: f32, v1: Vect, v2: Vect, dir: f32) -> Vect{
    calc_velocity_after_collision_alt(m1, m2, v1.length(), v2.length(), v1.direction(), v2.direction(), dir)
}

pub fn calc_velocity_after_collision_alt(m1: f32, m2: f32, v1: f32, v2: f32, a1: f32, a2: f32, phi: f32) -> Vect{
    let (r1_sin, r1_cos) = (a1 - phi).sin_cos();
    let r2_cos = (a2 - phi).cos();

    let sum1 = (v1 * r1_cos * (m1-m2) + 2. * m2 * v2 * r2_cos) / (m1 + m2);
    let sum2 = v1 * r1_sin;

    let (sin1, cos1) = phi.sin_cos();
    let (sin2, cos2) = (phi + FRAC_PI_2).sin_cos();

    Vector2(
        sum1 * cos1 + sum2 * cos2,
        sum1 * sin1 + sum2 * sin2
    )
}

use std::io::{Write, stdin, stdout};

fn main() {
    let (m1, m2, v1, v2, dist);

    m1 = ask_for_value("Mass 1");
    m2 = ask_for_value("Mass 2");
    v1 = Vector2(ask_for_value("v1.x"), ask_for_value("v1.y"));
    v2 = Vector2(ask_for_value("v2.x"), ask_for_value("v2.y"));
    dist = Vector2::<f32>(ask_for_value("p1.x"), ask_for_value("p1.y")) - Vector2(ask_for_value("p2.x"), ask_for_value("p2.y"));

    println!("= {:?}", calc_velocity_after_collision(m1, m2, v1, v2, dist));
    println!("= {:?}", call_alt(m1, m2, v1, v2, dist.direction()));
}

use std::str::FromStr;
use std::fmt::Debug;

fn ask_for_value<T: FromStr>(prompt: &str) -> T
where <T as FromStr>::Err: Debug {
    let mut line = String::new();

    print!("{}: ", prompt);stdout().flush().unwrap();
    stdin().read_line(&mut line).unwrap();
    line.trim().parse::<T>().unwrap()
}
