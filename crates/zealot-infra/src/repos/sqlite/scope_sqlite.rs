use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::str::FromStr;
use uuid::Uuid;
use zealot_app::repos::{
    common::RepoError,
    scope::{ScopeRepo, ServerMetadata},
};
use zealot_domain::scope::{
    PrincipalKind, PrincipalStatus, Scope, ScopeInvitation, ScopeInvitationStatus,
    ScopeLifecycleEvent, ScopeMember, ScopeMemberStatus, ScopeRole, ScopeStatus, ServerPrincipal,
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
#[derive(sqlx::FromRow)]
struct InvitationRow {
    invitation_id: String,
    scope_id: String,
    issuer_principal_id: String,
    recipient_principal_id: Option<String>,
    permitted_role: String,
    token_hash: String,
    token_signature: String,
    status: String,
    created_at: i64,
    expires_at: i64,
    accepted_at: Option<i64>,
    terminal_at: Option<i64>,
    correlation_id: String,
}
#[derive(sqlx::FromRow)]
struct EventRow {
    event_id: String,
    scope_id: String,
    actor_principal_id: Option<String>,
    subject_principal_id: Option<String>,
    invitation_id: Option<String>,
    event_type: String,
    outcome: String,
    reason_code: String,
    correlation_id: String,
    occurred_at: i64,
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
fn invitation(r: InvitationRow) -> Result<ScopeInvitation, RepoError> {
    Ok(ScopeInvitation {
        invitation_id: uuid(r.invitation_id)?,
        scope_id: uuid(r.scope_id)?,
        issuer_principal_id: uuid(r.issuer_principal_id)?,
        recipient_principal_id: r.recipient_principal_id.map(uuid).transpose()?,
        permitted_role: ScopeRole::from_str(&r.permitted_role)
            .map_err(|err| RepoError::DatabaseError { err })?,
        token_hash: r.token_hash,
        token_signature: r.token_signature,
        status: ScopeInvitationStatus::from_str(&r.status)
            .map_err(|err| RepoError::DatabaseError { err })?,
        created_at: time(r.created_at)?,
        expires_at: time(r.expires_at)?,
        accepted_at: r.accepted_at.map(time).transpose()?,
        terminal_at: r.terminal_at.map(time).transpose()?,
        correlation_id: uuid(r.correlation_id)?,
    })
}
fn event(r: EventRow) -> Result<ScopeLifecycleEvent, RepoError> {
    Ok(ScopeLifecycleEvent {
        event_id: uuid(r.event_id)?,
        scope_id: uuid(r.scope_id)?,
        actor_principal_id: r.actor_principal_id.map(uuid).transpose()?,
        subject_principal_id: r.subject_principal_id.map(uuid).transpose()?,
        invitation_id: r.invitation_id.map(uuid).transpose()?,
        event_type: r.event_type,
        outcome: r.outcome,
        reason_code: r.reason_code,
        correlation_id: uuid(r.correlation_id)?,
        occurred_at: time(r.occurred_at)?,
    })
}
const P: &str = "principal_id, server_id, kind, account_id, display_name, status, default_scope_id, created_at, retired_at";
const P_JOINED: &str = "p.principal_id, p.server_id, p.kind, p.account_id, p.display_name, p.status, p.default_scope_id, p.created_at, p.retired_at";
const S: &str = "sc.scope_id, sc.server_id, sc.title, sc.description, sc.status, sc.owner_principal_id, sc.created_at, sc.updated_at";
const M: &str = "scope_id, principal_id, role, status, created_at, updated_at";
const I: &str = "invitation_id,scope_id,issuer_principal_id,recipient_principal_id,permitted_role,token_hash,token_signature,status,created_at,expires_at,accepted_at,terminal_at,correlation_id";
const E: &str = "event_id,scope_id,actor_principal_id,subject_principal_id,invitation_id,event_type,outcome,reason_code,correlation_id,occurred_at";
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
                    "SELECT {P_JOINED} FROM server_principal p JOIN api_key k ON k.principal_id=p.principal_id WHERE k.key_hash=? AND p.status='active'"
                ))
                .bind(key_hash)
                .fetch_optional(&p)
                .await?
                .map(principal)
                .transpose()
            })
        })
    }
    fn principal_by_id(&self, id: Uuid) -> Result<Option<ServerPrincipal>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, PrincipalRow>(&format!(
                    "SELECT {P} FROM server_principal WHERE principal_id=?"
                ))
                .bind(id.to_string())
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
    fn scope_by_id(&self, id: Uuid) -> Result<Option<Scope>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, ScopeRow>(&format!(
                    "SELECT {S} FROM scope sc WHERE sc.scope_id=?"
                ))
                .bind(id.to_string())
                .fetch_optional(&p)
                .await?
                .map(scope)
                .transpose()
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
    fn server_enrolment_policy(&self) -> Result<String, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_scalar("SELECT enrolment_policy FROM server LIMIT 1")
                    .fetch_one(&p)
                    .await
                    .map_err(RepoError::from)
            })
        })
    }
    fn server_invitation_signing_key(&self) -> Result<Option<String>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_scalar::<_, Option<String>>(
                    "SELECT invitation_signing_key FROM server LIMIT 1",
                )
                .fetch_one(&p)
                .await
                .map_err(RepoError::from)
            })
        })
    }
    fn set_server_invitation_signing_key(&self, key: &str) -> Result<String, RepoError> {
        let p = self.pool.clone();
        let key = key.to_owned();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE server SET invitation_signing_key=? WHERE invitation_signing_key IS NULL",
                )
                .bind(key)
                .execute(&p)
                .await?;
                sqlx::query_scalar::<_, String>(
                    "SELECT invitation_signing_key FROM server WHERE invitation_signing_key IS NOT NULL LIMIT 1",
                )
                .fetch_one(&p)
                .await
                .map_err(RepoError::from)
            })
        })
    }
    fn create_invitation(&self, value: &ScopeInvitation) -> Result<ScopeInvitation, RepoError> {
        let p = self.pool.clone();
        let value = value.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, InvitationRow>(&format!(
                    "INSERT INTO scope_invitation ({I}) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?) RETURNING {I}"
                ))
                .bind(value.invitation_id.to_string())
                .bind(value.scope_id.to_string())
                .bind(value.issuer_principal_id.to_string())
                .bind(value.recipient_principal_id.map(|id| id.to_string()))
                .bind(value.permitted_role.to_string())
                .bind(value.token_hash)
                .bind(value.token_signature)
                .bind(value.status.to_string())
                .bind(value.created_at.timestamp())
                .bind(value.expires_at.timestamp())
                .bind(value.accepted_at.map(|at| at.timestamp()))
                .bind(value.terminal_at.map(|at| at.timestamp()))
                .bind(value.correlation_id.to_string())
                .fetch_one(&p)
                .await
                .map(invitation)
                .map_err(RepoError::from)?
            })
        })
    }
    fn invitation_by_token_hash(&self, hash: &str) -> Result<Option<ScopeInvitation>, RepoError> {
        let p = self.pool.clone();
        let hash = hash.to_owned();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, InvitationRow>(&format!(
                    "SELECT {I} FROM scope_invitation WHERE token_hash=?"
                ))
                .bind(hash)
                .fetch_optional(&p)
                .await?
                .map(invitation)
                .transpose()
            })
        })
    }
    fn invitations_for_scope(&self, id: Uuid) -> Result<Vec<ScopeInvitation>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, InvitationRow>(&format!(
                    "SELECT {I} FROM scope_invitation WHERE scope_id=? ORDER BY created_at DESC, invitation_id"
                ))
                .bind(id.to_string())
                .fetch_all(&p)
                .await?
                .into_iter()
                .map(invitation)
                .collect()
            })
        })
    }
    fn transition_invitation(
        &self,
        id: Uuid,
        status: ScopeInvitationStatus,
        when: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<ScopeInvitation>, RepoError> {
        let p = self.pool.clone();
        let status = status.to_string();
        let terminal = !matches!(
            status.parse::<ScopeInvitationStatus>(),
            Ok(ScopeInvitationStatus::Pending | ScopeInvitationStatus::PendingAdmission)
        );
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = if terminal {
                    sqlx::query_as::<_, InvitationRow>(&format!(
                        "UPDATE scope_invitation SET status=?,terminal_at=? WHERE invitation_id=? AND status IN ('pending','pending_admission') RETURNING {I}"
                    ))
                    .bind(status)
                    .bind(when.timestamp())
                    .bind(id.to_string())
                    .fetch_optional(&p)
                    .await?
                } else {
                    sqlx::query_as::<_, InvitationRow>(&format!(
                        "UPDATE scope_invitation SET status=?,terminal_at=NULL WHERE invitation_id=? AND status IN ('pending','pending_admission') RETURNING {I}"
                    ))
                    .bind(status)
                    .bind(id.to_string())
                    .fetch_optional(&p)
                    .await?
                };
                row.map(invitation).transpose()
            })
        })
    }
    fn activate_invitation(
        &self,
        id: Uuid,
        principal_id: Uuid,
        when: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<ScopeMember>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = p.begin().await?;
                let row = sqlx::query_as::<_, InvitationRow>(&format!(
                    "SELECT {I} FROM scope_invitation WHERE invitation_id=? AND status IN ('pending','pending_admission') AND expires_at>? AND EXISTS (SELECT 1 FROM scope sc WHERE sc.scope_id=scope_invitation.scope_id AND sc.status='active') AND (recipient_principal_id IS NULL OR recipient_principal_id=?)"
                ))
                .bind(id.to_string())
                .bind(when.timestamp())
                .bind(principal_id.to_string())
                .fetch_optional(&mut *tx)
                .await?;
                let Some(row) = row else {
                    tx.rollback().await?;
                    return Ok(None);
                };
                let member_row = sqlx::query_as::<_, MemberRow>(&format!(
                    "INSERT INTO scope_member(scope_id,principal_id,role,status) VALUES(?,?,?,'active') ON CONFLICT(scope_id,principal_id) DO UPDATE SET role=excluded.role,status='active',updated_at=? WHERE NOT (scope_member.role='owner' AND excluded.role <> 'owner' AND NOT EXISTS (SELECT 1 FROM scope_member other WHERE other.scope_id=scope_member.scope_id AND other.status='active' AND other.role='owner' AND other.principal_id<>scope_member.principal_id)) RETURNING {M}"
                ))
                .bind(row.scope_id.clone())
                .bind(principal_id.to_string())
                .bind(row.permitted_role.clone())
                .bind(when.timestamp())
                .fetch_optional(&mut *tx)
                .await?;
                let Some(member_row) = member_row else {
                    tx.rollback().await?;
                    return Ok(None);
                };
                sqlx::query("UPDATE scope_invitation SET status='active',accepted_at=?,terminal_at=? WHERE invitation_id=?")
                    .bind(when.timestamp())
                    .bind(when.timestamp())
                    .bind(id.to_string())
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                member(member_row).map(Some)
            })
        })
    }
    fn change_member_role(
        &self,
        scope_id: Uuid,
        principal_id: Uuid,
        role: ScopeRole,
        when: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<ScopeMember>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, MemberRow>(&format!(
                    "UPDATE scope_member SET role=?,updated_at=? WHERE scope_id=? AND principal_id=? AND status='active' AND NOT (role='owner' AND ? <> 'owner' AND NOT EXISTS (SELECT 1 FROM scope_member other WHERE other.scope_id=scope_member.scope_id AND other.status='active' AND other.role='owner' AND other.principal_id<>scope_member.principal_id)) RETURNING {M}"
                ))
                .bind(role.to_string())
                .bind(when.timestamp())
                .bind(scope_id.to_string())
                .bind(principal_id.to_string())
                .bind(role.to_string())
                .fetch_optional(&p)
                .await?
                .map(member)
                .transpose()
            })
        })
    }
    fn revoke_member(
        &self,
        scope_id: Uuid,
        principal_id: Uuid,
        when: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<ScopeMember>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, MemberRow>(&format!(
                    "UPDATE scope_member SET status='revoked',updated_at=? WHERE scope_id=? AND principal_id=? AND status='active' AND NOT (role='owner' AND NOT EXISTS (SELECT 1 FROM scope_member other WHERE other.scope_id=scope_member.scope_id AND other.status='active' AND other.role='owner' AND other.principal_id<>scope_member.principal_id)) RETURNING {M}"
                ))
                .bind(when.timestamp())
                .bind(scope_id.to_string())
                .bind(principal_id.to_string())
                .fetch_optional(&p)
                .await?
                .map(member)
                .transpose()
            })
        })
    }
    fn append_lifecycle_event(&self, value: &ScopeLifecycleEvent) -> Result<(), RepoError> {
        let p = self.pool.clone();
        let value = value.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(&format!(
                    "INSERT INTO scope_lifecycle_event ({E}) VALUES (?,?,?,?,?,?,?,?,?,?)"
                ))
                .bind(value.event_id.to_string())
                .bind(value.scope_id.to_string())
                .bind(value.actor_principal_id.map(|id| id.to_string()))
                .bind(value.subject_principal_id.map(|id| id.to_string()))
                .bind(value.invitation_id.map(|id| id.to_string()))
                .bind(value.event_type)
                .bind(value.outcome)
                .bind(value.reason_code)
                .bind(value.correlation_id.to_string())
                .bind(value.occurred_at.timestamp())
                .execute(&p)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
    fn lifecycle_events_for_scope(&self, id: Uuid) -> Result<Vec<ScopeLifecycleEvent>, RepoError> {
        let p = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, EventRow>(&format!(
                    "SELECT {E} FROM scope_lifecycle_event WHERE scope_id=? ORDER BY occurred_at DESC,event_id"
                ))
                .bind(id.to_string())
                .fetch_all(&p)
                .await?
                .into_iter()
                .map(event)
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
