//! SeaORM Entity. Generated by sea-orm-codegen 0.9.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "record")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub zone: String,
    pub value: Vec<u8>,
    pub active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::zone::Entity",
        from = "Column::Zone",
        to = "super::zone::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Zone,
    #[sea_orm(has_many = "super::proxy::Entity")]
    Proxy,
}

impl Related<super::zone::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Zone.def()
    }
}

impl Related<super::proxy::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Proxy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}