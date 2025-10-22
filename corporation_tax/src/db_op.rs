use crate::esi::{
    Portraits, ResCharacterPublicInformation, ResCorporationInformation,
    ResCorporationWalletJournal,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use db_wallet::{
    JournalRefType,
    entities::{
        characters::{ActiveModel as AmCharacters, Column as CCharacters, Entity as ECharacters},
        corporation_wallet_journal::{
            ActiveModel as AmCorporationWalletJournal, Column as CCorporationWalletJournal,
            Entity as ECorporationWalletJournal,
        },
        corporations::{ActiveModel as AmCorporations, Entity as ECorporations},
        pap_journal::{Column as CPapJournal, Entity as EPapJournal},
        tax_parameters::{Column as CTaxParameters, Entity as ETaxParameters},
        taxable_list::{Column as CTaxableList, Entity as ETaxableList},
        users::{Column as CUsers, Entity as EUsers},
    },
};
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult, NotSet, QueryFilter,
    QuerySelect, Set,
};
use std::collections::BTreeSet;

pub async fn db_upgrade_wall_journal<DB: ConnectionTrait>(
    db: &DB,
    journal: ResCorporationWalletJournal,
) -> Result<usize, String> {
    let mut wait_write = Vec::new();

    for item in journal.0 {
        let row = ECorporationWalletJournal::find_by_id(item.id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        if row.is_none() {
            wait_write.push(item);
        }
    }

    let count = wait_write.len();

    for item in wait_write {
        let data = AmCorporationWalletJournal {
            id: Set(item.id),
            date: Set(item.date.timestamp()),
            description: Set(item.description),
            ref_type: Set(item.ref_type as i32),
            amount: Set(item.amount.map(|i| decimal_to_i64(i))),
            balance: Set(item.balance.map(|i| decimal_to_i64(i))),
            context_id: Set(item.context_id),
            context_id_type: Set(item.context_id_type.map(|t| t as i32)),
            reason: Set(item.reason),
            first_party_id: Set(item.first_party_id),
            second_party_id: Set(item.second_party_id),
            tax: Set(item.tax.map(|i| decimal_to_i64(i))),
            tax_receiver_id: Set(item.tax_receiver_id),
        };

        ECorporationWalletJournal::insert(data)
            .exec(db)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(count)
}

#[derive(FromQueryResult)]
struct FirstPartyId {
    first_party_id: Option<i64>,
}

#[derive(FromQueryResult)]
struct SecondPartyId {
    second_party_id: Option<i64>,
}

// 获取 corporation_wallet_journal first_party_id second_party_id 中的所有ID
pub async fn get_all_ids<DB: ConnectionTrait>(db: &DB) -> Result<Vec<i64>, String> {
    let first_party_ids = ECorporationWalletJournal::find()
        .select_only()
        .column(CCorporationWalletJournal::FirstPartyId)
        .group_by(CCorporationWalletJournal::FirstPartyId)
        .into_model::<FirstPartyId>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let second_party_ids = ECorporationWalletJournal::find()
        .select_only()
        .column(CCorporationWalletJournal::SecondPartyId)
        .group_by(CCorporationWalletJournal::SecondPartyId)
        .into_model::<SecondPartyId>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut ids = BTreeSet::<i64>::new();
    for m in first_party_ids {
        if let Some(id) = m.first_party_id {
            ids.insert(id);
        }
    }
    for m in second_party_ids {
        if let Some(id) = m.second_party_id {
            ids.insert(id);
        }
    }

    // 需要排除的ID号
    let abnormal_ids: [i64; _] = [500016];
    for id in abnormal_ids {
        ids.remove(&id);
    }

    Ok(Vec::from_iter(ids.iter().copied()))
}

// 获取不在 characters, 也不在 corporations 表中的 ID
pub async fn check_out_unknown_ids<DB: ConnectionTrait>(
    db: &DB,
    ids: Vec<i64>,
) -> Result<Vec<i64>, String> {
    let mut unknown_ids = Vec::new();

    for id in ids {
        let r = check_id(db, id).await?;
        if r.is_some() {
            continue;
        }

        unknown_ids.push(id);
    }

    Ok(unknown_ids)
}

// 查询ID是 character 还是 corporation
// Some(true) 表示 character, Some(false) 表示 corporation
// None 表示不在两个表中
pub async fn check_id<DB: ConnectionTrait>(db: &DB, id: i64) -> Result<Option<bool>, String> {
    let flag_character = ECharacters::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    if flag_character.is_some() {
        return Ok(Some(true));
    }

    let flag_corporation = ECorporations::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    if flag_corporation.is_some() {
        return Ok(Some(false));
    }

    Ok(None)
}

// 获取角色名
pub async fn get_character_name<DB: ConnectionTrait>(
    db: &DB,
    id: i64,
) -> Result<Option<String>, String> {
    let d = ECharacters::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(d.map(|c| c.name.clone()))
}

// 获取公司名
pub async fn get_corporation_name<DB: ConnectionTrait>(
    db: &DB,
    id: i64,
) -> Result<Option<String>, String> {
    let d = ECorporations::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(d.map(|c| c.name.clone()))
}

// 插入角色数据
pub async fn insert_character_info<DB: ConnectionTrait>(
    db: &DB,
    character_id: i64,
    info: ResCharacterPublicInformation,
    portraits: Portraits,
) -> Result<(), String> {
    let m = AmCharacters {
        character_id: Set(character_id),
        alliance_id: Set(info.alliance_id),
        corporation_id: Set(info.corporation_id),
        birthday: Set(info.birthday.timestamp()),
        name: Set(info.name),
        user_id: NotSet,
        main: Set(false),
        portrait64: Set(Some(portraits.portrait64)),
        portrait128: Set(Some(portraits.portrait128)),
        portrait256: Set(Some(portraits.portrait256)),
        portrait512: Set(Some(portraits.portrait512)),
    };

    ECharacters::insert(m)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// 更新角色数据
pub async fn update_character_info<DB: ConnectionTrait>(
    db: &DB,
    character_id: i64,
    info: ResCharacterPublicInformation,
    portraits: Portraits,
) -> Result<(), String> {
    let m = AmCharacters {
        character_id: Set(character_id),
        alliance_id: Set(info.alliance_id),
        corporation_id: Set(info.corporation_id),
        birthday: Set(info.birthday.timestamp()),
        name: Set(info.name),
        user_id: NotSet,
        main: Set(false),
        portrait64: Set(Some(portraits.portrait64)),
        portrait128: Set(Some(portraits.portrait128)),
        portrait256: Set(Some(portraits.portrait256)),
        portrait512: Set(Some(portraits.portrait512)),
    };

    ECharacters::update(m)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// 插入公司数据
pub async fn insert_corporation_info<DB: ConnectionTrait>(
    db: &DB,
    corporation_id: i64,
    info: ResCorporationInformation,
) -> Result<(), String> {
    let m = AmCorporations {
        corporation_id: Set(corporation_id),
        name: Set(info.name),
        ticker: Set(info.ticker),
        date_founded: Set(info.date_founded.map(|t| t.timestamp())),
        description: Set(info.description),
    };

    ECorporations::insert(m)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// 更新公司数据
pub async fn update_corporation_info<DB: ConnectionTrait>(
    db: &DB,
    corporation_id: i64,
    info: ResCorporationInformation,
) -> Result<(), String> {
    let m = AmCorporations {
        corporation_id: Set(corporation_id),
        name: Set(info.name),
        ticker: Set(info.ticker),
        date_founded: Set(info.date_founded.map(|t| t.timestamp())),
        description: Set(info.description),
    };

    ECorporations::update(m)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// 获取所有用户ID
pub async fn get_users_ids<DB: ConnectionTrait>(db: &DB) -> Result<Vec<i32>, String> {
    #[derive(FromQueryResult)]
    struct RowData {
        id: i32,
    }

    let data = EUsers::find()
        .select_only()
        .column(CUsers::Id)
        .into_model::<RowData>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(data.iter().map(|c| c.id).collect())
}

// 获取指定用户的所有角色
pub async fn get_user_characters_ids<DB: ConnectionTrait>(
    db: &DB,
    user_id: i32,
) -> Result<Vec<i64>, String> {
    #[derive(FromQueryResult)]
    struct RowData {
        character_id: i64,
    }

    let ids = ECharacters::find()
        .select_only()
        .column(CCharacters::CharacterId)
        .filter(CCharacters::UserId.eq(user_id))
        .into_model::<RowData>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ids.iter().map(|c| c.character_id).collect())
}

// 获取指定用户的主角色名, 若没有标注主角色,  则返回微信群昵称
pub async fn get_user_main_character_name<DB: ConnectionTrait>(
    db: &DB,
    user_id: i32,
) -> Result<String, String> {
    #[derive(FromQueryResult)]
    struct RowData {
        name: String,
    }

    let data = ECharacters::find()
        .select_only()
        .column(CCharacters::Name)
        .filter(
            Condition::all()
                .add(CCharacters::UserId.eq(user_id))
                .add(CCharacters::Main.eq(true)),
        )
        .into_model::<RowData>()
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(d) = data {
        return Ok(d.name);
    }

    let user = EUsers::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(u) = user {
        if let Some(n) = u.we_chat_group_nickname {
            return Ok(n);
        }
    }

    Err("User not found".to_string())
}

// 查询指定角色在指定时间范围内上缴的税收总数
pub async fn find_character_tax_amount<DB: ConnectionTrait>(
    db: &DB,
    character_id: i64,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Decimal, String> {
    #[derive(FromQueryResult)]
    struct RowData {
        amount: i64,
    }

    assert!(start_time <= end_time);

    let start_time = start_time.timestamp();
    let end_time = end_time.timestamp();

    let data = ECorporationWalletJournal::find()
        .select_only()
        .column(CCorporationWalletJournal::Amount)
        .filter(
            Condition::all()
                .add(CCorporationWalletJournal::Date.gte(start_time))
                .add(CCorporationWalletJournal::Date.lt(end_time))
                .add(CCorporationWalletJournal::FirstPartyId.eq(character_id))
                .add(CCorporationWalletJournal::RefType.eq(JournalRefType::PlayerDonation as i32))
                .add(CCorporationWalletJournal::Amount.gt(0)),
        )
        .into_model::<RowData>()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut sum = Decimal::ZERO;
    for d in data {
        sum += decimal_from_i64(d.amount);
    }

    Ok(sum)
}

// 查询指定用户在指定时间范围内上缴的税收总额
pub async fn find_user_pay_tax_amount<DB: ConnectionTrait>(
    db: &DB,
    user_id: i32,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Decimal, String> {
    assert!(start_time <= end_time);
    let ids = get_user_characters_ids(db, user_id).await?;
    let mut sum = Decimal::ZERO;
    for character_id in ids {
        let a = find_character_tax_amount(db, character_id, start_time, end_time).await?;
        sum += a;
    }
    Ok(sum)
}

// 查询指定用户在指定月份上缴的税收总额
pub async fn find_user_year_month_pay_tax<DB: ConnectionTrait>(
    db: &DB,
    user_id: i32,
    year_month: YearMonth,
) -> Result<Decimal, String> {
    let start = year_month.lower();
    let end = year_month.upper();
    find_user_pay_tax_amount(db, user_id, start, end).await
}

// 获取指定用户在指定月份需上缴的税收
// (poll_tax, pap_tax)
pub async fn get_user_tax<DB: ConnectionTrait>(
    db: &DB,
    user_id: i32,
    year_month: YearMonth,
) -> Result<(Decimal, Decimal), String> {
    let mut poll_tax_amount = Decimal::ZERO;
    let mut pap_tax_amount = Decimal::ZERO;

    let (flag_poll_tax, flag_pap_tax) = get_user_taxable(db, user_id, year_month).await?;
    if (flag_poll_tax == false) & (flag_pap_tax == false) {
        return Ok((poll_tax_amount, pap_tax_amount));
    }

    let (par_poll_tax, par_pap_tax, par_pap_standard) = get_tax_parameters(db, year_month).await?;

    if flag_poll_tax {
        poll_tax_amount = par_poll_tax;
    }

    if flag_pap_tax {
        let user_pap = find_user_pap(db, user_id, year_month).await?;
        let delta_pap = par_pap_standard - user_pap;
        if delta_pap.is_sign_positive() {
            pap_tax_amount = delta_pap * par_pap_tax;
        }
    }

    Ok((poll_tax_amount, pap_tax_amount))
}

// 获取指定用户在指定月份是否需要交税
// (poll_tax, pap_tax)
pub async fn get_user_taxable<DB: ConnectionTrait>(
    db: &DB,
    user_id: i32,
    year_month: YearMonth,
) -> Result<(bool, bool), String> {
    #[derive(FromQueryResult)]
    struct RowData {
        poll_tax: bool,
        pap_tax: bool,
    }

    let data = ETaxableList::find()
        .filter(
            Condition::all()
                .add(CTaxableList::UserId.eq(user_id))
                .add(CTaxableList::Year.eq(year_month.year as i32))
                .add(CTaxableList::Month.eq(year_month.month as i32)),
        )
        .into_model::<RowData>()
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    match data {
        None => Ok((false, false)),
        Some(o) => Ok((o.poll_tax, o.pap_tax)),
    }
}

// 查询指定角色在指定月份的PAP分
pub async fn find_character_pap<DB: ConnectionTrait>(
    db: &DB,
    character_id: i64,
    year_month: YearMonth,
) -> Result<Decimal, String> {
    #[derive(FromQueryResult)]
    struct RowData {
        pap: i32,
    }

    let data = EPapJournal::find()
        .filter(
            Condition::all()
                .add(CPapJournal::CharacterId.eq(character_id))
                .add(CPapJournal::Year.eq(year_month.year as i32))
                .add(CPapJournal::Month.eq(year_month.month as i32)),
        )
        .into_model::<RowData>()
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    match data {
        None => Ok(Decimal::ZERO),
        Some(o) => Ok(decimal_from_i64(o.pap as i64)),
    }
}

// 查询指定用户在指定月份的PAP分
pub async fn find_user_pap<DB: ConnectionTrait>(
    db: &DB,
    user_id: i32,
    year_month: YearMonth,
) -> Result<Decimal, String> {
    let ids = get_user_characters_ids(db, user_id).await?;
    let mut sum = Decimal::ZERO;
    for id in ids {
        let pap = find_character_pap(db, id, year_month).await?;
        sum += pap;
    }

    Ok(sum)
}

// 获取指定月份的税收计算参数
// (poll_tax, pap_tax, pap_standard)
pub async fn get_tax_parameters<DB: ConnectionTrait>(
    db: &DB,
    year_month: YearMonth,
) -> Result<(Decimal, Decimal, Decimal), String> {
    #[derive(FromQueryResult)]
    struct RowData {
        poll_tax: i64,
        pap_tax: i64,
        pap_standard: i32,
    }

    let data = ETaxParameters::find()
        .filter(
            Condition::all()
                .add(CTaxParameters::Year.eq(year_month.year as i32))
                .add(CTaxParameters::Month.eq(year_month.month as i32)),
        )
        .into_model::<RowData>()
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    let row_data = data.ok_or(format!(
        "no tax parameters found, year:{}, month:{}",
        year_month.year, year_month.month
    ))?;

    let poll_tax = decimal_from_i64(row_data.poll_tax);
    let pap_tax = decimal_from_i64(row_data.pap_tax);
    let pap_standard = decimal_from_i64(row_data.pap_standard as i64);

    Ok((poll_tax, pap_tax, pap_standard))
}

fn decimal_to_i64(mut d: Decimal) -> i64 {
    d.rescale(2);
    d.mantissa() as i64
}

pub fn decimal_from_i64(d: i64) -> Decimal {
    Decimal::new(d, 2)
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub struct YearMonth {
    pub year: i16,
    pub month: u8,
}

impl YearMonth {
    pub fn new(year: i16, month: u8) -> Self {
        Self { year, month }
    }

    pub fn lower(&self) -> DateTime<Utc> {
        let date = NaiveDate::from_ymd_opt(self.year as i32, self.month as u32, 1).unwrap();
        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        DateTime::from_naive_utc_and_offset(datetime, Utc)
    }

    pub fn upper(&self) -> DateTime<Utc> {
        let mut month = self.month as u32;
        let mut year = self.year as i32;
        if self.month >= 12 {
            month = 1;
            year += 1;
        } else {
            month += 1;
        }
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let datetime = NaiveDateTime::new(date, time);
        DateTime::from_naive_utc_and_offset(datetime, Utc)
    }

    pub fn add_month(&self, month: i32) -> YearMonth {
        let month = (self.month as i32) + month;

        let delta_year: i32 = month.div_euclid(12);
        let delta_month: i32 = month.rem_euclid(12);

        let tmp_year = (self.year as i32) + delta_year;
        let tmp_month = delta_month;

        if tmp_month == 0 {
            YearMonth {
                year: (tmp_year - 1) as i16,
                month: 12,
            }
        } else {
            YearMonth {
                year: tmp_year as i16,
                month: tmp_month as u8,
            }
        }
    }

    pub fn to_string_zh(&self) -> String {
        format!("{}年{}月", self.year, self.month)
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        // 2025-11
        if let Some((y_str, m_str)) = s.split_once("-") {
            let y = y_str.parse::<i16>().map_err(|e| e.to_string())?;
            let m = m_str.parse::<u8>().map_err(|e| e.to_string())?;
            if m < 1 || m > 12 {
                Err(format!("invalid month: {}", m))
            } else {
                Ok(Self::new(y, m))
            }
        } else {
            Err("illegal format".to_string())
        }
    }
}

#[test]
fn year_month_add() {
    let a = YearMonth::new(2025, 9);
    println!("2025.9 + 1 = {:?}", a.add_month(1));
    println!("2025.9 + 3 = {:?}", a.add_month(3));
    println!("2025.9 + 4 = {:?}", a.add_month(4));
    println!("2025.9 + 10 = {:?}", a.add_month(10));
    println!("2025.9 - 1 = {:?}", a.add_month(-1));
    println!("2025.9 - 8 = {:?}", a.add_month(-8));
    println!("2025.9 - 9 = {:?}", a.add_month(-9));
    println!("2025.9 - 10 = {:?}", a.add_month(-10));
    println!("2025.9 - 20 = {:?}", a.add_month(-20));

    let a = YearMonth::new(2025, 12);
    println!("2025.12 + 0 = {:?}", a.add_month(0));
    println!("2025.12 + 1 = {:?}", a.add_month(1));
}

pub struct RangeYearMonth {
    start: YearMonth, // 包含
    end: YearMonth,   // 包含
}

impl RangeYearMonth {
    pub fn new(start: YearMonth, end: YearMonth) -> Self {
        assert!(start <= end);
        Self { start, end }
    }
}

impl Iterator for RangeYearMonth {
    type Item = YearMonth;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let out = self.start;
            self.start = self.start.add_month(1);
            Some(out)
        }
    }
}

#[test]
fn test_range_year_month_iter() {
    let start = YearMonth::new(2025, 10);
    let end = YearMonth::new(2025, 10);
    let range = RangeYearMonth::new(start, end);
    for i in range {
        println!("{:?}", i);
    }
    let start = YearMonth::new(2025, 10);
    let end = YearMonth::new(2026, 3);
    let range = RangeYearMonth::new(start, end);
    for i in range {
        println!("{:?}", i);
    }
}
