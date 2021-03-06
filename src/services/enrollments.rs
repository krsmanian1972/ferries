use diesel::prelude::*;

use crate::models::programs::Program;
use crate::models::users::User;

use crate::models::correspondences::{MailOut, MailRecipient};
use crate::models::enrollments::{Enrollment, EnrollmentCriteria, EnrollmentFilter, ManagedEnrollmentRequest, NewEnrollment, NewEnrollmentRequest};

use crate::services::correspondences::create_mail;
use crate::services::programs;
use crate::services::users;

use crate::schema::enrollments::dsl::*;
use crate::schema::programs::dsl::*;
use crate::schema::users::dsl::*;

const WARNING: &str = "It seems the user have already enrolled in this program or in a similar program offered by a peer coach.";
const ERROR_002: &str = "Error in creating enrollment. Error-002.";
const ERROR_003: &str = "Error in finding enrollment for the program and member. Error-003.";
const ERROR_004: &str = "Error in marking the enrollment as Old";
const QUERY_ERROR: &str = "Error in fetching enrolled members";

pub fn create_new_enrollment(connection: &MysqlConnection, request: &NewEnrollmentRequest) -> Result<Enrollment, &'static str> {
    let user: User = users::find(connection, request.user_id.as_str())?;
    let program: Program = programs::find(connection, request.program_id.as_str())?;

    gate_prior_enrollment(connection, &program, &user)?;
    insert_enrollment(connection, &program, &user)?;

    let enrollment = find(connection, &program, &user)?;

    let coach = users::find(connection, program.coach_id.as_str())?;

    create_self_enrollment_mail(connection, enrollment.id.as_str(), &program, &user, &coach)?;

    Ok(enrollment)
}

fn insert_enrollment(connection: &MysqlConnection, program: &Program, user: &User) -> Result<usize, &'static str> {
    let enrollment: NewEnrollment = NewEnrollment::from(&program, &user);
    let insert_result = diesel::insert_into(enrollments).values(enrollment).execute(connection);

    if insert_result.is_err() {
        return Err(ERROR_002);
    }

    Ok(insert_result.unwrap())
}

/**
 * For conferences we need to have the coach is enrolled in her own program.
 * This is because, the notes and other artifacts are tied to the session_user.
 * In order to create a session user we need a session that needs an enrollment.
 */
pub fn find_or_create_coach_enrollment(connection: &MysqlConnection, given_program_id: &str) -> Result<Enrollment, &'static str> {
    let program = programs::find(connection, given_program_id)?;
    let given_coach_id = program.coach_id.as_str();

    let enrollment_result: Result<Enrollment, diesel::result::Error> = enrollments.filter(program_id.eq(given_program_id)).filter(member_id.eq(given_coach_id)).first(connection);

    if let Ok(enrollment) = enrollment_result {
        return Ok(enrollment);
    }

    let user = users::find(connection, given_coach_id)?;
    insert_enrollment(connection, &program, &user)?;

    find(connection, &program, &user)
}

/**
 * Check if the User is enrolled into a Spawned or Root Program already.
 *
 */
fn gate_prior_enrollment(connection: &MysqlConnection, program: &Program, user: &User) -> Result<bool, &'static str> {
    let prog_query = programs.filter(parent_program_id.eq(program.coalesce_parent_id())).select(crate::schema::programs::id);

    let prior_enrollments: QueryResult<Enrollment> = enrollments.filter(member_id.eq(user.id.as_str())).filter(program_id.eq_any(prog_query)).first(connection);

    if prior_enrollments.is_err() {
        return Ok(true);
    }

    Err(WARNING)
}

pub fn find(connection: &MysqlConnection, program: &Program, user: &User) -> Result<Enrollment, &'static str> {
    let result = enrollments.filter(program_id.eq(program.id.to_owned())).filter(member_id.eq(user.id.to_owned())).first(connection);

    if result.is_err() {
        return Err(ERROR_003);
    }

    Ok(result.unwrap())
}

pub fn mark_as_old(connection: &MysqlConnection, enrollment_id: &str) -> Result<usize, &'static str> {
    let query = enrollments.filter(crate::schema::enrollments::id.eq(enrollment_id));

    let result = diesel::update(query).set(is_new.eq(false)).execute(connection);

    if result.is_err() {
        return Err(ERROR_004);
    }

    Ok(result.unwrap())
}

pub fn get_active_enrollments(connection: &MysqlConnection, criteria: EnrollmentCriteria) -> Result<Vec<User>, &'static str> {
    use crate::schema::users::dsl::*;

    let mut query = enrollments
        .inner_join(users)
        .filter(program_id.eq(criteria.program_id))
        .select(users::all_columns())
        .order_by(full_name.asc())
        .into_boxed();

    if let EnrollmentFilter::NEW = criteria.desire {
        query = query.filter(is_new.eq(true));
    }

    let result: QueryResult<Vec<User>> = query.load(connection);

    if result.is_err() {
        return Err(QUERY_ERROR);
    }

    Ok(result.unwrap())
}

const INVALID_MEMBER_MAIL: &str = "Invalid Member Mail Id";
const CONFLICT_PROGRAM_OWNER_MAIL: &str = "The coach does not have rights to enroll this member.";

/**
 * When a coach enrolls a member into her program
 */
pub fn create_managed_enrollment(connection: &MysqlConnection, request: &ManagedEnrollmentRequest) -> Result<Enrollment, &'static str> {
    let user_result: QueryResult<User> = users.filter(email.eq(request.member_mail.as_str())).first(connection);

    if user_result.is_err() {
        return Err(INVALID_MEMBER_MAIL);
    }

    let program_result: QueryResult<Program> = programs
        .filter(crate::schema::programs::id.eq(request.program_id.as_str()))
        .filter(coach_id.eq(request.coach_id.as_str()))
        .first(connection);

    if program_result.is_err() {
        return Err(CONFLICT_PROGRAM_OWNER_MAIL);
    }

    let member = user_result.unwrap();
    let program = program_result.unwrap();
    let coach = users::find(connection, request.coach_id.as_str())?;

    gate_prior_enrollment(connection, &program, &member)?;
    insert_enrollment(connection, &program, &member)?;

    let enrollment = find(connection, &program, &member)?;

    create_managed_enrollment_mail(connection, request, enrollment.id.as_str(), &member, &coach)?;

    Ok(enrollment)
}

/**
 * Mail when a coach enrolls a member into his program
 */
fn create_managed_enrollment_mail(connection: &MysqlConnection, request: &ManagedEnrollmentRequest, new_enroll_id: &str, member: &User, coach: &User) -> Result<usize, &'static str> {
    let mail_out = MailOut::for_managed_enrollment(request, new_enroll_id);
    let recipients = MailRecipient::build_recipients(member, coach, mail_out.id.as_str());

    create_mail(connection, mail_out, recipients)
}

/**
 * Mail when a member chooses a coach from a List of coaches of a Program
 */
fn create_self_enrollment_mail(connection: &MysqlConnection, enrollment_id: &str, program: &Program, member: &User, coach: &User) -> Result<usize, &'static str> {
    let mail_out = MailOut::for_self_enrollment(program, enrollment_id);
    let recipients = MailRecipient::build_recipients(member, coach, mail_out.id.as_str());

    create_mail(connection, mail_out, recipients)
}
