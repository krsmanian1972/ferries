use juniper::{FieldResult, RootNode};

use crate::db_manager::MySqlConnectionPool;

use crate::models::abstract_tasks::{AbstractTask, AbstractTaskCriteria, NewAbstractTaskRequest};
use crate::models::coach_members::{get_coach_members, CoachCriteria, MemberRow};
use crate::models::conferences::{Conference, MemberRequest, NewConferenceRequest};
use crate::models::correspondences::Mailable;
use crate::models::discussion_queue::PendingFeed;
use crate::models::discussions::{Discussion, DiscussionCriteria, NewDiscussionRequest};
use crate::models::enrollments::{Enrollment, EnrollmentCriteria, ManagedEnrollmentRequest, NewEnrollmentRequest, PlanCriteria};
use crate::models::master_plans::{MasterPlan, MasterPlanCriteria, NewMasterPlanRequest, UpdateMasterPlanRequest};
use crate::models::master_tasks::{MasterTask, MasterTaskCriteria, NewMasterTaskRequest, UpdateMasterTaskRequest};
use crate::models::notes::{NewNoteRequest, Note, NoteCriteria};
use crate::models::objectives::{NewObjectiveRequest, Objective, UpdateObjectiveRequest};
use crate::models::observations::{NewObservationRequest, Observation, UpdateObservationRequest};
use crate::models::options::{Constraint, NewOptionRequest, UpdateOptionRequest};
use crate::models::programs::{AssociateCoachRequest, ChangeProgramStateRequest, NewProgramRequest, Program, ProgramCoach};
use crate::models::sessions::{ChangeSessionStateRequest, NewSessionRequest, Session};
use crate::models::tasks::{ChangeCoachTaskStateRequest, ChangeMemberTaskStateRequest, NewTaskRequest, Task, UpdateClosingNoteRequest, UpdateResponseRequest, UpdateTaskRequest};
use crate::models::user_artifacts::{get_boards, get_enrollment_notes, BoardRow, NoteRow};
use crate::models::user_events::{get_events,get_plan_events, get_to_dos, EventCriteria, EventRow, PlanRow, ToDo};
use crate::models::session_users::{get_people,SessionCriteria, SessionPeople};
use crate::models::user_programs::{get_programs, ProgramCriteria, ProgramRow};
use crate::models::users::{LoginRequest, Registration, ResetPasswordRequest, User, UserCriteria};

use crate::services::abstract_tasks::{create_abstract_task, get_abstract_tasks};
use crate::services::conferences::{create_conference, manage_members};
use crate::services::correspondences::sendable_mails;
use crate::services::discussions::{create_new_discussion, get_discussions, get_pending_discussions};
use crate::services::enrollments::{create_managed_enrollment, create_new_enrollment, get_active_enrollments};
use crate::services::master_plans::{create_master_plan, get_master_plans, update_master_plan};
use crate::services::master_tasks::{create_master_task, get_master_tasks, update_master_task};
use crate::services::notes::{create_new_note, get_notes};
use crate::services::objectives::{create_objective, get_objectives, update_objective};
use crate::services::observations::{create_observation, get_observations, update_observation};
use crate::services::options::{create_option, get_options, update_option};
use crate::services::programs::{associate_coach, change_program_state, create_new_program, get_peer_coaches};
use crate::services::sessions::{change_session_state, create_session, find};
use crate::services::tasks::{change_coach_task_state, change_member_task_state, create_task, get_tasks, update_closing_notes, update_response, update_task};
use crate::services::users::{authenticate, register, reset_password};

use crate::commons::chassis::{mutation_error, query_error, service_error, MutationResult, QueryError, QueryResult};

#[derive(Clone)]
pub struct DBContext {
    pub db: MySqlConnectionPool,
}


pub struct QueryRoot;

#[juniper::object(Context = DBContext,description="Graph Query Root")]
impl QueryRoot {
    #[graphql(description = "Authenticate a user with email and password")]
    fn authenticate(context: &DBContext, request: LoginRequest) -> FieldResult<User> {
        let connection = context.db.get().unwrap();
        let user = authenticate(&connection, request)?;
        Ok(user)
    }

    #[graphql(description = "Return the basic information of a user")]
    fn get_user(context: &DBContext, criteria: UserCriteria) -> FieldResult<User> {
        let connection = context.db.get().unwrap();
        let user = crate::services::users::find(&connection, &criteria.id)?;
        Ok(user)
    }

    fn get_pending_discussions(context: &DBContext, criteria: UserCriteria) -> QueryResult<Vec<PendingFeed>> {
        let connection = context.db.get().unwrap();
        let result = get_pending_discussions(&connection, &criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get Programs of a Coach Or Member Or Latest 10.")]
    fn get_programs(context: &DBContext, criteria: ProgramCriteria) -> QueryResult<Vec<ProgramRow>> {
        let connection = context.db.get().unwrap();
        let result = get_programs(&connection, &criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of coaches associated with a Program through its parent program.")]
    fn get_program_coaches(context: &DBContext, program_id: String) -> QueryResult<Vec<ProgramCoach>> {
        let connection = context.db.get().unwrap();
        let result = get_peer_coaches(&connection, program_id.as_str());

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get The List of Abstract Tasks of a Coach")]
    fn get_abstract_tasks(context: &DBContext, criteria: AbstractTaskCriteria) -> QueryResult<Vec<AbstractTask>> {
        let connection = context.db.get().unwrap();
        let result = get_abstract_tasks(&connection, &criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get The List of Master Plans of a Coach")]
    fn get_master_plans(context: &DBContext, criteria: MasterPlanCriteria) -> QueryResult<Vec<MasterPlan>> {
        let connection = context.db.get().unwrap();
        let result = get_master_plans(&connection, &criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of tasks for an Enrollment")]
    fn get_master_tasks(context: &DBContext, criteria: MasterTaskCriteria) -> QueryResult<Vec<MasterTask>> {
        let connection = context.db.get().unwrap();
        let result = get_master_tasks(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of members enrolled into a Program")]
    fn get_enrollments(context: &DBContext, criteria: EnrollmentCriteria) -> Vec<User> {
        let connection = context.db.get().unwrap();
        get_active_enrollments(&connection, criteria).unwrap()
    }

    #[graphql(description = "Get the list of members enrolled into Programs offered by a Coach")]
    fn get_coach_members(context: &DBContext, criteria: CoachCriteria) -> QueryResult<Vec<MemberRow>> {
        let connection = context.db.get().unwrap();
        let result = get_coach_members(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the Session Events for a User, during a period")]
    fn get_events(context: &DBContext, criteria: EventCriteria) -> QueryResult<Vec<EventRow>> {
        let connection = context.db.get().unwrap();
        let result = get_events(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => QueryResult(Err(e)),
        }
    }

    #[graphql(description = "Get the list of Plan Events for a User")]
    fn get_plan_events(context: &DBContext, criteria: EventCriteria) -> QueryResult<Vec<PlanRow>> {
        let connection = context.db.get().unwrap();
        let result = get_plan_events(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => QueryResult(Err(QueryError { message: e })),
        }
    }

    #[graphql(description = "Get the list of events due for a user")]
    fn get_due(context: &DBContext, criteria: EventCriteria) -> QueryResult<Vec<ToDo>> {
        let connection = context.db.get().unwrap();
        let result = get_to_dos(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => QueryResult(Err(QueryError { message: e })),
        }
    }

    #[graphql(description = "Get the list of objectives for an Enrollment")]
    fn get_objectives(context: &DBContext, criteria: PlanCriteria) -> QueryResult<Vec<Objective>> {
        let connection = context.db.get().unwrap();
        let result = get_objectives(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of options for an Enrollment")]
    fn get_options(context: &DBContext, criteria: PlanCriteria) -> QueryResult<Vec<Constraint>> {
        let connection = context.db.get().unwrap();
        let result = get_options(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of observations for an Enrollment")]
    fn get_observations(context: &DBContext, criteria: PlanCriteria) -> QueryResult<Vec<Observation>> {
        let connection = context.db.get().unwrap();
        let result = get_observations(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of tasks for an Enrollment")]
    fn get_tasks(context: &DBContext, criteria: PlanCriteria) -> QueryResult<Vec<Task>> {
        let connection = context.db.get().unwrap();
        let result = get_tasks(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of notes for a SessionUser")]
    fn get_notes(context: &DBContext, criteria: NoteCriteria) -> QueryResult<Vec<Note>> {
        let connection = context.db.get().unwrap();
        let result = get_notes(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    fn get_discussions(context: &DBContext, criteria: DiscussionCriteria) -> QueryResult<Vec<Discussion>> {
        let connection = context.db.get().unwrap();
        let result = get_discussions(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the list of notes of an enrollment. Hence both the member and the coach notes directly to the member.")]
    fn get_enrollment_notes(context: &DBContext, criteria: PlanCriteria) -> QueryResult<Vec<NoteRow>> {
        let connection = context.db.get().unwrap();
        let result = get_enrollment_notes(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the Session by its id")]
    fn get_session(context: &DBContext, criteria: SessionCriteria) -> FieldResult<Session> {
        let connection = context.db.get().unwrap();
        let session = find(&connection, &criteria.id)?;
        Ok(session)
    }

    #[graphql(description = "Get the People participating in an Event")]
    fn get_session_users(context: &DBContext, criteria: SessionCriteria) -> QueryResult<Vec<SessionPeople>> {
        let connection = context.db.get().unwrap();
        let result = get_people(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Top 3 mails marked as Pending")]
    fn get_sendable_mails(context: &DBContext) -> QueryResult<Vec<Mailable>> {
        let connection = context.db.get().unwrap();
        let result = sendable_mails(&connection);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }

    #[graphql(description = "Get the List of all the Boards of an enrolled member")]
    fn get_boards(context: &DBContext, criteria: EventCriteria) -> QueryResult<Vec<BoardRow>> {
        let connection = context.db.get().unwrap();
        let result = get_boards(&connection, criteria);

        match result {
            Ok(value) => QueryResult(Ok(value)),
            Err(e) => query_error(e),
        }
    }
}

pub struct MutationRoot;

#[juniper::object(Context = DBContext)]
impl MutationRoot {
    fn create_user(context: &DBContext, registration: Registration) -> MutationResult<User> {

        let connection = context.db.get().unwrap();
        let result = register(&connection, &registration);

        match result {
            Ok(user) => MutationResult(Ok(user)),
            Err(e) => MutationResult(Err(e.errors)),
        }
    }

    fn reset_password(context: &DBContext, request: ResetPasswordRequest) -> MutationResult<User> {
        let errors = request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = reset_password(&connection, &request);

        match result {
            Ok(user) => MutationResult(Ok(user)),
            Err(e) => service_error(e),
        }
    }

    fn create_abstract_task(context: &DBContext, request: NewAbstractTaskRequest) -> MutationResult<AbstractTask> {
        let errors = request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_abstract_task(&connection, &request);

        match result {
            Ok(abstract_task) => MutationResult(Ok(abstract_task)),
            Err(e) => mutation_error(e),
        }
    }

    fn create_master_plan(context: &DBContext, request: NewMasterPlanRequest) -> MutationResult<MasterPlan> {
        let errors = request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_master_plan(&connection, &request);

        match result {
            Ok(master_plan) => MutationResult(Ok(master_plan)),
            Err(e) => mutation_error(e),
        }
    }

    fn create_master_task(context: &DBContext, new_master_task_request: NewMasterTaskRequest) -> MutationResult<MasterTask> {
        let errors = new_master_task_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_master_task(&connection, &new_master_task_request);

        match result {
            Ok(master_task) => MutationResult(Ok(master_task)),
            Err(e) => mutation_error(e),
        }
    }

    fn update_master_task(context: &DBContext, update_master_task_request: UpdateMasterTaskRequest) -> MutationResult<MasterTask> {
        let errors = update_master_task_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = update_master_task(&connection, &update_master_task_request);

        match result {
            Ok(task) => MutationResult(Ok(task)),
            Err(e) => mutation_error(e),
        }
    }

    fn save_master_plan(context: &DBContext, request: UpdateMasterPlanRequest) -> MutationResult<String> {
        let connection = context.db.get().unwrap();
        let result = update_master_plan(&connection, &request);

        match result {
            Ok(value) => MutationResult(Ok(value)),
            Err(e) => mutation_error(e),
        }
    }

    fn create_program(context: &DBContext, new_program_request: NewProgramRequest) -> MutationResult<Program> {
        let errors = new_program_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_new_program(&connection, &new_program_request);

        match result {
            Ok(program) => MutationResult(Ok(program)),
            Err(e) => service_error(e),
        }
    }

    fn associate_coach(context: &DBContext, request: AssociateCoachRequest) -> MutationResult<Program> {
        let connection = context.db.get().unwrap();
        let result = associate_coach(&connection, &request);

        match result {
            Ok(program) => MutationResult(Ok(program)),
            Err(e) => service_error(e),
        }
    }

    fn create_enrollment(context: &DBContext, new_enrollment_request: NewEnrollmentRequest) -> MutationResult<Enrollment> {
        let errors = new_enrollment_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_new_enrollment(&connection, &new_enrollment_request);

        match result {
            Ok(enrollment) => MutationResult(Ok(enrollment)),
            Err(e) => service_error(e),
        }
    }

    fn managed_enrollment(context: &DBContext, managed_enrollment_request: ManagedEnrollmentRequest) -> MutationResult<Enrollment> {
        let connection = context.db.get().unwrap();
        let result = create_managed_enrollment(&connection, &managed_enrollment_request);

        match result {
            Ok(enrollment) => MutationResult(Ok(enrollment)),
            Err(e) => service_error(e),
        }
    }

    fn create_session(context: &DBContext, new_session_request: NewSessionRequest) -> MutationResult<Session> {
        let errors = new_session_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_session(&connection, &new_session_request);

        match result {
            Ok(session) => MutationResult(Ok(session)),
            Err(e) => service_error(e),
        }
    }

    fn create_conference(context: &DBContext, new_conference_request: NewConferenceRequest) -> MutationResult<Conference> {
        let errors = new_conference_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_conference(&connection, &new_conference_request);

        match result {
            Ok(conference) => MutationResult(Ok(conference)),
            Err(e) => service_error(e),
        }
    }

    fn manage_conference(context: &DBContext, member_request: MemberRequest) -> MutationResult<Vec<String>> {
        let connection = context.db.get().unwrap();
        let result = manage_members(&connection, &member_request);

        match result {
            Ok(members) => MutationResult(Ok(members)),
            Err(e) => service_error(e),
        }
    }

    fn create_objective(context: &DBContext, new_objective_request: NewObjectiveRequest) -> MutationResult<Objective> {
        let errors = new_objective_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_objective(&connection, &new_objective_request);

        match result {
            Ok(objective) => MutationResult(Ok(objective)),
            Err(e) => mutation_error(e),
        }
    }

    fn create_option(context: &DBContext, new_option_request: NewOptionRequest) -> MutationResult<Constraint> {
        let errors = new_option_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_option(&connection, &new_option_request);

        match result {
            Ok(option) => MutationResult(Ok(option)),
            Err(e) => mutation_error(e),
        }
    }

    fn create_observation(context: &DBContext, new_observation_request: NewObservationRequest) -> MutationResult<Observation> {
        let errors = new_observation_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_observation(&connection, &new_observation_request);

        match result {
            Ok(option) => MutationResult(Ok(option)),
            Err(e) => mutation_error(e),
        }
    }

    fn update_observation(context: &DBContext, update_observation_request: UpdateObservationRequest) -> MutationResult<Observation> {
        let errors = update_observation_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = update_observation(&connection, &update_observation_request);

        match result {
            Ok(obs) => MutationResult(Ok(obs)),
            Err(e) => mutation_error(e),
        }
    }

    fn update_option(context: &DBContext, update_option_request: UpdateOptionRequest) -> MutationResult<Constraint> {
        let errors = update_option_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = update_option(&connection, &update_option_request);

        match result {
            Ok(option) => MutationResult(Ok(option)),
            Err(e) => mutation_error(e),
        }
    }

    fn update_objective(context: &DBContext, update_objective_request: UpdateObjectiveRequest) -> MutationResult<Objective> {
        let errors = update_objective_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = update_objective(&connection, &update_objective_request);

        match result {
            Ok(objective) => MutationResult(Ok(objective)),
            Err(e) => mutation_error(e),
        }
    }

    fn create_task(context: &DBContext, new_task_request: NewTaskRequest) -> MutationResult<Task> {
        let errors = new_task_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_task(&connection, &new_task_request);

        match result {
            Ok(task) => MutationResult(Ok(task)),
            Err(e) => mutation_error(e),
        }
    }

    fn update_task(context: &DBContext, update_task_request: UpdateTaskRequest) -> MutationResult<Task> {
        let errors = update_task_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = update_task(&connection, &update_task_request);

        match result {
            Ok(task) => MutationResult(Ok(task)),
            Err(e) => mutation_error(e),
        }
    }

    fn update_task_closing_notes(context: &DBContext, request: UpdateClosingNoteRequest) -> MutationResult<Task> {
        let connection = context.db.get().unwrap();
        let result = update_closing_notes(&connection, &request);
        match result {
            Ok(task) => MutationResult(Ok(task)),
            Err(e) => service_error(e),
        }
    }

    fn update_task_response(context: &DBContext, request: UpdateResponseRequest) -> MutationResult<Task> {
        let connection = context.db.get().unwrap();
        let result = update_response(&connection, &request);
        match result {
            Ok(task) => MutationResult(Ok(task)),
            Err(e) => service_error(e),
        }
    }
    fn alter_coach_task_state(context: &DBContext, request: ChangeCoachTaskStateRequest) -> MutationResult<Task> {
        let connection = context.db.get().unwrap();
        let result = change_coach_task_state(&connection, &request);
        match result {
            Ok(task) => MutationResult(Ok(task)),
            Err(e) => service_error(e),
        }
    }

    fn alter_member_task_state(context: &DBContext, request: ChangeMemberTaskStateRequest) -> MutationResult<Task> {
        let connection = context.db.get().unwrap();
        let result = change_member_task_state(&connection, &request);
        match result {
            Ok(task) => MutationResult(Ok(task)),
            Err(e) => service_error(e),
        }
    }

    fn alter_session_state(context: &DBContext, request: ChangeSessionStateRequest) -> MutationResult<Session> {
        let connection = context.db.get().unwrap();
        let result = change_session_state(&connection, &request);
        match result {
            Ok(session) => MutationResult(Ok(session)),
            Err(e) => service_error(e),
        }
    }

    fn alter_program_state(context: &DBContext, request: ChangeProgramStateRequest) -> MutationResult<String> {
        let connection = context.db.get().unwrap();
        let result = change_program_state(&connection, &request);

        match result {
            Ok(rows) => MutationResult(Ok(String::from("Ok"))),
            Err(e) => service_error(e),
        }
    }

    fn create_note(context: &DBContext, new_note_request: NewNoteRequest) -> MutationResult<Note> {
        let errors = new_note_request.validate();
        if !errors.is_empty() {
            return MutationResult(Err(errors));
        }

        let connection = context.db.get().unwrap();
        let result = create_new_note(&connection, &new_note_request);

        match result {
            Ok(note) => MutationResult(Ok(note)),
            Err(e) => mutation_error(e),
        }
    }

    fn create_discussion(context: &DBContext, new_discussion_request: NewDiscussionRequest) -> MutationResult<Discussion> {
        let connection = context.db.get().unwrap();
        let result = create_new_discussion(&connection, &new_discussion_request);

        match result {
            Ok(discussion) => MutationResult(Ok(discussion)),
            Err(e) => mutation_error(e),
        }
    }
}

pub type GQSchema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_gq_schema() -> GQSchema {
    GQSchema::new(QueryRoot {}, MutationRoot {})
}
