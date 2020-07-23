use crate::commons::util;
use crate::schema::session_users;

use crate::models::sessions::Session;
use crate::models::users::{User};

#[derive(Queryable, Debug, Identifiable)]
pub struct SessionUser {
    pub id: i32,
    pub fuzzy_id: String,
    pub session_id: i32,
    pub user_id: i32,
    pub user_type: String,
}

// Fields that we can safely expose to APIs
#[juniper::object]
impl SessionUser {

    pub fn fuzzy_id(&self) -> &str {
        self.fuzzy_id.as_str()
    }

    pub fn user_type(&self) -> &str {
        self.user_type.as_str()
    }
}


#[derive(Insertable)]
#[table_name = "session_users"]
pub struct NewSessionUser {
    pub fuzzy_id: String,
    pub session_id: i32,
    pub user_id: i32,
    pub user_type: String,
}

impl NewSessionUser {

    pub fn from(session: &Session, user: &User, session_user_type: &str) -> NewSessionUser {
        
        let fuzzy_id = util::fuzzy_id();
        
        NewSessionUser {
            fuzzy_id,
            session_id: session.id,
            user_id: user.id,
            user_type: String::from(session_user_type)
        }
    }
}
