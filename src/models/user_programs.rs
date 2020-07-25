use diesel::prelude::*;

use crate::models::enrollments::Enrollment;
use crate::models::programs::Program;
use crate::models::users::User;
use crate::models::coaches::Coach;

use crate::schema::enrollments::dsl::*;
use crate::schema::programs::dsl::*;
use crate::schema::coaches::dsl::*;
use crate::schema::users::dsl::*;


#[derive(juniper::GraphQLEnum)]
pub enum Desire {
    EXPLORE,
    ENROLLED,
    YOURS,
}

#[derive(juniper::GraphQLInputObject)]
pub struct ProgramCriteria {
    user_fuzzy_id: String,
    program_fuzzy_id: String,
    desire: Desire,
}

pub struct ProgramRow {
    pub program: Program,
    pub coach: Coach,
}

#[juniper::object]
impl ProgramRow {
   
    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn coach(&self) -> &Coach {
        &self.coach
    }
}

type ProgramType = (Program,Coach);
pub type ProgramResult = Result<Vec<ProgramRow>,diesel::result::Error>;

pub fn get_programs(connection: &MysqlConnection, criteria: &ProgramCriteria) -> ProgramResult {

    match &criteria.desire {
        Desire::EXPLORE => get_latest_programs(connection),
        Desire::ENROLLED => get_enrolled_programs(connection, criteria),
        Desire::YOURS => get_coach_programs(connection, criteria),
    }
}

pub type ProgramFinderResult = Result<ProgramRow, diesel::result::Error>;

pub fn find_program(connection: &MysqlConnection,criteria: &ProgramCriteria)->ProgramFinderResult {
  
    use crate::schema::programs::dsl::fuzzy_id;

    let pc: ProgramType = programs
        .inner_join(coaches)
        .filter(fuzzy_id.eq(&criteria.program_fuzzy_id))
        .first(connection)?;

    Ok(ProgramRow{program:pc.0, coach:pc.1})    
}

fn get_enrolled_programs(connection: &MysqlConnection,criteria: &ProgramCriteria) -> ProgramResult {
    
    use crate::schema::users::dsl::fuzzy_id;
    type Row = (Enrollment, User, ProgramType);

    let data: Vec<Row> = enrollments
        .inner_join(users)
        .inner_join(programs.inner_join(coaches))
        .filter(fuzzy_id.eq(&criteria.user_fuzzy_id))
        .load(connection)?;

    let mut rows: Vec<ProgramRow> = Vec::new();

    for item in data {
        let pc = item.2;
        rows.push(ProgramRow{program:pc.0,coach:pc.1});
    }

    Ok(rows)
}

fn get_coach_programs(connection: &MysqlConnection,criteria: &ProgramCriteria) -> ProgramResult {
  
    use crate::schema::coaches::dsl::fuzzy_id;

    let data: Vec<ProgramType> = programs
        .inner_join(coaches)
        .filter(fuzzy_id.eq(&criteria.user_fuzzy_id))
        .order_by(name.asc())
        .load(connection)?;

    Ok(to_program_rows(data))
}


fn get_latest_programs(connection: &MysqlConnection)-> ProgramResult {

    use crate::schema::programs::dsl::created_at;

    let data: Vec<ProgramType> = programs
    .inner_join(coaches)
    .order_by(created_at.asc())
    .filter(active.eq(true))
    .limit(10)
    .load(connection)?;

    Ok(to_program_rows(data))
   
}

fn to_program_rows(data: Vec<ProgramType>) -> Vec<ProgramRow> {

    let mut rows: Vec<ProgramRow> = Vec::new();

    for pc in data {
        rows.push(ProgramRow{program:pc.0, coach:pc.1});
    }

    rows
}
