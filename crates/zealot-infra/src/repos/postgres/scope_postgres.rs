use chrono::{DateTime, Utc};
use sqlx::PgPool;
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
#[derive(sqlx::FromRow)]
struct IR {
    invitation_id: Uuid,
    scope_id: Uuid,
    issuer_principal_id: Uuid,
    recipient_principal_id: Option<Uuid>,
    permitted_role: String,
    token_hash: String,
    token_signature: String,
    status: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    accepted_at: Option<DateTime<Utc>>,
    terminal_at: Option<DateTime<Utc>>,
    correlation_id: Uuid,
}
#[derive(sqlx::FromRow)]
struct ER {
    event_id: Uuid,
    scope_id: Uuid,
    actor_principal_id: Option<Uuid>,
    subject_principal_id: Option<Uuid>,
    invitation_id: Option<Uuid>,
    event_type: String,
    outcome: String,
    reason_code: String,
    correlation_id: Uuid,
    occurred_at: DateTime<Utc>,
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
fn invitation(r: IR) -> Result<ScopeInvitation, RepoError> {
    Ok(ScopeInvitation {
        invitation_id: r.invitation_id,
        scope_id: r.scope_id,
        issuer_principal_id: r.issuer_principal_id,
        recipient_principal_id: r.recipient_principal_id,
        permitted_role: ScopeRole::from_str(&r.permitted_role)
            .map_err(|err| RepoError::DatabaseError { err })?,
        token_hash: r.token_hash,
        token_signature: r.token_signature,
        status: ScopeInvitationStatus::from_str(&r.status)
            .map_err(|err| RepoError::DatabaseError { err })?,
        created_at: r.created_at,
        expires_at: r.expires_at,
        accepted_at: r.accepted_at,
        terminal_at: r.terminal_at,
        correlation_id: r.correlation_id,
    })
}
fn event(r: ER) -> Result<ScopeLifecycleEvent, RepoError> {
    Ok(ScopeLifecycleEvent {
        event_id: r.event_id,
        scope_id: r.scope_id,
        actor_principal_id: r.actor_principal_id,
        subject_principal_id: r.subject_principal_id,
        invitation_id: r.invitation_id,
        event_type: r.event_type,
        outcome: r.outcome,
        reason_code: r.reason_code,
        correlation_id: r.correlation_id,
        occurred_at: r.occurred_at,
    })
}
const P: &str = "principal_id,server_id,kind,account_id,display_name,status,default_scope_id,created_at,retired_at";
const P_JOINED: &str = "p.principal_id,p.server_id,p.kind,p.account_id,p.display_name,p.status,p.default_scope_id,p.created_at,p.retired_at";
const S: &str = "sc.scope_id,sc.server_id,sc.title,sc.description,sc.status,sc.owner_principal_id,sc.created_at,sc.updated_at";
const M: &str = "scope_id,principal_id,role,status,created_at,updated_at";
const I: &str = "invitation_id,scope_id,issuer_principal_id,recipient_principal_id,permitted_role,token_hash,token_signature,status,created_at,expires_at,accepted_at,terminal_at,correlation_id";
const E: &str = "event_id,scope_id,actor_principal_id,subject_principal_id,invitation_id,event_type,outcome,reason_code,correlation_id,occurred_at";
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
                    "UPDATE server SET invitation_signing_key=$1 WHERE invitation_signing_key IS NULL",
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
                sqlx::query_as::<_, IR>(&format!(
                    "INSERT INTO scope_invitation ({I}) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) RETURNING {I}"
                ))
                .bind(value.invitation_id)
                .bind(value.scope_id)
                .bind(value.issuer_principal_id)
                .bind(value.recipient_principal_id)
                .bind(value.permitted_role.to_string())
                .bind(value.token_hash)
                .bind(value.token_signature)
                .bind(value.status.to_string())
                .bind(value.created_at)
                .bind(value.expires_at)
                .bind(value.accepted_at)
                .bind(value.terminal_at)
                .bind(value.correlation_id)
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
                sqlx::query_as::<_, IR>(&format!(
                    "SELECT {I} FROM scope_invitation WHERE token_hash=$1"
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
                sqlx::query_as::<_, IR>(&format!(
                    "SELECT {I} FROM scope_invitation WHERE scope_id=$1 ORDER BY created_at DESC, invitation_id"
                ))
                .bind(id)
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
                    sqlx::query_as::<_, IR>(&format!(
                        "UPDATE scope_invitation SET status=$1,terminal_at=$2 WHERE invitation_id=$3 AND status IN ('pending','pending_admission') RETURNING {I}"
                    ))
                    .bind(status)
                    .bind(when)
                    .bind(id)
                    .fetch_optional(&p)
                    .await?
                } else {
                    sqlx::query_as::<_, IR>(&format!(
                        "UPDATE scope_invitation SET status=$1,terminal_at=NULL WHERE invitation_id=$2 AND status IN ('pending','pending_admission') RETURNING {I}"
                    ))
                    .bind(status)
                    .bind(id)
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
                let row = sqlx::query_as::<_, IR>(&format!(
                    "SELECT {I} FROM scope_invitation WHERE invitation_id=$1 AND status IN ('pending','pending_admission') AND expires_at>$2 AND EXISTS (SELECT 1 FROM scope sc WHERE sc.scope_id=scope_invitation.scope_id AND sc.status='active') AND (recipient_principal_id IS NULL OR recipient_principal_id=$3) FOR UPDATE"
                ))
                .bind(id)
                .bind(when)
                .bind(principal_id)
                .fetch_optional(&mut *tx)
                .await?;
                let Some(row) = row else {
                    tx.rollback().await?;
                    return Ok(None);
                };
                let member_row = sqlx::query_as::<_, MR>(&format!(
                    "INSERT INTO scope_member(scope_id,principal_id,role,status) VALUES($1,$2,$3,'active') ON CONFLICT(scope_id,principal_id) DO UPDATE SET role=EXCLUDED.role,status='active',updated_at=$4 WHERE NOT (scope_member.role='owner' AND EXCLUDED.role <> 'owner' AND NOT EXISTS (SELECT 1 FROM scope_member other WHERE other.scope_id=scope_member.scope_id AND other.status='active' AND other.role='owner' AND other.principal_id<>scope_member.principal_id)) RETURNING {M}"
                ))
                .bind(row.scope_id)
                .bind(principal_id)
                .bind(row.permitted_role)
                .bind(when)
                .fetch_optional(&mut *tx)
                .await?;
                let Some(member_row) = member_row else {
                    tx.rollback().await?;
                    return Ok(None);
                };
                sqlx::query("UPDATE scope_invitation SET status='active',accepted_at=$1,terminal_at=$2 WHERE invitation_id=$3")
                    .bind(when)
                    .bind(when)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                mb(member_row).map(Some)
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
                sqlx::query_as::<_, MR>(&format!(
                    "UPDATE scope_member SET role=$1,updated_at=$2 WHERE scope_id=$3 AND principal_id=$4 AND status='active' AND NOT (role='owner' AND $1 <> 'owner' AND NOT EXISTS (SELECT 1 FROM scope_member other WHERE other.scope_id=scope_member.scope_id AND other.status='active' AND other.role='owner' AND other.principal_id<>scope_member.principal_id)) RETURNING {M}"
                ))
                .bind(role.to_string())
                .bind(when)
                .bind(scope_id)
                .bind(principal_id)
                .fetch_optional(&p)
                .await?
                .map(mb)
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
                sqlx::query_as::<_, MR>(&format!(
                    "UPDATE scope_member SET status='revoked',updated_at=$1 WHERE scope_id=$2 AND principal_id=$3 AND status='active' AND NOT (role='owner' AND NOT EXISTS (SELECT 1 FROM scope_member other WHERE other.scope_id=scope_member.scope_id AND other.status='active' AND other.role='owner' AND other.principal_id<>scope_member.principal_id)) RETURNING {M}"
                ))
                .bind(when)
                .bind(scope_id)
                .bind(principal_id)
                .fetch_optional(&p)
                .await?
                .map(mb)
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
                    "INSERT INTO scope_lifecycle_event ({E}) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
                ))
                .bind(value.event_id)
                .bind(value.scope_id)
                .bind(value.actor_principal_id)
                .bind(value.subject_principal_id)
                .bind(value.invitation_id)
                .bind(value.event_type)
                .bind(value.outcome)
                .bind(value.reason_code)
                .bind(value.correlation_id)
                .bind(value.occurred_at)
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
                sqlx::query_as::<_, ER>(&format!(
                    "SELECT {E} FROM scope_lifecycle_event WHERE scope_id=$1 ORDER BY occurred_at DESC,event_id"
                ))
                .bind(id)
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
