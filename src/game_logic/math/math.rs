use std::f32::consts::E;

use bevy::prelude::Vec2;

#[inline]
pub fn sigmoid(x: f32) -> f32 {
    1. / (1. + E.powf(-x))
}

#[inline]
pub fn sigmoid_inplace(x: &mut f32) {
    *x = sigmoid(*x);
}

#[inline]
pub fn tanh_inplace(x: &mut f32) {
    *x = f32::tanh(*x);
}

pub fn nearest_intersection(c: Vec2, r: f32, m: Vec2, n: Vec2) -> Option<Vec2> {
    if c.length_squared() <= r*r {
        Some(Vec2::new(0.,0.))
    }
    else if c.dot(m) < 0. {
        //nearest point is intersecting the m line
        //m.y\left(c.xm.y-c.ym.x+\sqrt{r^{2}-\left(c.xm.x+c.ym.y\right)^{2}}\right)
        //m.x\left(-c.xm.y+c.ym.x-\sqrt{r^{2}-\left(c.xm.x+c.ym.y\right)^{2}}\right)
        line_intersection(c, r, m, 1.)
    }
    else if c.dot(n) < 0. {
        //nearest point is intersecting the n line
        //n.y\left(c.xn.y-c.yn.x-\sqrt{r^{2}-\left(c.xn.x+c.yn.y\right)^{2}}\right)
        //n.x\left(-c.xn.y+c.yn.x+\sqrt{r^{2}-\left(c.xn.x+c.yn.y\right)^{2}}\right)
        line_intersection(c, r, n, -1.)
    }
    else {
        //nearest point is on the straight line to center
        //c.x\left(1-\frac{r}{\sqrt{c.x^{2}+c.y^{2}}}\right)
        //c.y\left(1-\frac{r}{\sqrt{c.x^{2}+c.y^{2}}}\right)
        let q = 1. - r/c.length();
        Some(c * q)
    }
}

#[inline]
fn line_intersection(c: Vec2, r: f32, v: Vec2, sign: f32) -> Option<Vec2>{
    let d = c.dot(v);
    let d = r*r - d*d;
    if d < 0. {
        None?
    }
    let q = c.x*v.y - c.y*v.x + sign * d.sqrt();
    Some(Vec2::new(v.y*q, -v.x*q))
}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const EPSILON: f32 = 0.0001;
    fn approx_eq<T: 
        std::ops::Add<f32> + 
        std::ops::Sub<f32> + 
        std::cmp::PartialOrd<<T as std::ops::Add<f32>>::Output> + 
        std::cmp::PartialOrd<<T as std::ops::Sub<f32>>::Output> +
        Copy>(a:T, b:T) -> bool {
        a >= b - EPSILON && a <= b + EPSILON
    }
    fn approx_eq_vec(a: Vec2, b: Vec2) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }
    fn approx_eq_some_vec(a: Option<Vec2>, b: Option<Vec2>) -> bool {
        println!("{:?} ? {:?}", a, b);
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => approx_eq_vec(a, b),
            _ => false,
        }
    }

    #[test]
    fn test_nearby_ahead() {
        let c = Vec2::new(0.7, 0.82);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert_eq!(nearest_intersection(c, r, m, n), Some(Vec2::new(0., 0.)));
    }

    #[test]
    fn test_nearby_left() {
        let c = Vec2::new(-0.17, 1.3);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert_eq!(nearest_intersection(c, r, m, n), Some(Vec2::new(0., 0.)));
    }

    #[test]
    fn test_nearby_right() {
        let c = Vec2::new(1.34, -0.2);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert_eq!(nearest_intersection(c, r, m, n), Some(Vec2::new(0., 0.)));
    }

    #[test]
    fn test_nearby_behind() {
        let c = Vec2::new(-0.8, -0.82);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert_eq!(nearest_intersection(c, r, m, n), Some(Vec2::new(0., 0.)));
    }

    #[test]
    fn test_ahead_intersecting() {
        let c = Vec2::new(2.67, 2.72);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), Some(Vec2::new(1.26896558166, 1.29272898207))));
    }

    #[test]
    fn test_ahead_nonintersecting() {
        let c = Vec2::new(4.34, 4.14);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), Some(Vec2::new(2.89283477944, 2.75952442094))));
    }

    #[test]
    fn test_left() {
        let c = Vec2::new(0.11, 3.6);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), Some(Vec2::new(0.645876556346, 1.6731278412))));
    }

    #[test]
    fn test_right() {
        let c = Vec2::new(4.71, 0.33);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), Some(Vec2::new(2.91658284465, 1.21524285194))));
    }

    #[test]
    fn test_left_none() {
        let c = Vec2::new(-0.45, 5.28);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), None));
    }

    #[test]
    fn test_right_none() {
        let c = Vec2::new(4.98, -0.85);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), None));
    }

    #[test]
    fn test_right_none_behind() {
        let c = Vec2::new(0.64, -4.65);
        let r = 2.;
        let m = Vec2::new(2.72, -1.05).normalize();
        let n = Vec2::new(-0.9, 2.16).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), None));
    }

    #[test]
    fn test_rotated_ahead() {
        let c = Vec2::new(-6.87,-0.7);
        let r = 2.;
        let m = Vec2::new(-0.84, 2.25).normalize();
        let n = Vec2::new(-0.56, -3.41).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), Some(Vec2::new(-4.88030189088, -0.497265112608))));
    }

    #[test]
    fn test_rotated_left() {
        let c = Vec2::new(-3.61, -2.54);
        let r = 2.3;
        let m = Vec2::new(-0.84, 2.25).normalize();
        let n = Vec2::new(-0.56, -3.41).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), Some(Vec2::new(-2.11707822971, -0.790375872425))));
    }

    #[test]
    fn test_rotated_right() {
        let c = Vec2::new(-4.7, 2.05);
        let r = 2.3;
        let m = Vec2::new(-0.84, 2.25).normalize();
        let n = Vec2::new(-0.56, -3.41).normalize();
        assert!(approx_eq_some_vec(nearest_intersection(c, r, m, n), Some(Vec2::new(-3.00647426196, 0.493731843607))));
    }
}