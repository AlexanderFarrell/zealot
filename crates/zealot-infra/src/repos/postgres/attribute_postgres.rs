use sqlx::PgPool;
use zealot_app::repos::attribute::AttributeRepo;

#[derive(Debug)]
pub struct AttributePostgresRepo {
    pool: PgPool,
}

impl AttributePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AttributeRepo for AttributePostgresRepo {
    fn get_attribute_kind(
        &self,
        key: &str,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::attribute::AttributeKind>, zealot_app::repos::common::RepoError>
    {
        todo!()
    }

    fn get_attribute_kind_by_id(
        &self,
        id: &zealot_domain::common::id::Id,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::attribute::AttributeKind>, zealot_app::repos::common::RepoError>
    {
        todo!()
    }

    fn get_attribute_kinds_for_user(
        &self,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<
        std::collections::HashMap<String, zealot_domain::attribute::AttributeKind>,
        zealot_app::repos::common::RepoError,
    > {
        todo!()
    }

    fn add_attribute_kind(
        &self,
        dto: &zealot_domain::attribute::AddAttributeKindDto,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::attribute::AttributeKind>, zealot_app::repos::common::RepoError>
    {
        todo!()
    }

    fn update_attribute_kind(
        &self,
        dto: &zealot_domain::attribute::UpdateAttributeKindDto,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::attribute::AttributeKind>, zealot_app::repos::common::RepoError>
    {
        todo!()
    }

    fn delete_attribute_kind(
        &self,
        key: &str,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}
