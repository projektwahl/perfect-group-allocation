use good_lp::{
    default_solver, solvers::ObjectiveDirection::Maximisation, variable, Expression, ModelWithSOS1,
    ProblemVariables, Solution, SolverModel,
};

use crate::examples::workshops::{Requirement, Room, Timeslot, Workshop, WorkshopTopic};

pub mod examples;

fn main() {
    println!("Hello, world!");

    let mut variables = ProblemVariables::new();

    let requirement_outside = Requirement {
        identifier: "Outside".to_string(),
    };
    let requirement_computer_pool = Requirement {
        identifier: "Computer-Pool".to_string(),
    };
    let requirements: Vec<Requirement> = vec![requirement_outside, requirement_computer_pool];

    let rooms: Vec<Room> = vec![
        Room {
            identifier: "C-Pool".to_string(),
            requirements: vec![&requirement_computer_pool],
        },
        Room {
            identifier: "Bosch".to_string(),
            requirements: vec![],
        },
        Room {
            identifier: "draussen".to_string(),
            requirements: vec![&requirement_outside],
        },
    ];

    let timeslots: Vec<Timeslot> = vec![
        Timeslot {
            identifier: "morgens".to_string(),
        },
        Timeslot {
            identifier: "mittags".to_string(),
        },
        Timeslot {
            identifier: "abends".to_string(),
        },
    ];

    let workshop_topics: Vec<WorkshopTopic> = vec![WorkshopTopic {
        identifier: "linux-lernen".to_string(),
        requirements: vec![&requirement_computer_pool],
    }];

    let workshops: Vec<Workshop> = vec![Workshop {
        topic: &workshop_topics[0],
        timeslot: &timeslots[0],
    }];

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
