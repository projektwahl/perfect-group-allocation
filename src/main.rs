use good_lp::{
    default_solver, solvers::ObjectiveDirection::Maximisation, variable, Expression, ModelWithSOS1,
    ProblemVariables, Solution, SolverModel,
};

mod api;

fn main() {
    println!("Hello, world!");

    let mut variables = ProblemVariables::new();

    let test = variables.add(variable().name("test").binary());
    let vars = variables.add_vector(variable().name("awesome").binary(), 100);
    let objective: Expression = vars.iter().sum();

    println!("{}", variables.display(&objective));

    let problem = variables.optimise(Maximisation, objective);

    let solution = problem
        .using(default_solver)
        .with(test << 3)
        .solve()
        .unwrap();

    println!("{}", solution.value(test));
}
