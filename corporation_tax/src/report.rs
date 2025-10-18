use chrono::{DateTime, Timelike, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect,
};
use strum::{AsRefStr, EnumCount, EnumIter, IntoEnumIterator};
use umya_spreadsheet::{Alignment, HorizontalAlignmentValues, NumberingFormat, Style, Worksheet};

use crate::db_op::{check_id, decimal_from_i64, get_character_name, get_corporation_name};
use db_wallet::{
    JournalRefType,
    entities::corporation_wallet_journal::{
        Column as CCorporationWalletJournal, Entity as ECorporationWalletJournal,
    },
};

#[derive(EnumIter, EnumCount, AsRefStr, Clone, Copy)]
pub enum ColumnWalletJournal {
    #[strum(serialize = "日期时间")]
    DateTime = 1,
    #[strum(serialize = "类型")]
    RefType = 2,
    #[strum(serialize = "收支金额")]
    Amount = 3,
    #[strum(serialize = "账户余额")]
    Balance = 4,
    #[strum(serialize = "相关角色")]
    Character = 5,
    #[strum(serialize = "备注")]
    Description = 6,
}
impl ColumnWalletJournal {
    fn get_style(&self) -> Style {
        let mut style = Style::default();
        let format_str = match self {
            ColumnWalletJournal::DateTime => r#"yyyy-mm-dd hh:mm:ss"#,
            ColumnWalletJournal::RefType => r#"@"#,
            ColumnWalletJournal::Amount => {
                // r#"_ [$isk]\ * #,##0.00_ ;_ [$isk]\ * \-#,##0.00_ ;_ [$isk]\ * "-"?_ ;"#
                r#"_ [$isk]\ * #,##0_ ;_ [$isk]\ * \-#,##0_ ;_ [$isk]\ * "-"?_ ;"#
            }
            ColumnWalletJournal::Balance => {
                // r#"_ [$isk]\ * #,##0.00_ ;_ [$isk]\ * \-#,##0.00_ ;_ [$isk]\ * "-"?_ ;"#
                r#"_ [$isk]\ * #,##0_ ;_ [$isk]\ * \-#,##0_ ;_ [$isk]\ * "-"?_ ;"#
            }
            ColumnWalletJournal::Character => r#"@"#,
            ColumnWalletJournal::Description => r#"@"#,
        };
        let numbering_format = NumberingFormat::default()
            .set_format_code(format_str)
            .to_owned();
        style.set_numbering_format(numbering_format);

        style
    }
}

pub struct RowWalletJournal {
    date_time: DateTime<Utc>,
    ref_type: JournalRefType,
    amount: Decimal,
    balance: Decimal,
    character: String,
    description: String,
}

pub struct SheetWalletJournal {
    data: Vec<RowWalletJournal>,
}

impl SheetWalletJournal {
    pub fn insert_worksheet(&self, w: &mut Worksheet) {
        // 插入标题
        for column in ColumnWalletJournal::iter() {
            let cell = w.get_cell_mut((column as u32, 1));
            cell.set_value_string(column.as_ref());
            let mut alignment = Alignment::default();
            alignment.set_horizontal(HorizontalAlignmentValues::Center);
            cell.get_style_mut().set_alignment(alignment);
        }

        // 插入数据
        for (i, data) in self.data.iter().enumerate() {
            let row = (i + 2) as u32;
            for column in ColumnWalletJournal::iter() {
                let col = column as u32;
                let cell = w.get_cell_mut((col, row));
                let mut style = column.get_style();
                if data.amount.is_sign_negative() {
                    // 支出标红
                    style.set_background_color("FFFFC7CE");
                } else {
                    if data.ref_type == JournalRefType::PlayerDonation {
                        // 交税标绿
                        style.set_background_color("FFC6EFCE");
                    }
                }
                cell.set_style(style);

                match column {
                    ColumnWalletJournal::DateTime => {
                        let date = data.date_time.date_naive().to_epoch_days() as f64;
                        let time = data.date_time.time().num_seconds_from_midnight() as f64;
                        let days = 25569.0 + date + (time / (3600.0 * 24.0));
                        cell.set_value_number(days);
                    }
                    ColumnWalletJournal::RefType => {
                        cell.set_value_string(data.ref_type.zh_str());
                    }
                    ColumnWalletJournal::Amount => {
                        cell.set_value_number(data.amount.to_f64().unwrap());
                    }
                    ColumnWalletJournal::Balance => {
                        cell.set_value_number(data.balance.to_f64().unwrap());
                    }
                    ColumnWalletJournal::Character => {
                        cell.set_value_string(data.character.as_str());
                    }
                    ColumnWalletJournal::Description => {
                        cell.set_value_string(data.description.as_str());
                    }
                }
            }
        }

        // w.get_column_dimension_mut("A").set_auto_width(true);
        w.get_column_dimension_mut("B").set_auto_width(true);
        // w.get_column_dimension_mut("C").set_auto_width(true);
        // w.get_column_dimension_mut("D").set_auto_width(true);
        w.get_column_dimension_mut("E").set_auto_width(true);
        w.get_column_dimension_mut("F").set_auto_width(true);
    }

    pub async fn select_from_db<DB: ConnectionTrait>(
        db: &DB,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<SheetWalletJournal, String> {
        #[derive(FromQueryResult)]
        struct Journal {
            date: i64,
            description: String,
            ref_type: i32,
            amount: Option<i64>,
            balance: Option<i64>,
            first_party_id: Option<i64>,
            second_party_id: Option<i64>,
        }

        assert!(start_time <= end_time);
        let start_time = start_time.timestamp();
        let end_time = end_time.timestamp();

        let journals = ECorporationWalletJournal::find()
            .select_only()
            .column(CCorporationWalletJournal::Date)
            .column(CCorporationWalletJournal::Description)
            .column(CCorporationWalletJournal::RefType)
            .column(CCorporationWalletJournal::Amount)
            .column(CCorporationWalletJournal::Balance)
            .column(CCorporationWalletJournal::FirstPartyId)
            .column(CCorporationWalletJournal::SecondPartyId)
            .filter(
                Condition::all()
                    .add(CCorporationWalletJournal::Date.gte(start_time))
                    .add(CCorporationWalletJournal::Date.lt(end_time)),
            )
            .order_by_asc(CCorporationWalletJournal::Date)
            .into_model::<Journal>()
            .all(db)
            .await
            .map_err(|e| e.to_string())?;

        let mut data = Vec::with_capacity(journals.len());

        for journal in journals {
            let amount = decimal_from_i64(journal.amount.unwrap());
            let mut character = String::new();

            let mut character_id = None;
            let mut corporation_id = None;

            let ref_type = JournalRefType::from_repr(journal.ref_type).unwrap();
            match ref_type {
                JournalRefType::PlayerDonation => {
                    character_id = journal.first_party_id;
                }
                JournalRefType::OfficeRentalFee => {
                    corporation_id = if amount.is_sign_positive() {
                        journal.first_party_id
                    } else {
                        journal.second_party_id
                    };
                }
                JournalRefType::AgentMissionReward => {
                    character_id = journal.second_party_id;
                }
                JournalRefType::AgentMissionTimeBonusReward => {
                    character_id = journal.second_party_id;
                }
                JournalRefType::CorporationAccountWithdrawal => {
                    let id = journal.second_party_id.unwrap();
                    let r = check_id(db, id).await?;
                    if let Some(flag) = r {
                        if flag {
                            character_id = Some(id);
                        } else {
                            corporation_id = Some(id);
                        }
                    }
                }
                JournalRefType::CorporationDividendPayment => {
                    let id = journal.second_party_id.unwrap();
                    let r = check_id(db, id).await?;
                    if let Some(flag) = r {
                        if flag {
                            character_id = Some(id);
                        } else {
                            corporation_id = Some(id);
                        }
                    }
                }
                JournalRefType::BountyPrizes => {
                    character_id = journal.second_party_id;
                }
                JournalRefType::ProjectDiscoveryReward => {
                    character_id = journal.second_party_id;
                }
                JournalRefType::EssEscrowTransfer => {
                    character_id = journal.second_party_id;
                }
                JournalRefType::DailyGoalPayouts => {
                    character_id = journal.second_party_id;
                }
                _ => {
                    let id = journal.second_party_id.unwrap();
                    let r = check_id(db, id).await?;
                    if let Some(flag) = r {
                        if flag {
                            character_id = Some(id);
                        } else {
                            corporation_id = Some(id);
                        }
                    }
                }
            }

            if let Some(character_id) = character_id {
                if let Some(n) = get_character_name(db, character_id).await? {
                    character = n;
                }
            }

            if let Some(corporation_id) = corporation_id {
                if let Some(n) = get_corporation_name(db, corporation_id).await? {
                    character = n;
                }
            }

            let date_time: DateTime<Utc> = DateTime::from_timestamp_secs(journal.date).unwrap();
            let balance = decimal_from_i64(journal.balance.unwrap());
            let description = journal.description;

            let row = RowWalletJournal {
                date_time,
                ref_type,
                amount,
                balance,
                character,
                description,
            };
            data.push(row);
        }

        Ok(SheetWalletJournal { data })
    }
}
