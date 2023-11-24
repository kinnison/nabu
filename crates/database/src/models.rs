//! Core model functionality for the Nabu database
//!

use diesel::{ExpressionMethods, Insertable, OptionalExtension, QueryDsl, QueryResult, Queryable};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use metadata::index::Entry;
use semver::{Version, VersionReq};

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
    pub async fn by_name(db: &mut AsyncPgConnection, name: &str) -> QueryResult<Option<Self>> {
        use crate::schema::identity::dsl;
        dsl::identity
            .filter(dsl::name.eq(name))
            .get_result(db)
            .await
            .optional()
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

impl Token {
    pub async fn from_token(db: &mut AsyncPgConnection, token: &str) -> QueryResult<Option<Self>> {
        use crate::schema::token::dsl;
        dsl::token
            .filter(dsl::content.eq(token))
            .get_result(db)
            .await
            .optional()
    }

    pub async fn owner(&self, db: &mut AsyncPgConnection) -> QueryResult<Identity> {
        use crate::schema::identity::dsl;
        dsl::identity
            .filter(dsl::id.eq(self.identity))
            .get_result(db)
            .await
    }
}

#[derive(Debug, Queryable)]
pub struct Krate {
    pub id: i32,
    pub name: String,
    pub owner: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name=crate::schema::krate)]
pub struct NewKrate<'a> {
    pub name: &'a str,
    pub owner: i32,
}

#[derive(Debug, Queryable)]
pub struct KrateVer {
    pub id: i32,
    pub krate: i32,
    pub exposed: bool,
    pub ver: String,
    pub yanked: bool,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Insertable)]
#[diesel(table_name=crate::schema::kratever)]
pub struct NewKrateVer<'a> {
    pub krate: i32,
    pub exposed: bool,
    pub ver: &'a str,
    pub yanked: bool,
    pub metadata: serde_json::Value,
}

impl Krate {
    pub async fn by_name(db: &mut AsyncPgConnection, name: &str) -> QueryResult<Option<Self>> {
        use crate::schema::krate::dsl;
        dsl::krate
            .filter(dsl::name.eq(name))
            .get_result(db)
            .await
            .optional()
    }

    pub async fn by_name_or_new(
        db: &mut AsyncPgConnection,
        name: &str,
        owner: &Identity,
    ) -> QueryResult<Self> {
        db.build_transaction()
            .run(|db| {
                Box::pin(async move {
                    if let Some(krate) = Krate::by_name(db, name).await? {
                        Ok(krate)
                    } else {
                        Krate::new(db, name, owner).await
                    }
                })
            })
            .await
    }

    pub async fn new(
        db: &mut AsyncPgConnection,
        name: &str,
        owner: &Identity,
    ) -> QueryResult<Self> {
        use crate::schema::krate::dsl;
        let newkrate = NewKrate {
            name,
            owner: owner.id,
        };
        diesel::insert_into(dsl::krate)
            .values(newkrate)
            .get_result(db)
            .await
    }

    pub async fn new_version(
        &self,
        db: &mut AsyncPgConnection,
        entry: &Entry,
    ) -> QueryResult<KrateVer> {
        use crate::schema::kratever::dsl;
        let newver = NewKrateVer {
            krate: self.id,
            exposed: true,
            ver: &entry.vers,
            yanked: entry.yanked,
            metadata: serde_json::to_value(entry)
                .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?,
        };
        diesel::insert_into(dsl::kratever)
            .values(newver)
            .get_result(db)
            .await
    }

    pub async fn satisfies(&self, db: &mut AsyncPgConnection, req: &str) -> QueryResult<bool> {
        use crate::schema::kratever::dsl;
        let versions: Vec<String> = dsl::kratever
            .select(dsl::ver)
            .filter(dsl::krate.eq(self.id))
            .load(db)
            .await?;
        let req = VersionReq::parse(req)
            .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?;
        Ok(versions
            .into_iter()
            .flat_map(|ver| Version::parse(&ver).ok())
            .any(move |ver| req.matches(&ver)))
    }

    pub async fn versions(&self, db: &mut AsyncPgConnection) -> QueryResult<Vec<KrateVer>> {
        use crate::schema::kratever::dsl;
        dsl::kratever
            .filter(dsl::krate.eq(self.id))
            .order_by(dsl::id.asc())
            .get_results(db)
            .await
    }
}

impl KrateVer {
    pub fn index_line(&self) -> String {
        serde_json::to_string(&self.metadata).expect("Unable to re-serialise valid JSON")
    }
}
