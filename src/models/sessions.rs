
use crate::schema::sessions;
use crate::commons::util;
use crate::commons::chassis::{ValidationError};

use chrono::{NaiveDateTime,Duration};


// The Order of the fiels are very important 
#[derive(Queryable,Debug)]
pub struct Session {
    pub id:i32,
    pub program_id:i32,
    pub name:String,
    pub duration:i32,
    pub original_start_date : NaiveDateTime,
    pub original_end_date : NaiveDateTime,
    pub revised_start_date : Option<NaiveDateTime>,
    pub revised_end_date : Option<NaiveDateTime>,
    pub offered_start_date : Option<NaiveDateTime>,
    pub offered_end_date : Option<NaiveDateTime>,
    pub is_ready:bool,
    pub actual_start_date : Option<NaiveDateTime>,
    pub actual_end_date : Option<NaiveDateTime>,
    pub created_at : NaiveDateTime,
    pub updated_at : NaiveDateTime,
    pub description : Option<String>,
    pub fuzzy_id: String,
}

#[derive(juniper::GraphQLEnum)]
enum Status {
    DONE,
    PROGRESS,
    CANCELLED,
    READY,
    OVERDUE,
    PLANNED
}

// Fields that we can safely expose to APIs
#[juniper::object(description = "Fields that we can safely expose to APIs")]
impl Session {

    pub fn fuzzy_id(&self) -> &str {
        self.fuzzy_id.as_str()
    }

    pub fn program_id(&self) -> i32 {
        self.program_id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn description(&self) -> &str {
        let value: &str = match &self.description {
            None=>"_",
            Some(value)=>value.as_str()
        };

        value
    }

    pub fn duration(&self) -> i32 {
        self.duration
    }

    pub fn status(&self) -> Status {
        if self.actual_end_date.is_some() {
            return Status::DONE;
        }
        if self.actual_start_date.is_some() {
            return Status::PROGRESS;
        }
        if self.is_ready {
            return Status::READY;
        }
        
        Status::PLANNED
    }

    pub fn scheduleStart(&self) -> NaiveDateTime {
        self.revised_start_date.unwrap_or(self.original_start_date)
    }

    pub fn scheduleEnd(&self) -> NaiveDateTime {
        self.revised_end_date.unwrap_or(self.original_end_date)
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct NewSessionRequest {
    pub program_id: i32,
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
            errors.push(ValidationError::new("start_time","unparsable date."));
        }

        let date = util::as_date(given_time);
        if util::is_past_date(date) {
            errors.push(ValidationError::new("start_time","should be a future date."));
        }
        
        if self.duration <= 0 {
            errors.push(ValidationError::new("duration","should be a minimum of 1 hour."));
        }

        if self.program_id <= 0 {
            errors.push(ValidationError::new("program_id","program is invalid."));
        }

        if self.name.trim().is_empty() {
            errors.push(ValidationError::new("name","name of the session is a must."));
        }

        if self.description.trim().is_empty() {
            errors.push(ValidationError::new("desciption", "description of the session is a must."));
        }

        errors
    }
}

// The Persistable entity
#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    pub program_id: i32,
    pub name: String,
    pub duration: i32,
    pub original_start_date: NaiveDateTime,
    pub original_end_date: NaiveDateTime,
    pub description: String,
    pub fuzzy_id: String,
}

impl NewSession  {

    pub fn from(request: &NewSessionRequest) -> NewSession {
 
        let start_date = util::as_date(request.start_time.as_str());
        let duration = Duration::hours(request.duration as i64);
        let end_date = start_date.checked_add_signed(duration);

        let fuzzy_id = util::fuzzy_id();
        
        
        let new_session = NewSession {
                program_id:request.program_id,
                name:request.name.to_owned(),
                description:request.description.to_owned(),
                duration:request.duration,
                original_start_date:start_date,
                original_end_date: end_date.unwrap_or(start_date),
                fuzzy_id: fuzzy_id,
        };

        new_session
    }
}
