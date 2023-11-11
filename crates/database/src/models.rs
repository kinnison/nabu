//! Core model functionality for the Nabu database
//!

use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, Queryable};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Queryable)]
pub struct Identity {
    pub id: i32,
    pub name: String,
    pub admin: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::identity)]
pub struct NewIdentity<'a> {
    pub name: &'a str,
    pub admin: bool,
}

#[derive(Queryable)]
pub struct Token {
    pub id: i32,
    pub identity: i32,
    pub title: String,
    pub content: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::token)]
pub struct NewToken<'a> {
    pub identity: i32,
    pub title: &'a str,
}

impl Identity {
    pub async fn all(db: &mut AsyncPgConnection) -> QueryResult<Vec<Self>> {
        use crate::schema::identity::dsl;
        dsl::identity
            .order_by(dsl::name.asc())
            .get_results(db)
            .await
    }
    pub async fn by_name(db: &mut AsyncPgConnection, name: &str) -> QueryResult<Self> {
        use crate::schema::identity::dsl;
        dsl::identity
            .filter(dsl::name.eq(name))
            .get_result(db)
            .await
    }

    pub async fn new(db: &mut AsyncPgConnection, name: &str, admin: bool) -> QueryResult<Self> {
        let newuser = NewIdentity { name, admin };
        use crate::schema::identity::dsl;
        diesel::insert_into(dsl::identity)
            .values(newuser)
            .get_result(db)
            .await
    }

    pub async fn tokens(&self, db: &mut AsyncPgConnection) -> QueryResult<Vec<Token>> {
        use crate::schema::token::dsl;
        dsl::token
            .filter(dsl::identity.eq(self.id))
            .get_results(db)
            .await
    }

    pub async fn new_token(&self, db: &mut AsyncPgConnection, title: &str) -> QueryResult<Token> {
        let newtoken = NewToken {
            identity: self.id,
            title,
        };
        use crate::schema::token::dsl;
        diesel::insert_into(dsl::token)
            .values(&newtoken)
            .get_result(db)
            .await
    }

    pub async fn delete_token(
        &self,
        db: &mut AsyncPgConnection,
        token: &str,
    ) -> QueryResult<usize> {
        use crate::schema::token::dsl;
        diesel::delete(dsl::token)
            .filter(dsl::content.eq(token))
            .execute(db)
            .await
    }
}
