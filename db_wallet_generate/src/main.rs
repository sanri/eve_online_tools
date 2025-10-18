use sea_orm_migration::prelude::cli;

use db_wallet::Migrator;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
