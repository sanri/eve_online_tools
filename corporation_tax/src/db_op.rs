use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, EntityTrait, FromQueryResult, NotSet, QuerySelect, Set};
use std::collections::BTreeSet;

use crate::esi::{
    Portraits, ResCharacterPublicInformation, ResCorporationInformation,
    ResCorporationWalletJournal,
};
use db_wallet::entities::{
    characters::{ActiveModel as AmCharacters, Entity as ECharacters},
    corporation_wallet_journal::{
        ActiveModel as AmCorporationWalletJournal, Column as CCorporationWalletJournal,
        Entity as ECorporationWalletJournal,
    },
    corporations::{ActiveModel as AmCorporations, Entity as ECorporations},
};

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

fn decimal_to_i64(mut d: Decimal) -> i64 {
    d.rescale(2);
    d.mantissa() as i64
}

pub fn decimal_from_i64(d: i64) -> Decimal {
    Decimal::new(d, 2)
}
