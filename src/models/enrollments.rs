use chrono::NaiveDateTime;

use crate::models::programs::Program;
use crate::models::users::User;

use crate::commons::chassis::ValidationError;
use crate::commons::util;

use crate::schema::enrollments;

#[derive(Queryable, Debug, Identifiable)]
pub struct Enrollment {
    pub id: String,
    pub program_id: String,
    pub member_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_new: bool,
}

#[juniper::object(description = "The fields we offer to the Web-UI ")]
impl Enrollment {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }
    pub fn program_id(&self) -> &str {
        self.program_id.as_str()
    }
    pub fn member_id(&self) -> &str {
        self.member_id.as_str()
    }
    pub fn is_new(&self) -> bool {
        self.is_new
    }
    pub fn created_at(&self) -> &NaiveDateTime {
        &self.created_at
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct NewEnrollmentRequest {
    pub program_id: String,
    pub user_id: String,
    pub coach_id: String,
}

impl NewEnrollmentRequest {
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors: Vec<ValidationError> = Vec::new();

        if self.program_id.trim().is_empty() {
            errors.push(ValidationError::new("program_id", "The Program id is invalid."));
        }

        if self.user_id.trim().is_empty() {
            errors.push(ValidationError::new("user_id", "The User id is invalid."));
        }

        if self.coach_id.trim().is_empty() {
            errors.push(ValidationError::new("coach_id", "We need the coach id who offers the program."));
        }

        errors
    }
}

#[derive(juniper::GraphQLEnum)]
pub enum EnrollmentFilter {
    ALL,
    NEW,
}

#[derive(juniper::GraphQLInputObject)]
pub struct EnrollmentCriteria {
    pub program_id: String,
    pub desire: EnrollmentFilter,
}

#[derive(juniper::GraphQLInputObject)]
pub struct PlanCriteria {
    pub enrollment_id: String,
}

#[derive(Insertable)]
#[table_name = "enrollments"]
pub struct NewEnrollment {
    pub id: String,
    pub program_id: String,
    pub member_id: String,
}

impl NewEnrollment {
    pub fn from(program: &Program, user: &User) -> NewEnrollment {
        let fuzzy_id = util::fuzzy_id();
        NewEnrollment {
            id: fuzzy_id,
            program_id: program.id.to_owned(),
            member_id: user.id.to_owned(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct ManagedEnrollmentRequest {
    pub program_id: String,
    pub coach_id: String,
    pub member_mail: String,
    pub subject: String,
    pub message: String
}

