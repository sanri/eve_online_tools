mod db_op;
mod esi;
mod report;

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use sea_orm::{ConnectOptions, ConnectionTrait, Database};
use std::{path::Path, str::FromStr};
use tokio::fs::read_to_string;
use umya_spreadsheet::{new_file_empty_worksheet, writer};

use crate::{
    db_op::{
        check_out_unknown_ids, db_upgrade_wall_journal, get_all_ids, get_character_name,
        get_corporation_name, insert_character_info, insert_corporation_info,
        update_character_info, update_corporation_info,
    },
    esi::{CORPORATION_ID, QueryDevice},
    report::SheetWalletJournal,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let db_url = format!("sqlite://{}?mode=rw", cli.db_path);
    let connect_options = ConnectOptions::new(db_url);
    let db = Database::connect(connect_options).await.unwrap();

    match cli.command {
        SubCommands::UpgradeWalletJournal {
            token_path,
            https_proxy,
        } => {
            println!("Upgrading Wallet Journal");

            if let Err(e) = upgrade_wallet_journal(https_proxy, token_path, &db).await {
                println!("{}", e);
            }

            println!("Upgraded Wallet Journal");
        }
        SubCommands::UpgradeInformation {
            https_proxy,
            character_id,
            corporation_id,
        } => {
            println!("Upgrading Information");
            let query_device = QueryDevice::new(https_proxy, None);

            if character_id.is_none() && corporation_id.is_none() {
                if let Err(e) = upgrade_information(&query_device, &db).await {
                    println!("{}", e);
                }
            }

            if let Some(corporation_id) = corporation_id {
                if let Err(e) = upgrade_character_info(&query_device, &db, corporation_id).await {
                    println!("{}", e);
                }
            }

            if let Some(corporation_id) = corporation_id {
                if let Err(e) = upgrade_corporation_info(&query_device, &db, corporation_id).await {
                    println!("{}", e);
                }
            }

            println!("Upgraded Information");
        }
        SubCommands::GenerateReport {
            output_path,
            start_time,
            end_time,
        } => {
            println!("Generating report");
            let p = Path::new(output_path.as_str());
            let start_time = DateTime::<Utc>::from_str(start_time.as_str()).unwrap();
            let end_time = DateTime::<Utc>::from_str(end_time.as_str()).unwrap();

            if let Err(e) = generate_report(&db, &p, start_time, end_time).await {
                println!("{}", e);
            }

            println!("Generated report");
        }
    }
}

async fn upgrade_wallet_journal<DB: ConnectionTrait>(
    proxy: Option<String>,
    token_path: String,
    db: &DB,
) -> Result<(), String> {
    let token_str = read_to_string(token_path)
        .await
        .map_err(|e| e.to_string())?;
    let query_device = QueryDevice::new(proxy, Some(token_str));

    for page in 1..100 {
        let journals = query_device
            .get_corporation_wallet_journal(CORPORATION_ID, 1, page)
            .await?;

        let journals = match journals {
            None => {
                break;
            }
            Some(o) => o,
        };

        let count = db_upgrade_wall_journal(db, journals).await?;
        println!("wallet journal page {} upgrade {} rows", page, count);
    }
    Ok(())
}

async fn upgrade_information<DB: ConnectionTrait>(
    query_device: &QueryDevice,
    db: &DB,
) -> Result<(), String> {
    let ids = get_all_ids(db).await?;
    println!("all id count: {}", ids.len());
    let ids = check_out_unknown_ids(db, ids).await?;
    println!("unknown id count: {}", ids.len());
    if ids.is_empty() {
        return Ok(());
    }

    let mut unknown_ids = Vec::new();
    for id in ids {
        if let Some(info) = query_device.get_character_public_information(id).await? {
            let name = info.name.clone();
            let urls = query_device.get_character_portraits(id).await?;
            let portraits = query_device.get_portraits(&urls).await?;
            insert_character_info(db, id, info, portraits).await?;
            println!("inserted character {}: {}", id, name);
            continue;
        }

        if let Some(info) = query_device.get_corporation_information(id).await? {
            let name = info.name.clone();
            insert_corporation_info(db, id, info).await?;
            println!("inserted corporation {}: {}", id, name);
            continue;
        }

        unknown_ids.push(id);
    }

    if unknown_ids.is_empty() == false {
        println!("the final unknown ids: {:?}", unknown_ids);
    }

    Ok(())
}

async fn upgrade_character_info<DB: ConnectionTrait>(
    query_device: &QueryDevice,
    db: &DB,
    character_id: i64,
) -> Result<(), String> {
    if let Some(info) = query_device
        .get_character_public_information(character_id)
        .await?
    {
        let name = info.name.clone();
        let urls = query_device.get_character_portraits(character_id).await?;
        let portraits = query_device.get_portraits(&urls).await?;

        if get_character_name(db, character_id).await?.is_some() {
            update_character_info(db, character_id, info, portraits).await?;
            println!("updated character {}: {}", character_id, name);
            Ok(())
        } else {
            insert_character_info(db, character_id, info, portraits).await?;
            println!("inserted character {}: {}", character_id, name);
            Ok(())
        }
    } else {
        Err(format!("unknown character_id: {}", character_id))
    }
}

async fn upgrade_corporation_info<DB: ConnectionTrait>(
    query_device: &QueryDevice,
    db: &DB,
    corporation_id: i64,
) -> Result<(), String> {
    if let Some(info) = query_device
        .get_corporation_information(corporation_id)
        .await?
    {
        let name = info.name.clone();
        if get_corporation_name(db, corporation_id).await?.is_some() {
            update_corporation_info(db, corporation_id, info).await?;
            println!("updated corporation {}: {}", corporation_id, name);
            Ok(())
        } else {
            insert_corporation_info(db, corporation_id, info).await?;
            println!("inserted corporation {}: {}", corporation_id, name);
            Ok(())
        }
    } else {
        Err(format!("unknown corporation_id: {}", corporation_id))
    }
}

async fn generate_report<DB: ConnectionTrait>(
    db: &DB,
    output_path: &Path,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<(), String> {
    let data = SheetWalletJournal::select_from_db(db, start_time, end_time).await?;
    let mut book = new_file_empty_worksheet();
    let worksheet = book.new_sheet("主账户流水").map_err(|e| e.to_string())?;
    data.insert_worksheet(worksheet);
    writer::xlsx::write(&book, output_path).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Parser)]
#[command(version, rename_all = "snake_case")]
struct Cli {
    #[arg(long)]
    db_path: String,

    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand)]
#[command(rename_all = "snake_case")]
enum SubCommands {
    #[command(about = "upgrade corporation wallet journal")]
    UpgradeWalletJournal {
        #[arg(long)]
        token_path: String,

        #[arg(long)]
        https_proxy: Option<String>,
    },

    #[command(about = "upgrade characters and corporations information")]
    UpgradeInformation {
        #[arg(long)]
        https_proxy: Option<String>,

        #[arg(long)]
        character_id: Option<i64>,

        #[arg(long)]
        corporation_id: Option<i64>,
    },

    #[command(about = "generate report")]
    GenerateReport {
        #[arg(long)]
        output_path: String,

        #[arg(long)]
        start_time: String,

        #[arg(long)]
        end_time: String,
    },
}
