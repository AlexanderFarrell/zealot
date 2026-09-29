//! Async data layer: every fetch/mutation is a spawned task whose result
//! comes back to the app loop as a [`Msg`] on the shared channel.

use chrono::NaiveDate;
use tokio::sync::mpsc::UnboundedSender;
use zealot_client::ZealotClient;
use zealot_domain::item::SearchScope;
use zealot_domain::repeat::UpdateRepeatEntryDto;

use crate::msg::{DayData, LinkKind, Msg};

#[derive(Clone)]
pub struct Net {
    pub client: ZealotClient,
    pub tx: UnboundedSender<Msg>,
}

impl Net {
    fn send(&self, msg: Msg) {
        let _ = self.tx.send(msg);
    }

    pub fn load_day(&self, date: NaiveDate, generation: u64) {
        let net = self.clone();
        tokio::spawn(async move {
            let (plan, habits, blocks, comments) = tokio::join!(
                net.client.planner_day(date),
                net.client.repeats_for_day(date),
                net.client.time_blocks_for_day(date),
                net.client.comments_for_day(date),
            );
            let data = (|| {
                Ok(DayData {
                    plan: plan.map_err(stringify)?,
                    habits: habits.map_err(stringify)?,
                    blocks: blocks.map_err(stringify)?,
                    comments: comments.map_err(stringify)?,
                })
            })();
            net.send(Msg::Day { generation, data });
        });
    }

    pub fn load_roots(&self, generation: u64) {
        let net = self.clone();
        tokio::spawn(async move {
            let items = net.client.list_items(None).await.map_err(stringify);
            net.send(Msg::Roots { generation, items });
        });
    }

    pub fn load_children(&self, parent_id: i64, generation: u64) {
        let net = self.clone();
        tokio::spawn(async move {
            let items = net.client.get_children(parent_id).await.map_err(stringify);
            net.send(Msg::Children {
                generation,
                parent_id,
                items,
            });
        });
    }

    pub fn load_random(&self, count: usize, generation: u64) {
        let net = self.clone();
        tokio::spawn(async move {
            let items = net.client.random_items(count).await.map_err(stringify);
            net.send(Msg::Random { generation, items });
        });
    }

    pub fn search(&self, term: String, scope: SearchScope, regex: bool, generation: u64) {
        let net = self.clone();
        tokio::spawn(async move {
            let results = net
                .client
                .search_items(&term, scope, regex, 30, 0)
                .await
                .map_err(stringify);
            net.send(Msg::SearchResults {
                generation,
                results,
            });
        });
    }

    pub fn open_item(&self, item_id: i64) {
        let net = self.clone();
        tokio::spawn(async move {
            let item = net
                .client
                .get_item(item_id)
                .await
                .map(Box::new)
                .map_err(stringify);
            net.send(Msg::OpenItem { item });
        });
    }

    pub fn open_item_by_title(&self, title: String) {
        let net = self.clone();
        tokio::spawn(async move {
            let item = net
                .client
                .get_item_by_title(&title)
                .await
                .map(Box::new)
                .map_err(stringify);
            net.send(Msg::OpenItem { item });
        });
    }

    pub fn load_links(&self, item_id: i64, kind: LinkKind) {
        let net = self.clone();
        tokio::spawn(async move {
            let items = match kind {
                LinkKind::Backlinks => net.client.get_backlinks(item_id).await,
                LinkKind::Related => net.client.get_related(item_id).await,
                LinkKind::Children => net.client.get_children(item_id).await,
            }
            .map_err(stringify);
            net.send(Msg::Links { kind, items });
        });
    }

    pub fn set_habit_status(&self, item_id: i64, date: NaiveDate, status: &str) {
        let net = self.clone();
        let dto = UpdateRepeatEntryDto {
            item_id,
            date: date.format("%Y-%m-%d").to_string(),
            status: Some(status.to_string()),
            comment: None,
        };
        let status = status.to_string();
        tokio::spawn(async move {
            let result = net
                .client
                .set_repeat_status(&dto)
                .await
                .map(|()| format!("habit → {status}"))
                .map_err(stringify);
            net.send(Msg::Done(result));
        });
    }

    pub fn add_comment(&self, item_id: i64, content: String) {
        let net = self.clone();
        tokio::spawn(async move {
            let dto = zealot_domain::comment::AddCommentDto {
                item_id,
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                content,
            };
            let result = net
                .client
                .add_comment(&dto)
                .await
                .map(|c| format!("logged comment #{}", c.comment_id))
                .map_err(stringify);
            net.send(Msg::Done(result));
        });
    }

    /// Persist content returned by `$EDITOR` before the app accepts more input.
    ///
    /// Unlike the other fire-and-forget mutations, an editor round-trip must
    /// not outlive the main loop: quitting the TUI would drop the Tokio runtime
    /// and cancel an in-flight save.
    pub async fn save_item_content(&self, item_id: i64, content: String) -> Result<String, String> {
        let dto = zealot_domain::item::UpdateItemDto {
            item_id,
            title: None,
            content: Some(content),
            attributes: None,
            links: None,
        };
        self.client
            .update_item(&dto)
            .await
            .map(|item| format!("saved '{}'", item.title))
            .map_err(stringify)
    }

    pub fn load_habits_range(&self, start: NaiveDate, end: NaiveDate, generation: u64) {
        let net = self.clone();
        tokio::spawn(async move {
            let entries = net
                .client
                .repeats_for_range(start, end)
                .await
                .map_err(stringify);
            net.send(Msg::HabitsRange {
                generation,
                entries,
            });
        });
    }

    pub fn load_rules(&self, generation: u64) {
        let net = self.clone();
        tokio::spawn(async move {
            let rules = net.client.list_rules().await.map_err(stringify);
            net.send(Msg::Rules { generation, rules });
        });
    }

    pub fn run_rule(&self, rule_id: i64, name: String) {
        let net = self.clone();
        tokio::spawn(async move {
            let result = net.client.run_rule(rule_id).await.map_err(stringify);
            net.send(Msg::RuleRan { name, result });
        });
    }

    /// Persist a rule script returned by `$EDITOR` before the main loop resumes.
    pub async fn save_rule_script(&self, rule_id: i64, script: String) -> Result<String, String> {
        let dto = zealot_domain::rule::UpdateRuleDto {
            name: None,
            description: None,
            trigger: None,
            script: Some(script),
            enabled: None,
        };
        self.client
            .update_rule(rule_id, &dto)
            .await
            .map(|rule| format!("saved script for '{}'", rule.name))
            .map_err(stringify)
    }
}

fn stringify(e: zealot_client::ApiError) -> String {
    e.to_string()
}
