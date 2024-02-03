use std::{collections::{HashMap, VecDeque}, ops::{Deref, DerefMut}};

use minilp::{ComparisonOp, OptimizationDirection, Problem, Solution, Variable};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mat {
    Right,
    Up,
    Left,
    Down,
    None
}
pub fn lay_mats<T: PartialEq>(coloring: &[Vec<T>], w: usize, h: usize) -> Option<Vec<Vec<Mat>>> {
    let wm = w.max(1) - 1;
    let hm = h.max(1) - 1;
    let mut model = MixedProblem::new(OptimizationDirection::Minimize);
    let mut vars = vec![vec![[None; 5]; w]; h];
    for i in 0..h {
        for j in 0..w {
            if i + 1 != h && coloring[i + 1][j] == coloring[i][j] {
                vars[i][j][3] = Some(model.add_int(0., (0., 1.)));
                vars[i + 1][j][1] = Some(model.add_int(0., (0., 1.)));
                model.add_constraint([(vars[i][j][3].unwrap(), 1.), (vars[i + 1][j][1].unwrap(), -1.)], ComparisonOp::Eq, 0.);
            }
            if j + 1 != w && coloring[i][j + 1] == coloring[i][j] {
                vars[i][j][0] = Some(model.add_int(0., (0., 1.)));
                vars[i][j + 1][2] = Some(model.add_int(0., (0., 1.)));
                model.add_constraint([(vars[i][j][0].unwrap(), 1.), (vars[i][j + 1][2].unwrap(), -1.)], ComparisonOp::Eq, 0.);
            }
            let coeff = (j*j+i*i+wm*wm+hm*hm-j*wm-i*hm) as f64 / (wm*wm+hm*hm) as f64 * 3.6 - 1.7;
            vars[i][j][4] = Some(model.add_int(coeff, (0., 1.)));
            model.add_constraint(vars[i][j].iter().filter_map(Option::clone).map(|x| (x, 1.)), ComparisonOp::Eq, 1.);
        }
    }
    for i in 0..hm {
        for j in 0..wm {
            model.add_constraint([vars[i][j][0], vars[i][j][3], vars[i + 1][j + 1][1], vars[i + 1][j + 1][2]].into_iter().flatten().map(|x| (x, 1.)), ComparisonOp::Ge, 1.)
        }
    }
    let solution = model.solve(1e-4)?;
    Some(vars.into_iter().map(|x| x.into_iter().map(|x| {
        match x.into_iter().enumerate().find(|(_, x)| x.map_or(false, |x| {
            solution[x] >= 0.9
        })).unwrap().0 {
            0 => Mat::Right,
            1 => Mat::Up,
            2 => Mat::Left,
            3 => Mat::Down,
            4 => Mat::None,
            _ => panic!()
        }
    }).collect::<Vec<_>>()).collect::<Vec<_>>())
    
}
pub struct MixedProblem {
    vars: HashMap<Variable, bool>,
    problem: Problem,
    direction: OptimizationDirection
}
impl Deref for MixedProblem {
    type Target = Problem;
    fn deref(&self) -> &Self::Target {
        &self.problem
    }
}
impl DerefMut for MixedProblem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.problem
    }
}
impl MixedProblem {
    pub fn new(direction: OptimizationDirection) -> Self {
        MixedProblem {direction, problem: Problem::new(direction), vars: HashMap::new()}
    }
    pub fn add_col(&mut self, obj_coeff: f64, range: (f64, f64), int: bool) -> Variable {
        let v = self.problem.add_var(obj_coeff, range);
        self.vars.insert(v, int);
        v
    }
    pub fn add_var(&mut self, obj_coeff: f64, range: (f64, f64)) -> Variable {
        self.add_col(obj_coeff, range, false)
    }
    pub fn add_int(&mut self, obj_coeff: f64, range: (f64, f64)) -> Variable {
        self.add_col(obj_coeff, range, true)
    }
    pub fn solve(&self, eps: f64) -> Option<Solution> {
        let mut solutions = Vec::new();
        let mut problems = VecDeque::from([self.problem.solve().ok()?]);
        while let Some(solution) = problems.pop_front() {
            let Some((var, _)) = self.vars.iter().find(|(k, v)| {
                **v && 0.5 - (solution[**k].fract().abs() - 0.5).abs() > eps
            }) else {solutions.push(solution); continue;};
            let coeff = solution[*var];
            problems.extend(solution.clone().add_constraint([(*var, 1.)], ComparisonOp::Ge, coeff.ceil()));
            problems.extend(solution.add_constraint([(*var, 1.)], ComparisonOp::Le, coeff.floor()));
        }
        
        if let OptimizationDirection::Maximize = self.direction {
            solutions.into_iter().max_by(|x, y| x.objective().total_cmp(&y.objective()))
        } else {
            solutions.into_iter().min_by(|x, y| x.objective().total_cmp(&y.objective()))
        }
    }
}