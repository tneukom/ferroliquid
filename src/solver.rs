use crate::sides::Direction;
use itertools::Itertools;

#[derive(Debug, Default, Clone, Copy)]
pub struct Row {
    // TODO: Make Neighbors<T> { up, left, down, right }
    pub neighbors: [usize; 4],
    pub coeffs: [f64; 4],
    pub diagonal: f64,
}

impl Row {
    /// associated index of the cell up from this one
    pub fn up(&self) -> usize {
        self.neighbors[Direction::Up as usize]
    }

    pub fn left(&self) -> usize {
        self.neighbors[Direction::Left as usize]
    }

    pub fn down(&self) -> usize {
        self.neighbors[Direction::Down as usize]
    }

    pub fn right(&self) -> usize {
        self.neighbors[Direction::Right as usize]
    }
}

pub struct Solver {
    pub precon: Vec<f64>,
    pub e: Vec<f64>,
    pub b: Vec<f64>,
    pub pressure: Vec<f64>,
    pub rows: Vec<Row>,
}

impl Solver {
    #[inline(never)]
    pub fn new(len: usize) -> Self {
        Self {
            precon: vec![0.0; len],
            e: vec![0.0; len],
            b: vec![0.0; len],
            pressure: vec![0.0; len],
            rows: vec![Row::default(); len],
        }
    }

    #[inline(never)]
    pub fn apply(&self, x: &[f64], b: &mut [f64]) {
        for i in 0..x.len() {
            let row = &self.rows[i];

            b[i] = row.diagonal * x[i]
                + row.coeffs[Direction::Up as usize] * x[row.up()]
                + row.coeffs[Direction::Left as usize] * x[row.left()]
                + row.coeffs[Direction::Down as usize] * x[row.down()]
                + row.coeffs[Direction::Right as usize] * x[row.right()];
        }
    }

    #[inline(never)]
    pub fn calc_preconditioner(&mut self) {
        for i in 0..self.rows.len() {
            let row = &self.rows[i];

            let top_row = &self.rows[row.up()];
            let left_row = &self.rows[row.left()];
            let tau = 0.97;
            let t1 = row.coeffs[Direction::Up as usize] * self.precon[row.up()];
            let t2 = row.coeffs[Direction::Left as usize] * self.precon[row.left()];
            let s1 = row.coeffs[Direction::Up as usize]
                * top_row.coeffs[Direction::Right as usize]
                * self.precon[row.up()]
                * self.precon[row.up()];
            let s2 = row.coeffs[Direction::Left as usize]
                * left_row.coeffs[Direction::Down as usize]
                * self.precon[row.left()]
                * self.precon[row.left()];
            self.e[i] = row.diagonal - t1 * t1 - t2 * t2 - tau * (s1 + s2);
            if self.e[i].abs() < 0.25 * row.diagonal {
                self.e[i] = row.diagonal;
            }

            self.precon[i] = 1.0 / self.e[i].sqrt();
        }
    }

    #[inline(never)]
    pub fn apply_preconditioner(&self, r: &[f64], z: &mut [f64]) {
        let mut q = vec![0.0; r.len()];

        //L = FE^-1 + E
        //Lq = r
        for i in 0..self.rows.len() {
            let row = &self.rows[i];

            let t = r[i]
                - row.coeffs[Direction::Up as usize] * self.precon[row.up()] * q[row.up()]
                - row.coeffs[Direction::Left as usize] * self.precon[row.left()] * q[row.left()];
            q[i] = t * self.precon[i];
        }

        //LTz = q
        for i in (0..self.rows.len()).rev() {
            let row = &self.rows[i];

            let t = q[i]
                - row.coeffs[Direction::Down as usize] * self.precon[i] * z[row.down()]
                - row.coeffs[Direction::Right as usize] * self.precon[i] * z[row.right()];
            z[i] = t * self.precon[i];
        }
    }

    /// TODO: Cleanup commented code
    #[inline(never)]
    pub fn solve_with_preconditioner(&mut self, iterations: usize, stop_epsilon: f64) -> usize {
        let mut z = vec![0.0; self.pressure.len()];
        let mut r = vec![0.0; self.pressure.len()];
        let mut s = vec![0.0; self.pressure.len()];
        let mut temp = vec![0.0; self.pressure.len()];
        let n = self.pressure.len();

        // self.pressure.fill(0.0);

        // TODO: What's this for
        self.apply(&self.pressure, &mut temp);
        // r = b - temp;
        for i in 0..n {
            r[i] = self.b[i] - temp[i];
        }

        if inf_norm(&r) < stop_epsilon {
            return 0;
        }

        self.apply_preconditioner(&r, &mut z);

        // s = z;
        for i in 0..n {
            s[i] = z[i];
        }

        let mut sigma = dot(&z, &r);

        for k in 0..iterations {
            self.apply(&s, &mut z);
            let alpha = sigma / dot(&z, &s);

            // pressure += alpha * s;
            for i in 0..n {
                self.pressure[i] += alpha * s[i];
            }

            // r -= alpha * z;
            for i in 0..n {
                r[i] -= alpha * z[i];
            }

            let error_norm = inf_norm(&r);
            if error_norm < stop_epsilon {
                println!("Solved with error_norm: {error_norm}");
                return k;
            }

            self.apply_preconditioner(&r, &mut z);
            let sigma_next = dot(&z, &r);
            let beta = sigma_next / sigma;
            // s = z + beta * s;
            for i in 0..n {
                s[i] = z[i] + beta * s[i];
            }

            sigma = sigma_next;
        }

        println!("failed to reach stop_epsilon!");
        iterations
    }

    //     unsigned int Solver::solve_with_preconditioner(unsigned int iterations, REAL stopEpsilon, REAL worstEpsilonAcceptable) {

    //
    //

    //
    //         //cout << r.lInfNorm() << endl;
    //         //std::cout << "solver error: " << errorNorm << std::endl;
    // //        printf("solver Error %1.5f \n", errorNorm);
    // //        printf("solver iterations %d \n", k);
    //         //if(std::isnan(errorNorm)) {
    //         //    throw NoConvergence(errorNorm);
    //         //}
    //
    //         //if(errorNorm > worstEpsilonAcceptable && FAIL_IF_NO_CONVERGENCE) {
    //         //    throw NoConvergence(errorNorm);
    //         //}
    //         //gassertc(errorNorm < worstEpsilonAcceptable, "Solver doesn't converge");
    //
    //         return k;
    //     }
}

fn inf_norm(xs: &[f64]) -> f64 {
    xs.iter().map(|x| x.abs()).reduce(f64::max).unwrap()
}

fn dot(xs: &[f64], ys: &[f64]) -> f64 {
    xs.iter().zip_eq(ys).map(|(&x, &y)| x * y).sum()
}
