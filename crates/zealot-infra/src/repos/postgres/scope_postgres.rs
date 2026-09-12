use chrono::{DateTime, Utc};
use sqlx::PgPool;
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
pub struct ScopePostgresRepo {
    pool: PgPool,
}
impl ScopePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[derive(sqlx::FromRow)]
struct PR {
    principal_id: Uuid,
    server_id: Uuid,
    kind: String,
    account_id: Option<i32>,
    display_name: String,
    status: String,
    default_scope_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    retired_at: Option<DateTime<Utc>>,
}
#[derive(sqlx::FromRow)]
struct SR {
    scope_id: Uuid,
    server_id: Uuid,
    title: String,
    description: Option<String>,
    status: String,
    owner_principal_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
#[derive(sqlx::FromRow)]
struct MR {
    scope_id: Uuid,
    principal_id: Uuid,
    role: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
fn pr(r: PR) -> Result<ServerPrincipal, RepoError> {
    Ok(ServerPrincipal {
        principal_id: r.principal_id,
        server_id: r.server_id,
        kind: PrincipalKind::from_str(&r.kind).map_err(|err| RepoError::DatabaseError { err })?,
        account_id: r.account_id.map(i64::from),
        display_name: r.display_name,
        status: PrincipalStatus::from_str(&r.status)
            .map_err(|err| RepoError::DatabaseError { err })?,
        default_scope_id: r.default_scope_id,
        created_at: r.created_at,
        retired_at: r.retired_at,
    })
}
fn sc(r: SR) -> Result<Scope, RepoError> {
    Ok(Scope {
        scope_id: r.scope_id,
        server_id: r.server_id,
        title: r.title,
        description: r.description,
        status: ScopeStatus::from_str(&r.status).map_err(|err| RepoError::DatabaseError { err })?,
        owner_principal_id: r.owner_principal_id,
        created_at: r.created_at,
        updated_at: r.updated_at,
    })
}
fn mb(r: MR) -> Result<ScopeMember, RepoError> {
    Ok(ScopeMember {
        scope_id: r.scope_id,
        principal_id: r.principal_id,
        role: ScopeRole::from_str(&r.role).map_err(|err| RepoError::DatabaseError { err })?,
        status: ScopeMemberStatus::from_str(&r.status)
            .map_err(|err| RepoError::DatabaseError { err })?,
        created_at: r.created_at,
        updated_at: r.updated_at,
    })
}
const P: &str = "principal_id,server_id,kind,account_id,display_name,status,default_scope_id,created_at,retired_at";
const P_JOINED: &str = "p.principal_id,p.server_id,p.kind,p.account_id,p.display_name,p.status,p.default_scope_id,p.created_at,p.retired_at";
const S: &str =
    "sc.scope_id,sc.server_id,sc.title,sc.description,sc.status,sc.owner_principal_id,sc.created_at,sc.updated_at";
const M: &str = "scope_id,principal_id,role,status,created_at,updated_at";
impl ScopeRepo for ScopePostgresRepo {
    fn server_metadata(&self) -> Result<ServerMetadata, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, (Uuid, i64)>(
                    "SELECT server_id,schema_migration_version FROM server LIMIT 1",
                )
                .fetch_one(&p)
                .await
                .map(|(server_id, schema_migration_version)| ServerMetadata {
                    server_id: server_id.to_string(),
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
                sqlx::query_as::<_, PR>(&format!(
                    "SELECT {P} FROM server_principal WHERE account_id=$1 AND kind='human'"
                ))
                .bind(a)
                .fetch_optional(&p)
                .await?
                .map(pr)
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
                sqlx::query_as::<_, PR>(&format!(
                    "SELECT {P_JOINED} FROM server_principal p JOIN api_key k ON k.principal_id=p.principal_id WHERE k.key_hash=$1 AND p.status='active'"
                ))
                .bind(key_hash).fetch_optional(&p).await?.map(pr).transpose()
            })
        })
    }
    fn principal_by_id(&self, id: Uuid) -> Result<Option<ServerPrincipal>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, PR>(&format!(
                    "SELECT {P} FROM server_principal WHERE principal_id=$1"
                ))
                .bind(id)
                .fetch_optional(&p)
                .await?
                .map(pr)
                .transpose()
            })
        })
    }
    fn default_scope_for_account(&self, a: i64) -> Result<Option<Scope>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{sqlx::query_as::<_,SR>(&format!("SELECT {S} FROM scope sc JOIN server_principal p ON p.default_scope_id=sc.scope_id WHERE p.account_id=$1")).bind(a).fetch_optional(&p).await?.map(sc).transpose()})
        })
    }
    fn default_scope_for_principal(&self, id: Uuid) -> Result<Option<Scope>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            sqlx::query_as::<_, SR>(&format!("SELECT {S} FROM scope sc JOIN server_principal p ON p.default_scope_id=sc.scope_id WHERE p.principal_id=$1"))
                .bind(id).fetch_optional(&p).await?.map(sc).transpose()
        })
        })
    }
    fn scope_by_id(&self, id: Uuid) -> Result<Option<Scope>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, SR>(&format!("SELECT {S} FROM scope sc WHERE sc.scope_id=$1"))
                    .bind(id)
                    .fetch_optional(&p)
                    .await?
                    .map(sc)
                    .transpose()
            })
        })
    }
    fn active_scopes_for_principal(&self, id: Uuid) -> Result<Vec<Scope>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{sqlx::query_as::<_,SR>(&format!("SELECT {S} FROM scope sc JOIN scope_member sm ON sm.scope_id=sc.scope_id WHERE sm.principal_id=$1 AND sm.status='active' AND sc.status='active' ORDER BY sc.title,sc.scope_id")).bind(id).fetch_all(&p).await?.into_iter().map(sc).collect()})
        })
    }
    fn members_for_scope(&self, id: Uuid) -> Result<Vec<ScopeMember>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{sqlx::query_as::<_,MR>(&format!("SELECT {M} FROM scope_member WHERE scope_id=$1 ORDER BY created_at,principal_id")).bind(id).fetch_all(&p).await?.into_iter().map(mb).collect()})
        })
    }
    fn create_service_principal(&self, n: &str) -> Result<ServerPrincipal, RepoError> {
        let p = self.pool.clone();
        let n = n.to_owned();
        let id = Uuid::new_v4();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{pr(sqlx::query_as::<_,PR>(&format!("INSERT INTO server_principal(principal_id,server_id,kind,display_name) SELECT $1,server_id,'service',$2 FROM server LIMIT 1 RETURNING {P}")).bind(id).bind(n).fetch_one(&p).await?)})
        })
    }
    fn service_principal_by_name(&self, n: &str) -> Result<Option<ServerPrincipal>, RepoError> {
        let p = self.pool.clone();
        let n = n.to_owned();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{sqlx::query_as::<_,PR>(&format!("SELECT {P} FROM server_principal WHERE kind='service' AND display_name=$1 ORDER BY created_at LIMIT 1")).bind(n).fetch_optional(&p).await?.map(pr).transpose()})
        })
    }
    fn upsert_member(&self, s: Uuid, pid: Uuid, r: ScopeRole) -> Result<ScopeMember, RepoError> {
        let p = self.pool.clone();
        let r = r.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move{mb(sqlx::query_as::<_,MR>(&format!("INSERT INTO scope_member(scope_id,principal_id,role,status) VALUES($1,$2,$3,'active') ON CONFLICT(scope_id,principal_id) DO UPDATE SET role=excluded.role,status='active',updated_at=now() RETURNING {M}")).bind(s).bind(pid).bind(r).fetch_one(&p).await?)})
        })
    }
}
