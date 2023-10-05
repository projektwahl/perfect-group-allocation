pub type TimeslotId = u8;

pub struct Timeslot {
    id: TimeslotId,
}

pub type WorkshopTopicId = u8;

pub struct WorkshopTopic {
    id: WorkshopTopicId,
}

pub type WorkshopId = u8;

pub struct Workshop {
    id: WorkshopId,
    timeslot: TimeslotId,
}

pub type ParticipantId = u8;

pub struct Participant {
    id: ParticipantId,
}

pub type Rank = u8;

pub struct Preference {
    participant: ParticipantId,
    topic: WorkshopTopicId,
    rank: Rank,
}

pub type RoomID = u8;

pub struct Room {
    id: RoomID,
}

// manche workshops haben festgeschriebene räume (roomrequirements)
// alle workshops präferenzenmod workshop;
