use crate::commons::chassis::ValidationError;
use crate::commons::util;
use crate::schema::tasks;

use chrono::{Duration, NaiveDateTime};

#[derive(Queryable, Debug, Identifiable)]
pub struct Task {
    pub id: String,
    pub enrollment_id: String,
    pub actor_id: String,
    pub name: String,
    pub duration: i32,
    pub min: i32,
    pub max: i32,
    pub original_start_date: NaiveDateTime,
    pub original_end_date: NaiveDateTime,
    pub revised_start_date: Option<NaiveDateTime>,
    pub revised_end_date: Option<NaiveDateTime>,
    pub offered_start_date: Option<NaiveDateTime>,
    pub offered_end_date: Option<NaiveDateTime>,
    pub actual_start_date: Option<NaiveDateTime>,
    pub actual_end_date: Option<NaiveDateTime>,
    pub locked: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub description: Option<String>,
    pub closing_notes: Option<String>,
    pub response: Option<String>,
    pub approved_at: Option<NaiveDateTime>,
    pub cancelled_at: Option<NaiveDateTime>,
    pub responded_date: Option<NaiveDateTime>,
}

#[derive(juniper::GraphQLEnum)]
enum Status {
    PLANNED,
    CANCELLED,
    DUE,
    DELAY,
    PROGRESS,
    RESPONDED,
    DONE
}

#[juniper::object]
impl Task {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn enrollmentId(&self) -> &str {
        self.enrollment_id.as_str()
    }

    pub fn description(&self) -> &str {
        let value: &str = match &self.description {
            None => "_",
            Some(value) => value.as_str(),
        };
        value
    }

    pub fn actorId(&self) -> &str {
        self.actor_id.as_str()
    }

    pub fn duration(&self) -> i32 {
        self.duration
    }

    pub fn min(&self) -> i32 {
        self.min
    }

    pub fn max(&self) -> i32 {
        self.max
    }

    pub fn scheduleStart(&self) -> NaiveDateTime {
        self.revised_start_date.unwrap_or(self.original_start_date)
    }

    pub fn scheduleEnd(&self) -> NaiveDateTime {
        self.revised_end_date.unwrap_or(self.original_end_date)
    }

    pub fn createdAt(&self) -> NaiveDateTime {
        self.created_at
    }

    pub fn actualStart(&self) -> Option<NaiveDateTime> {
        self.actual_start_date
    }

    pub fn response(&self) -> &str {
        let value: &str = match &self.response {
            None => "",
            Some(value) => value.as_str(),
        };
        value
    }
    
    pub fn respondedDate(&self) -> Option<NaiveDateTime> {
        self.responded_date
    }

    pub fn closingNotes(&self) -> &str {
        let value: &str = match &self.closing_notes {
            None => "",
            Some(value) => value.as_str(),
        };
        value
    }

    pub fn actualEnd(&self) -> Option<NaiveDateTime> {
        self.actual_end_date
    }

    pub fn cancelledDate(&self) -> Option<NaiveDateTime> {
        self.cancelled_at
    }

   
    pub fn status(&self) -> Status {

        if self.cancelled_at.is_some() {
            return Status::CANCELLED;
        }
    
        if self.actual_end_date.is_some() {
            return Status::DONE;
        }

        if self.responded_date.is_some() {
            return Status::RESPONDED;
        }

        let rev_end_date = self.revised_end_date.unwrap_or(self.original_end_date);
        if util::is_past_date(rev_end_date) {
            return Status::DELAY;
        }

        if self.actual_start_date.is_some() {
            return Status::PROGRESS;
        }

        let rev_start_date = self.revised_start_date.unwrap_or(self.original_start_date);
        if util::is_past_date(rev_start_date) {
            return Status::DUE;
        }

        Status::PLANNED
    }
 
 
    pub fn canStart(&self) -> bool {
        self.can_start()
    }

    pub fn canRespond(&self) -> bool {
        self.can_respond()
    }

    pub fn canFinish(&self) -> bool {
        self.can_finish()
    }

    pub fn canComplete(&self) -> bool {
        self.can_complete()
    }

    pub fn canCancel(&self) -> bool {
        self.can_cancel()
    }

    pub fn canReopen(&self) -> bool {
        self.can_reopen()
    }
}

impl Task {

    pub fn can_start(&self) -> bool {
        self.actual_start_date.is_none() && self.responded_date.is_none() && self.cancelled_at.is_none() && self.actual_end_date.is_none()
    }

    pub fn can_respond(&self) -> bool {
        self.cancelled_at.is_none() && self.actual_end_date.is_none() && self.responded_date.is_none() && self.actual_start_date.is_some()
    }

    pub fn can_finish(&self) -> bool {
        self.actual_start_date.is_some() && self.response.is_some() && self.cancelled_at.is_none() && self.responded_date.is_none() && self.actual_end_date.is_none()
    }

    pub fn can_complete(&self) -> bool {
        self.actual_end_date.is_none() && self.cancelled_at.is_none() && self.responded_date.is_some()
    }

    pub fn can_cancel(&self) -> bool {
        self.actual_end_date.is_none() && self.cancelled_at.is_none()
    }

    pub fn can_reopen(&self) -> bool {
        self.responded_date.is_some()
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct NewTaskRequest {
    pub enrollment_id: String,
    pub actor_id: String,
    pub start_time: String,
    pub duration: i32,
    pub description: String,
    pub name: String,
}

impl NewTaskRequest {
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors: Vec<ValidationError> = Vec::new();

        let given_time = self.start_time.as_str();

        if !util::is_valid_date(given_time) {
            errors.push(ValidationError::new("start_time", "unparsable date."));
        }

        let date = util::as_date(given_time);
        if util::is_in_past(date) {
            errors.push(ValidationError::new("start_time", "should be a future date."));
        }

        if self.duration <= 0 {
            errors.push(ValidationError::new("duration", "should be a minimum of 1 hour."));
        }

        if self.enrollment_id.trim().is_empty() {
            errors.push(ValidationError::new("enrollment_id", "Enrollment Id is a must."));
        }

        errors
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct UpdateTaskRequest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub start_time: String,
    pub duration: i32,
}

impl UpdateTaskRequest {
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors: Vec<ValidationError> = Vec::new();
        let given_time = self.start_time.as_str();

        if self.id.trim().is_empty() {
            errors.push(ValidationError::new("id", "Id is a must."));
        }

        if !util::is_valid_date(given_time) {
            errors.push(ValidationError::new("start_time", "unparsable date."));
        }

        let date = util::as_date(given_time);
        if util::is_in_past(date) {
            errors.push(ValidationError::new("start_time", "should be a future date."));
        }

        if self.duration <= 0 {
            errors.push(ValidationError::new("duration", "should be a minimum of 1 hour."));
        }

        errors
    }
}

#[derive(Insertable)]
#[table_name = "tasks"]
pub struct NewTask {
    pub id: String,
    pub enrollment_id: String,
    pub actor_id: String,
    pub duration: i32,
    pub original_start_date: NaiveDateTime,
    pub original_end_date: NaiveDateTime,
    pub description: String,
    pub name: String,
}

impl NewTask {
    pub fn from(request: &NewTaskRequest) -> NewTask {
        let start_date = util::as_date(request.start_time.as_str());
        let duration = Duration::hours(request.duration as i64);
        let end_date = start_date.checked_add_signed(duration);

        let fuzzy_id = util::fuzzy_id();

        NewTask {
            id: fuzzy_id,
            enrollment_id: request.enrollment_id.to_owned(),
            actor_id: request.actor_id.to_owned(),
            duration: request.duration,
            original_start_date: start_date,
            original_end_date: end_date.unwrap_or(start_date),
            description: request.description.to_owned(),
            name: request.name.to_owned(),
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "tasks"]
pub struct UpdateTask {
    pub description: String,
    pub name: String,
    pub duration: i32,
    pub original_start_date: NaiveDateTime,
    pub original_end_date: NaiveDateTime,
}

#[derive(juniper::GraphQLInputObject)]
pub struct UpdateResponseRequest {
    pub id: String,
    pub response: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct UpdateClosingNoteRequest {
    pub id: String,
    pub notes: String,
}

#[derive(juniper::GraphQLEnum, PartialEq)]
pub enum CoachTargetState {
    DONE,
    CANCEL,
    REOPEN,
}

#[derive(juniper::GraphQLInputObject)]
pub struct ChangeCoachTaskStateRequest {
    pub id: String,
    pub target_state: CoachTargetState,
}

#[derive(juniper::GraphQLEnum, PartialEq)]
pub enum MemberTargetState {
    START,
    FINISH,
}

#[derive(juniper::GraphQLInputObject)]
pub struct ChangeMemberTaskStateRequest {
    pub id: String,
    pub target_state: MemberTargetState,
}