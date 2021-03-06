use crate::commons::chassis::ValidationError;
use crate::commons::util;
use crate::schema::sessions;

use chrono::{Duration, NaiveDateTime};

// The Order of the fiels are very important
#[derive(Queryable, Debug, Identifiable, Clone)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub program_id: String,
    pub enrollment_id: String,
    pub people: Option<String>,
    pub duration: i32,
    pub original_start_date: NaiveDateTime,
    pub original_end_date: NaiveDateTime,
    pub revised_start_date: Option<NaiveDateTime>,
    pub revised_end_date: Option<NaiveDateTime>,
    pub offered_start_date: Option<NaiveDateTime>,
    pub offered_end_date: Option<NaiveDateTime>,
    pub is_ready: bool,
    pub actual_start_date: Option<NaiveDateTime>,
    pub actual_end_date: Option<NaiveDateTime>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closing_notes: Option<String>,
    pub is_request: bool,
    pub conference_id: Option<String>,
    pub session_type: String,
}

#[derive(juniper::GraphQLEnum)]
enum Status {
    DONE,
    PROGRESS,
    CANCELLED,
    READY,
    OVERDUE,
    PLANNED,
}

// Fields that we can safely expose to APIs
#[juniper::object]
impl Session {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn program_id(&self) -> &str {
        self.program_id.as_str()
    }

    pub fn enrollment_id(&self) -> &str {
        self.enrollment_id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn description(&self) -> &str {
        let value: &str = match &self.description {
            None => "_",
            Some(value) => value.as_str(),
        };

        value
    }

    pub fn people(&self) -> &str {
        let value: &str = match &self.people {
            None => "_",
            Some(value) => value.as_str(),
        };

        value
    }

    pub fn duration(&self) -> i32 {
        self.duration
    }

    pub fn scheduleStart(&self) -> NaiveDateTime {
        self.revised_start_date.unwrap_or(self.original_start_date)
    }

    pub fn scheduleEnd(&self) -> NaiveDateTime {
        self.revised_end_date.unwrap_or(self.original_end_date)
    }

    pub fn actualStart(&self) -> Option<NaiveDateTime> {
        self.actual_start_date
    }

    pub fn actualEnd(&self) -> Option<NaiveDateTime> {
        self.actual_end_date
    }

    pub fn isClosed(&self) -> bool {
        if self.cancelled_at.is_some() {
            return true;
        }

        if self.actual_end_date.is_some() {
            return true;
        }

        false
    }

    pub fn status(&self) -> Status {
        if self.cancelled_at.is_some() {
            return Status::CANCELLED;
        }

        if self.actual_end_date.is_some() {
            return Status::DONE;
        }
        if self.actual_start_date.is_some() {
            return Status::PROGRESS;
        }

        if self.is_ready {
            return Status::READY;
        }

        let rev_start_date = self.revised_start_date.unwrap_or(self.original_start_date);

        if util::is_past_date(rev_start_date) {
            return Status::OVERDUE;
        }

        Status::PLANNED
    }

    pub fn closing_notes(&self) -> Option<String> {
        self.closing_notes.clone()
    }

    pub fn session_type(&self) -> &str {
        self.session_type.as_str()
    }

    pub fn conference_id(&self) -> Option<String> {
        self.conference_id.clone()
    }
}

impl Session {
    // Allow a session is deleted during limited situations
    pub fn can_delete(&self) -> bool {
        if self.cancelled_at.is_some() {
            return false;
        }

        if self.actual_start_date.is_some() {
            return false;
        }

        if self.is_ready {
            return false;
        }

        true
    }

    pub fn is_conference(&self) -> bool {
        self.session_type.eq("multi")
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct NewSessionRequest {
    pub program_id: String,
    pub member_id: String,
    pub name: String,
    pub description: String,
    pub duration: i32,
    pub start_time: String,
}

impl NewSessionRequest {
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors: Vec<ValidationError> = Vec::new();

        let given_time = self.start_time.as_str();

        if !util::is_valid_date(given_time) {
            errors.push(ValidationError::new("start_time", "unparsable date."));
        }

        let date = util::as_date(given_time);
        if util::is_past_date(date) {
            errors.push(ValidationError::new("start_time", "should be a future date."));
        }

        if self.duration < 15 {
            errors.push(ValidationError::new("duration", "should be a minimum of 15 minutes"));
        }

        if self.program_id.trim().is_empty() {
            errors.push(ValidationError::new("program_id", "Program fuzzy id is a must."));
        }

        if self.member_id.trim().is_empty() {
            errors.push(ValidationError::new("member_id", "Member fuzzy id is a must."));
        }

        if self.name.trim().is_empty() {
            errors.push(ValidationError::new("name", "name of the session is a must."));
        }

        if self.description.trim().is_empty() {
            errors.push(ValidationError::new("description", "description of the session is a must."));
        }

        errors
    }
}

// The Persistable entity
#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    pub id: String,
    pub name: String,
    pub description: String,
    pub program_id: String,
    pub enrollment_id: String,
    pub people: String,
    pub duration: i32,
    pub original_start_date: NaiveDateTime,
    pub original_end_date: NaiveDateTime,
    pub conference_id: Option<String>,
    pub session_type: String,
    pub is_ready: bool,
}

impl NewSession {
    pub fn from(request: &NewSessionRequest, enrollment_id: String, people: String) -> NewSession {
        let start_date = util::as_date(request.start_time.as_str());
        let duration = Duration::minutes(request.duration as i64);
        let end_date = start_date.checked_add_signed(duration);

        let fuzzy_id = util::fuzzy_id();

        NewSession {
            id: fuzzy_id,
            name: request.name.to_owned(),
            description: request.description.to_owned(),
            program_id: request.program_id.to_owned(),
            enrollment_id,
            people,
            duration: request.duration,
            original_start_date: start_date,
            original_end_date: end_date.unwrap_or(start_date),
            conference_id: None,
            session_type: util::MONO.to_owned(),
            is_ready:false,
        }
    }
}

#[derive(juniper::GraphQLEnum, PartialEq)]
pub enum TargetState {
    READY,
    START,
    DONE,
    CANCEL,
}

#[derive(juniper::GraphQLInputObject)]
pub struct ChangeSessionStateRequest {
    pub id: String,
    pub target_state: TargetState,
    pub closing_notes: Option<String>,
}
