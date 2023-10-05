use good_lp::{
    default_solver, solvers::ObjectiveDirection::Maximisation, variable, Expression, ModelWithSOS1,
    ProblemVariables, Solution, SolverModel,
};

use crate::examples::workshops::{
    Participant, Preference, Rank, Requirement, Room, RoomInTimeSlot, RoomSize, Timeslot, Workshop,
    WorkshopTopic, WorkshopTopicSize,
};

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
    let requirements: Vec<&Requirement> = vec![&requirement_outside, &requirement_computer_pool];

    let timeslot_morgens = Timeslot {
        identifier: "morgens".to_string(),
    };
    let timeslot_mittags = Timeslot {
        identifier: "mittags".to_string(),
    };
    let timeslot_abends = Timeslot {
        identifier: "abends".to_string(),
    };
    let timeslots: Vec<&Timeslot> = vec![&timeslot_morgens, &timeslot_mittags, &timeslot_abends];

    let room_cpool = Room {
        identifier: "C-Pool".to_string(),
        requirements: vec![&requirement_computer_pool],
        max_size: RoomSize(128),
    };
    let room_bosch = Room {
        identifier: "Bosch".to_string(),
        requirements: vec![],
        max_size: RoomSize(75),
    };
    let room_draussen = Room {
        identifier: "draussen".to_string(),
        requirements: vec![&requirement_outside],
        max_size: RoomSize(200),
    };
    let rooms: Vec<&Room> = vec![&room_cpool, &room_bosch, &room_draussen];
    let rooms_in_timeslot: Vec<RoomInTimeSlot> = rooms
        .iter()
        .flat_map(|room| {
            timeslots
                .iter()
                .map(|timeslot| RoomInTimeSlot { room, timeslot })
        })
        .collect();

    let workshop_topic_linux = WorkshopTopic {
        identifier: "linux-lernen".to_string(),
        requirements: vec![&requirement_computer_pool],
        max_size: WorkshopTopicSize(50),
    };
    let workshop_topics: Vec<&WorkshopTopic> = vec![&workshop_topic_linux];

    let workshops: Vec<Workshop> = vec![Workshop {
        topic: workshop_topics[0],
        timeslot: timeslots[0],
    }];

    let participant_moritz = Participant {
        identifier: "moritz".to_string(),
    };
    let participants: Vec<&Participant> = vec![&participant_moritz];

    let preferences: Vec<&Preference> = vec![&Preference {
        participant: &participant_moritz,
        topic: &workshop_topic_linux,
        rank: Rank(0),
    }];

    //

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
