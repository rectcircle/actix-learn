use chrono::NaiveDateTime;
use crate::schema::{posts, users};

#[derive(Debug, Queryable, Insertable, Identifiable, AsChangeset, Associations)]
#[belongs_to(User)]
pub struct Post {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub hair_color: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name="users"]
pub struct UserForInsert {
    pub name: String,
    pub hair_color: Option<String>,
}

#[derive(Debug, Identifiable, AsChangeset)]
#[table_name="users"]
pub struct UserForUpdate {
    pub id: i64,
    pub name: String,
    pub hair_color: Option<Option<String>>,
}

impl From<User> for UserForUpdate {
    fn from(user: User) -> Self {
        UserForUpdate {
            id: user.id,
            name: user.name,
            hair_color: Some(user.hair_color),
        }
    }
}
