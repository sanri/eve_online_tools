use chrono::{DateTime, Timelike, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect,
};
use std::collections::BTreeMap;
use strum::{AsRefStr, EnumCount, EnumIter, IntoEnumIterator};
use umya_spreadsheet::{
    Alignment, HorizontalAlignmentValues, NumberingFormat, Style, VerticalAlignmentValues,
    Worksheet, helper::coordinate::string_from_column_index,
};

use crate::db_op::{
    RangeYearMonth, YearMonth, check_id, decimal_from_i64, find_user_year_month_pay_tax,
    get_character_name, get_corporation_name, get_user_main_character_name, get_user_tax,
    get_users_ids,
};
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
                    if data.ref_type == JournalRefType::PlayerDonation
                        || data.ref_type == JournalRefType::CorporationAccountWithdrawal
                    {
                        // 交税与对公转账收入标绿
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
        // w.get_column_dimension_mut("B").set_auto_width(true);
        // w.get_column_dimension_mut("C").set_auto_width(true);
        // w.get_column_dimension_mut("D").set_auto_width(true);
        // w.get_column_dimension_mut("E").set_auto_width(true);
        // w.get_column_dimension_mut("F").set_auto_width(true);
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
                    let id = if amount.is_sign_positive() {
                        journal.first_party_id.unwrap()
                    } else {
                        journal.second_party_id.unwrap()
                    };
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

struct UserTaxList {
    character_name: String,          // 主角色名
    amount_of_unpaid_taxes: Decimal, // 欠税金额
    list: BTreeMap<YearMonth, MonthTax>,
}

#[derive(Clone)]
struct MonthTax {
    pap_tax: Decimal,     // PAP税额
    poll_tax: Decimal,    // 人头税额
    paid_up_tax: Decimal, // 实缴税额
}

pub struct SheetTaxList {
    start: YearMonth,
    end: YearMonth,
    data: Vec<UserTaxList>,
}

impl SheetTaxList {
    pub fn insert_worksheet(&self, w: &mut Worksheet) {
        self.generate_sheet_header(w);
        self.generate_sheet_data(w);
    }

    fn generate_sheet_header(&self, w: &mut Worksheet) {
        let mut alignment = Alignment::default();
        alignment.set_horizontal(HorizontalAlignmentValues::Center);
        alignment.set_vertical(VerticalAlignmentValues::Center);

        let c = w.get_cell_mut("A1");
        c.set_value_string("主角色名");
        c.get_style_mut().set_alignment(alignment.clone());
        w.add_merge_cells("A1:A2");

        let c = w.get_cell_mut("B1");
        c.set_value_string("欠税额");
        c.get_style_mut().set_alignment(alignment.clone());
        w.add_merge_cells("B1:B2");

        let range_ym = RangeYearMonth::new(self.start, self.end);
        for (i, ym) in range_ym.enumerate() {
            let i = (i * 3 + 3) as u32;

            let c = w.get_cell_mut((i, 1));
            c.set_value_string(ym.to_string_zh());
            c.get_style_mut().set_alignment(alignment.clone());
            let start_col = string_from_column_index(&i);
            let end_col = string_from_column_index(&(i + 2));
            w.add_merge_cells(format!("{}1:{}1", start_col, end_col));

            let c = w.get_cell_mut((i, 2));
            c.set_value_string("PAP税额");
            c.get_style_mut().set_alignment(alignment.clone());

            let c = w.get_cell_mut((i + 1, 2));
            c.set_value_string("人头税额");
            c.get_style_mut().set_alignment(alignment.clone());

            let c = w.get_cell_mut((i + 2, 2));
            c.set_value_string("实缴税额");
            c.get_style_mut().set_alignment(alignment.clone());
        }
    }

    fn generate_sheet_data(&self, w: &mut Worksheet) {
        for (row, user_tax_list) in self.data.iter().enumerate() {
            let row = (row + 3) as u32;

            // 主角色名
            let c = w.get_cell_mut((1, row));
            c.set_value_string(user_tax_list.character_name.clone());

            // 欠税额
            let c = w.get_cell_mut((2, row));
            let v = user_tax_list.amount_of_unpaid_taxes.to_f64().unwrap();
            c.set_value_number(v);
            c.get_style_mut().set_numbering_format(format_isk());
            if v > 0.0 {
                // 标红
                c.get_style_mut().set_background_color("FFFFC7CE");
            }

            for (index, (_, month_tax)) in user_tax_list.list.iter().enumerate() {
                let col = (index * 3 + 3) as u32;

                // PAP税额
                let c = w.get_cell_mut((col, row));
                let v = month_tax.pap_tax.to_f64().unwrap();
                c.set_value_number(v);
                c.get_style_mut().set_numbering_format(format_isk());

                // 人头税额
                let c = w.get_cell_mut((col + 1, row));
                let v = month_tax.poll_tax.to_f64().unwrap();
                c.set_value_number(v);
                c.get_style_mut().set_numbering_format(format_isk());

                // 实缴税额
                let c = w.get_cell_mut((col + 2, row));
                let v = month_tax.paid_up_tax.to_f64().unwrap();
                c.set_value_number(v);
                c.get_style_mut().set_numbering_format(format_isk());
            }
        }
    }

    pub async fn select_from_db<DB: ConnectionTrait>(
        db: &DB,
        start: YearMonth,
        end: YearMonth,
    ) -> Result<SheetTaxList, String> {
        let mut users_tax_list = Vec::new();
        let users_ids = get_users_ids(db).await?;

        for user_id in users_ids {
            let character_name = get_user_main_character_name(db, user_id).await?;
            let mut list = BTreeMap::new();
            let range_ym = RangeYearMonth::new(start, end);
            for ym in range_ym {
                let (poll_tax, pap_tax) = get_user_tax(db, user_id, ym).await?;
                let paid_up_tax = find_user_year_month_pay_tax(db, user_id, ym).await?;
                let month_tax = MonthTax {
                    pap_tax,
                    poll_tax,
                    paid_up_tax,
                };
                list.insert(ym, month_tax);
            }
            let amount_of_unpaid_taxes = compute_unpaid_tax(&list);
            let user_tax_list = UserTaxList {
                character_name,
                amount_of_unpaid_taxes,
                list,
            };
            users_tax_list.push(user_tax_list);
        }

        Ok(SheetTaxList {
            start,
            end,
            data: users_tax_list,
        })
    }
}

fn compute_unpaid_tax(data: &BTreeMap<YearMonth, MonthTax>) -> Decimal {
    let mut paid = Decimal::ZERO;
    let mut unpaid = Decimal::ZERO;
    for (_, mt) in data {
        paid += mt.paid_up_tax;
        unpaid += mt.pap_tax + mt.poll_tax;
    }

    unpaid - paid
}

fn format_isk() -> NumberingFormat {
    NumberingFormat::default()
        .set_format_code(r#"_ [$isk]\ * #,##0_ ;_ [$isk]\ * \-#,##0_ ;_ [$isk]\ * "-"?_ ;"#)
        .to_owned()
}
