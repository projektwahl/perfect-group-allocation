// these will come from the database later
// it needs to be ensured that the names are *globally* unique, so it makes sense to append an id there

pub struct Requirement {
    pub identifier: String,
}

pub struct Room<'a> {
    pub identifier: String,
    pub requirements: Vec<&'a Requirement>,
}

pub struct Timeslot {
    pub identifier: String,
}

pub struct WorkshopTopic<'a> {
    pub identifier: String,
    pub requirements: Vec<&'a Requirement>,
}

pub struct Workshop<'a> {
    pub topic: &'a WorkshopTopic<'a>,
    pub timeslot: &'a Timeslot,
}

pub struct Participant {
    pub identifier: String,
}

pub struct Rank(u8);

pub struct Preference<'a> {
    pub participant: &'a Participant,
    pub topic: &'a WorkshopTopic<'a>,
    pub rank: Rank,
}
