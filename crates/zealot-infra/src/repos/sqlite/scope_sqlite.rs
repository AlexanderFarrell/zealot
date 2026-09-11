use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::str::FromStr;
use uuid::Uuid;
use zealot_app::repos::{
    common::RepoError,
    scope::{ScopeRepo, ServerMetadata},
};
use zealot_domain::scope::{
    PrincipalKind, PrincipalStatus, Scope, ScopeMember, ScopeMemberStatus, ScopeRole, ScopeStatus,
    ServerPrincipal,
};

#[derive(Debug)]
pub struct ScopeSqliteRepo {
    pool: SqlitePool,
}
impl ScopeSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
#[derive(sqlx::FromRow)]
struct PrincipalRow {
    principal_id: String,
    server_id: String,
    kind: String,
    account_id: Option<i64>,
    display_name: String,
    status: String,
    default_scope_id: Option<String>,
    created_at: i64,
    retired_at: Option<i64>,
}
#[derive(sqlx::FromRow)]
struct ScopeRow {
    scope_id: String,
    server_id: String,
    title: String,
    description: Option<String>,
    status: String,
    owner_principal_id: String,
    created_at: i64,
    updated_at: i64,
}
#[derive(sqlx::FromRow)]
struct MemberRow {
    scope_id: String,
    principal_id: String,
    role: String,
    status: String,
    created_at: i64,
    updated_at: i64,
}
fn uuid(value: String) -> Result<Uuid, RepoError> {
    Uuid::parse_str(&value).map_err(|e| RepoError::DatabaseError { err: e.to_string() })
}
fn time(value: i64) -> Result<DateTime<Utc>, RepoError> {
    DateTime::from_timestamp(value, 0).ok_or_else(|| RepoError::DatabaseError {
        err: format!("invalid timestamp: {value}"),
    })
}
fn principal(r: PrincipalRow) -> Result<ServerPrincipal, RepoError> {
    Ok(ServerPrincipal {
        principal_id: uuid(r.principal_id)?,
        server_id: uuid(r.server_id)?,
        kind: PrincipalKind::from_str(&r.kind).map_err(|err| RepoError::DatabaseError { err })?,
        account_id: r.account_id,
        display_name: r.display_name,
        status: PrincipalStatus::from_str(&r.status)
            .map_err(|err| RepoError::DatabaseError { err })?,
        default_scope_id: r.default_scope_id.map(uuid).transpose()?,
        created_at: time(r.created_at)?,
        retired_at: r.retired_at.map(time).transpose()?,
    })
}
fn scope(r: ScopeRow) -> Result<Scope, RepoError> {
    Ok(Scope {
        scope_id: uuid(r.scope_id)?,
        server_id: uuid(r.server_id)?,
        title: r.title,
        description: r.description,
        status: ScopeStatus::from_str(&r.status).map_err(|err| RepoError::DatabaseError { err })?,
        owner_principal_id: uuid(r.owner_principal_id)?,
        created_at: time(r.created_at)?,
        updated_at: time(r.updated_at)?,
    })
}
fn member(r: MemberRow) -> Result<ScopeMember, RepoError> {
    Ok(ScopeMember {
        scope_id: uuid(r.scope_id)?,
        principal_id: uuid(r.principal_id)?,
        role: ScopeRole::from_str(&r.role).map_err(|err| RepoError::DatabaseError { err })?,
        status: ScopeMemberStatus::from_str(&r.status)
            .map_err(|err| RepoError::DatabaseError { err })?,
        created_at: time(r.created_at)?,
        updated_at: time(r.updated_at)?,
    })
}
const P: &str = "principal_id, server_id, kind, account_id, display_name, status, default_scope_id, created_at, retired_at";
const S: &str =
    "scope_id, server_id, title, description, status, owner_principal_id, created_at, updated_at";
const M: &str = "scope_id, principal_id, role, status, created_at, updated_at";
impl ScopeRepo for ScopeSqliteRepo {
    fn server_metadata(&self) -> Result<ServerMetadata, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, (String, i64)>(
                    "SELECT server_id, schema_migration_version FROM server LIMIT 1",
                )
                .fetch_one(&p)
                .await
                .map(|(server_id, schema_migration_version)| ServerMetadata {
                    server_id,
                    schema_migration_version,
                })
                .map_err(RepoError::from)
            })
        })
    }
    fn human_principal_for_account(&self, a: i64) -> Result<Option<ServerPrincipal>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, PrincipalRow>(&format!(
                    "SELECT {P} FROM server_principal WHERE account_id=? AND kind='human'"
                ))
                .bind(a)
                .fetch_optional(&p)
                .await?
                .map(principal)
                .transpose()
            })
        })
    }
    fn principal_for_api_key_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<ServerPrincipal>, RepoError> {
        let p = self.pool.clone();
        let key_hash = key_hash.to_owned();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, PrincipalRow>(&format!(
                    "SELECT {P} FROM server_principal p JOIN api_key k ON k.principal_id=p.principal_id WHERE k.key_hash=? AND p.status='active'"
                ))
                .bind(key_hash)
                .fetch_optional(&p)
                .await?
                .map(principal)
                .transpose()
            })
        })
    }
    fn default_scope_for_account(&self, a: i64) -> Result<Option<Scope>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{sqlx::query_as::<_,ScopeRow>(&format!("SELECT {S} FROM scope sc JOIN server_principal p ON p.default_scope_id=sc.scope_id WHERE p.account_id=?")).bind(a).fetch_optional(&p).await?.map(scope).transpose()})
        })
    }
    fn default_scope_for_principal(&self, id: Uuid) -> Result<Option<Scope>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            sqlx::query_as::<_, ScopeRow>(&format!("SELECT {S} FROM scope sc JOIN server_principal p ON p.default_scope_id=sc.scope_id WHERE p.principal_id=?"))
                .bind(id.to_string()).fetch_optional(&p).await?.map(scope).transpose()
        })
        })
    }
    fn active_scopes_for_principal(&self, id: Uuid) -> Result<Vec<Scope>, RepoError> {
        let p = self.pool.clone();
        let id = id.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{sqlx::query_as::<_,ScopeRow>(&format!("SELECT {S} FROM scope sc JOIN scope_member sm ON sm.scope_id=sc.scope_id WHERE sm.principal_id=? AND sm.status='active' AND sc.status='active' ORDER BY sc.title,sc.scope_id")).bind(id).fetch_all(&p).await?.into_iter().map(scope).collect()})
        })
    }
    fn members_for_scope(&self, id: Uuid) -> Result<Vec<ScopeMember>, RepoError> {
        let p = self.pool.clone();
        let id = id.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, MemberRow>(&format!(
                    "SELECT {M} FROM scope_member WHERE scope_id=? ORDER BY created_at,principal_id"
                ))
                .bind(id)
                .fetch_all(&p)
                .await?
                .into_iter()
                .map(member)
                .collect()
            })
        })
    }
    fn create_service_principal(&self, n: &str) -> Result<ServerPrincipal, RepoError> {
        let p = self.pool.clone();
        let n = n.to_owned();
        let id = Uuid::new_v4().to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{principal(sqlx::query_as::<_,PrincipalRow>(&format!("INSERT INTO server_principal(principal_id,server_id,kind,display_name) SELECT ?,server_id,'service',? FROM server LIMIT 1 RETURNING {P}")).bind(id).bind(n).fetch_one(&p).await?)})
        })
    }
    fn service_principal_by_name(&self, n: &str) -> Result<Option<ServerPrincipal>, RepoError> {
        let p = self.pool.clone();
        let n = n.to_owned();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{sqlx::query_as::<_,PrincipalRow>(&format!("SELECT {P} FROM server_principal WHERE kind='service' AND display_name=? ORDER BY created_at LIMIT 1")).bind(n).fetch_optional(&p).await?.map(principal).transpose()})
        })
    }
    fn upsert_member(&self, s: Uuid, pid: Uuid, r: ScopeRole) -> Result<ScopeMember, RepoError> {
        let p = self.pool.clone();
        let s = s.to_string();
        let pid = pid.to_string();
        let r = r.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{member(sqlx::query_as::<_,MemberRow>(&format!("INSERT INTO scope_member(scope_id,principal_id,role,status) VALUES(?,?,?,'active') ON CONFLICT(scope_id,principal_id) DO UPDATE SET role=excluded.role,status='active',updated_at=strftime('%s','now') RETURNING {M}")).bind(s).bind(pid).bind(r).fetch_one(&p).await?)})
        })
    }
}
