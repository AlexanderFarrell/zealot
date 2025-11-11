use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use rusqlite::ToSql;
use time::{Date, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Item {
    pub id: u32,
    pub title: String,
    pub kind: Option<String>,
    pub icon: Option<String>,
    pub content: String,
    pub status: Option<String>,

    // Figure out dates
    pub date: Option<Date>,
    pub created_on: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
    // pub date: i32,
    // pub created_on
    // pub updated_at

    pub week: Option<u32>,
    pub year: Option<u32>,
    pub parent_id: Option<u32>,
    pub parent_title: Option<String>,
}

pub struct ItemDBO;
const ITEM_SELECT_FROM: &str = r#"
SELECT i.item_id, i.title, i.kind, i.icon, i.content, 
i.status, i.date, i.created_on, i.updated_at, i.week, i.year,
p.item_id as parent_id,
p.title as parent_title
FROM item i
left join item p on p.item_id = i.parent_id"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddInfo {
    title: String,
}

impl ItemDBO {
    pub async fn get_by_id(item_id: u32) -> Result<Option<Item>, String> {
        ItemDBO::scan_row(
            format!("{ITEM_SELECT_FROM} WHERE i.item_id=(?1);").as_str(), 
            &[&item_id])
            .await
    }


    pub async fn get_recent(page: u32, max: u32) -> Result<Vec<Item>, String> {
        let offset = (page - 1) * max;
        ItemDBO::scan_rows(
            format!("{ITEM_SELECT_FROM} ORDER BY i.created_on DESC LIMIT (?1) OFFSET (?2);").as_str(), 
            &[&max, &offset])
            .await
    }

    pub async fn get_by_title(title: &str) -> Result<Option<Item>, String> {
        ItemDBO::scan_row(
            format!("{ITEM_SELECT_FROM} WHERE i.title=(?1);").as_str(), 
            &[&title]
        )
        .await
    }

    pub async fn search_by_title(title: &str, max: u32) -> Result<Vec<Item>, String> {
        ItemDBO::scan_rows(
            format!("{ITEM_SELECT_FROM} WHERE i.title LIKE '%' || (?1) || '%' LIMIT (?2);").as_str(), 
            &[&title, &max])
            .await
    }

    pub async fn get_by_status(status: &str, page: u32, max: u32) -> Result<Vec<Item>, String> {
        let offset = (page - 1) * max;
        ItemDBO::scan_rows(
            format!("{ITEM_SELECT_FROM} where i.status=(?1) ORDER BY i.created_on DESC LIMIT (?2) OFFSET (?3);").as_str(), 
            &[&status, &max, &offset])
            .await
    }

    pub async fn get_by_kind(kind: &str, page: u32, max: u32) -> Result<Vec<Item>, String> {
        let offset = (page - 1) * max;
        ItemDBO::scan_rows(
            format!("{ITEM_SELECT_FROM} where i.kind=(?1) ORDER BY i.created_on DESC LIMIT (?2) OFFSET (?3);").as_str(), 
            &[&kind, &max, &offset])
            .await
    }

    pub async fn get_by_tag(tag: &str, page: u32, max: u32) -> Result<Vec<Item>, String> {
        todo!()
    }

    pub async fn get_by_metadata_key(key: &str, page: u32, max: u32) -> Result<Vec<Item>, String> {
        todo!()
    }

    pub async fn get_by_metadata_value(key: &str, value: &str, page: u32, max: u32) -> Result<Vec<Item>, String> {
        todo!()
    }

    pub async fn set_metadata_pair(item_id: u32, key: &str, value: &str) -> Result<(), String> {
        todo!()
    }

    pub async fn delete_metadata_pair(item_id: u32, key: &str, value: &str) -> Result<(), String> {
        todo!()
    }
    
    pub async fn add(item: &AddInfo) -> Result<(), String> {
        let db = crate::core::database::DB.lock().unwrap();
        let e = db.execute(
            "INSERT INTO item (title) values (?1);", 
            (&item.title,));
        if let Err(err) = e {
            eprintln!("Error adding item: {err}");
            return Err(String::from("error adding item"))
        }
        Ok(())
    }

    pub async fn update(item_id: &u32, fields: &BTreeMap<String, serde_json::Value>) -> Result<(), String> {
        todo!()
    }

    pub async fn delete(item_id: &u32) -> Result<(), String> {
        todo!()
    }

    async fn scan_rows(sql: &str, params: &[&dyn ToSql]) -> Result<Vec<Item>, String> {
        let db = crate::core::database::DB.lock().unwrap();
        let mut stmt = db
            .prepare(sql)
            .map_err(|e| {
                eprintln!("DB prepare error: {}", e);
                String::from("DB error")
            })?;
        let items = stmt
            .query_map(params, |row| {
                Ok(Item {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    kind: row.get(2)?,
                    icon: row.get(3)?,
                    content: row.get(4)?,
                    status: row.get(5)?,
                    date: None,
                    created_on: None,
                    updated_at: None,
                    week: row.get(9)?,
                    year: row.get(10)?,
                    parent_id: row.get(11)?,
                    parent_title: row.get(12)?,
                })
            })
            .map_err(|e| {
                eprintln!("DB Query Error?: {}", e);
                String::from("DB Error")
            })?
            .filter_map(|res| res.ok())
            .collect::<Vec<Item>>();
        Ok(items)
    }

    async fn scan_row(sql: &str, params: &[&dyn ToSql]) -> Result<Option<Item>, String> {        
        let items_res = ItemDBO::scan_rows(sql, params).await;
        if let Err(err) = items_res {
            return Err(err);
        }
        let items = items_res.unwrap();
        if items.len() == 0 {
            return Ok(None)
        }
        let item = &items[0];
        Ok(Some(item.clone()))
    }
}