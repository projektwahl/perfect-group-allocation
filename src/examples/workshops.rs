// these will come from the database later
// it needs to be ensured that the names are *globally* unique, so it makes sense to append an id there
pub struct Requirement {
    pub identifier: String,
}

#[derive(Hash, Eq, PartialEq)]
pub struct Timeslot {
    pub identifier: String,
}

pub struct RoomSize(pub u8);

pub struct Room<'a> {
    pub identifier: String,
    pub requirements: Vec<&'a Requirement>,
    pub max_size: RoomSize,
}

/// Not every room may be available in every timeslot
pub struct RoomInTimeSlot<'a> {
    pub room: &'a Room<'a>,
    pub timeslot: &'a Timeslot,
}

pub struct WorkshopTopicSize(pub u8);

pub struct WorkshopTopic<'a> {
    pub identifier: String,
    pub requirements: Vec<&'a Requirement>,
    pub max_size: WorkshopTopicSize,
}

// in theory if a person holds multiple workshops the system could decide which one should be held how many times. but we probably leave that problem for now.
pub struct Workshop<'a> {
    pub topic: &'a WorkshopTopic<'a>,
    pub timeslot: &'a Timeslot,
}

pub struct Participant {
    pub identifier: String,
}

pub struct Rank(pub u8);

pub struct Preference<'a> {
    pub participant: &'a Participant,
    pub topic: &'a WorkshopTopic<'a>,
    pub rank: Rank,
}
