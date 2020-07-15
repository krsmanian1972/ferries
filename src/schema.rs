table! {
    enrollments (id) {
        id -> Integer,
        program_id -> Integer,
        team_id -> Integer,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    programs (id) {
        id -> Integer,
        name -> Varchar,
        coach_id -> Integer,
        active -> Bool,
        created_at -> Datetime,
        updated_at -> Datetime,
        fuzzy_id -> Varchar,
        description -> Nullable<Text>,
    }
}

table! {
    session_boards (id) {
        id -> Integer,
        fuzzy_id -> Varchar,
        session_id -> Integer,
        file_name -> Varchar,
        file_path -> Varchar,
        created_by_id -> Integer,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    session_files (id) {
        id -> Integer,
        fuzzy_id -> Varchar,
        session_note_id -> Integer,
        file_name -> Varchar,
        file_path -> Varchar,
        file_type -> Nullable<Varchar>,
        file_size -> Nullable<Integer>,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    session_links (id) {
        id -> Integer,
        source_session_id -> Integer,
        target_session_id -> Integer,
        lead_time -> Integer,
        buffer_time -> Integer,
        coordinates -> Text,
        priority -> Integer,
        is_forward -> Bool,
    }
}

table! {
    session_notes (id) {
        id -> Integer,
        fuzzy_id -> Varchar,
        session_id -> Integer,
        description -> Text,
        remind_at -> Nullable<Datetime>,
        created_by_id -> Integer,
        is_private -> Bool,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    session_users (id) {
        id -> Integer,
        fuzzy_id -> Varchar,
        session_id -> Integer,
        user_id -> Integer,
        user_type -> Varchar,
    }
}

table! {
    session_visits (id) {
        id -> Integer,
        session_id -> Integer,
        user_id -> Integer,
        joined_at -> Datetime,
    }
}

table! {
    sessions (id) {
        id -> Integer,
        program_id -> Integer,
        name -> Varchar,
        duration -> Integer,
        original_start_date -> Datetime,
        original_end_date -> Datetime,
        revised_start_date -> Nullable<Datetime>,
        revised_end_date -> Nullable<Datetime>,
        offered_start_date -> Nullable<Datetime>,
        offered_end_date -> Nullable<Datetime>,
        is_ready -> Bool,
        actual_start_date -> Nullable<Datetime>,
        actual_end_date -> Nullable<Datetime>,
        created_at -> Datetime,
        updated_at -> Datetime,
        description -> Nullable<Text>,
        fuzzy_id -> Varchar,
    }
}

table! {
    team_members (id) {
        id -> Integer,
        team_id -> Integer,
        user_id -> Integer,
        blocked -> Bool,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    teams (id) {
        id -> Integer,
        name -> Varchar,
        fuzzy_id -> Varchar,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

table! {
    users (id) {
        id -> Integer,
        full_name -> Varchar,
        email -> Varchar,
        fuzzy_id -> Varchar,
        blocked -> Bool,
        created_at -> Datetime,
        updated_at -> Datetime,
    }
}

joinable!(enrollments -> programs (program_id));
joinable!(enrollments -> teams (team_id));
joinable!(programs -> users (coach_id));
joinable!(session_boards -> sessions (session_id));
joinable!(session_boards -> users (created_by_id));
joinable!(session_files -> session_notes (session_note_id));
joinable!(session_notes -> sessions (session_id));
joinable!(session_notes -> users (created_by_id));
joinable!(session_users -> sessions (session_id));
joinable!(session_users -> users (user_id));
joinable!(session_visits -> sessions (session_id));
joinable!(session_visits -> users (user_id));
joinable!(sessions -> programs (program_id));
joinable!(team_members -> teams (team_id));
joinable!(team_members -> users (user_id));

allow_tables_to_appear_in_same_query!(
    enrollments,
    programs,
    session_boards,
    session_files,
    session_links,
    session_notes,
    session_users,
    session_visits,
    sessions,
    team_members,
    teams,
    users,
);
