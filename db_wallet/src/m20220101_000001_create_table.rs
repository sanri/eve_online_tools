use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let create_statements: Vec<CreateStatement> =
            statements().iter().map(|(c, _)| c.clone()).collect();

        for CreateStatement { table, index } in create_statements {
            manager.create_table(table).await?;
            for i in index {
                manager.create_index(i).await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let drop_statements: Vec<DropStatement> =
            statements().iter().rev().map(|(_, d)| d.clone()).collect();

        for DropStatement { table, index } in drop_statements {
            for i in index {
                manager.drop_index(i).await?;
            }
            manager.drop_table(table).await?;
        }

        Ok(())
    }
}

// 由于 sea-sql 生成 sqlite sql 的问题, 需要将表创建与索引创建分开
// https://github.com/SeaQL/sea-query/issues/232
#[derive(Clone)]
struct CreateStatement {
    table: TableCreateStatement,
    index: Vec<IndexCreateStatement>,
}

impl CreateStatement {
    fn new(table: TableCreateStatement, index: Vec<IndexCreateStatement>) -> CreateStatement {
        CreateStatement { table, index }
    }

    #[allow(dead_code)]
    fn print(&self) {
        let sql_table = self.table.to_string(SqliteQueryBuilder);
        println!("{}\n", sql_table);

        for index in &self.index {
            let sql_index = index.to_string(SqliteQueryBuilder);
            println!("{}\n", sql_index);
        }
    }
}

#[derive(Clone)]
struct DropStatement {
    table: TableDropStatement,
    index: Vec<IndexDropStatement>,
}

impl DropStatement {
    fn new(table: TableDropStatement, index: Vec<IndexDropStatement>) -> DropStatement {
        DropStatement { table, index }
    }

    #[allow(dead_code)]
    fn print(&self) {
        let sql_table = self.table.to_string(SqliteQueryBuilder);
        println!("{}\n", sql_table);

        for index in &self.index {
            let sql_index = index.to_string(SqliteQueryBuilder);
            println!("{}\n", sql_index);
        }
    }
}

fn statements() -> Vec<(CreateStatement, DropStatement)> {
    vec![
        (ct_users(), dt_users()),
        (ct_characters(), dt_characters()),
        (
            ct_corporation_wallet_journal(),
            dt_corporation_wallet_journal(),
        ),
        (ct_corporations(), dt_corporations()),
        (ct_pap_journal(), dt_pap_journal()),
        (ct_taxable_list(), dt_taxable_list()),
        (ct_tax_parameters(), dt_tax_parameters()),
    ]
}

#[derive(DeriveIden)]
enum IdenCorporationWalletJournal {
    #[sea_orm(iden = "corporation_wallet_journal")]
    Table,
    Id,
    Date,
    Amount,
    Balance,
    ContextId,
    ContextIdType,
    Description,
    FirstPartyId,
    Reason,
    RefType,
    SecondPartyId,
    Tax,
    TaxReceiverId,
}

fn ct_corporation_wallet_journal() -> CreateStatement {
    let table = Table::create()
        .if_not_exists()
        .table(IdenCorporationWalletJournal::Table)
        .col(
            ColumnDef::new(IdenCorporationWalletJournal::Id)
                .big_integer()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(IdenCorporationWalletJournal::Date)
                .big_unsigned()
                .not_null(),
        )
        .col(
            ColumnDef::new(IdenCorporationWalletJournal::Description)
                .text()
                .not_null(),
        )
        .col(
            ColumnDef::new(IdenCorporationWalletJournal::RefType)
                .integer()
                .not_null(),
        )
        .col(ColumnDef::new(IdenCorporationWalletJournal::Amount).big_integer())
        .col(ColumnDef::new(IdenCorporationWalletJournal::Balance).big_integer())
        .col(ColumnDef::new(IdenCorporationWalletJournal::ContextId).big_integer())
        .col(ColumnDef::new(IdenCorporationWalletJournal::ContextIdType).integer())
        .col(ColumnDef::new(IdenCorporationWalletJournal::Reason).text())
        .col(ColumnDef::new(IdenCorporationWalletJournal::FirstPartyId).big_integer())
        .col(ColumnDef::new(IdenCorporationWalletJournal::SecondPartyId).big_integer())
        .col(ColumnDef::new(IdenCorporationWalletJournal::Tax).big_integer())
        .col(ColumnDef::new(IdenCorporationWalletJournal::TaxReceiverId).big_integer())
        .to_owned();

    let index = vec![
        Index::create()
            .if_not_exists()
            .name(format!(
                "index_{}_{}",
                IdenCorporationWalletJournal::Table.to_string(),
                IdenCorporationWalletJournal::Date.to_string(),
            ))
            .table(IdenCorporationWalletJournal::Table)
            .col(IdenCorporationWalletJournal::Date)
            .to_owned(),
    ];

    CreateStatement::new(table, index)
}

fn dt_corporation_wallet_journal() -> DropStatement {
    let table = Table::drop()
        .if_exists()
        .table(IdenCorporationWalletJournal::Table)
        .to_owned();

    let index = vec![
        Index::drop()
            .if_exists()
            .name(format!(
                "index_{}_{}",
                IdenCorporationWalletJournal::Table.to_string(),
                IdenCorporationWalletJournal::Date.to_string(),
            ))
            .table(IdenCorporationWalletJournal::Table)
            .to_owned(),
    ];

    DropStatement::new(table, index)
}

#[test]
fn print_table_name() {
    let s = IdenCorporationWalletJournal::Table.to_string();
    println!("{}", s);
    let s = IdenCorporationWalletJournal::Date.to_string();
    println!("{}", s);
}

#[derive(DeriveIden)]
enum IdenCharacters {
    #[sea_orm(iden = "characters")]
    Table,
    CharacterId,
    AllianceId,
    CorporationId,
    Name,
    Birthday,
    UserId,
    Main,
    Portrait64,
    Portrait128,
    Portrait256,
    Portrait512,
}

fn ct_characters() -> CreateStatement {
    let table = Table::create()
        .if_not_exists()
        .table(IdenCharacters::Table)
        .col(
            ColumnDef::new(IdenCharacters::CharacterId)
                .big_integer()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(IdenCharacters::AllianceId).big_integer())
        .col(
            ColumnDef::new(IdenCharacters::CorporationId)
                .big_integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(IdenCharacters::Birthday)
                .big_integer()
                .not_null(),
        )
        .col(ColumnDef::new(IdenCharacters::Name).text().not_null())
        .col(ColumnDef::new(IdenCharacters::UserId).integer())
        .col(ColumnDef::new(IdenCharacters::Main).boolean().not_null())
        .col(ColumnDef::new(IdenCharacters::Portrait64).binary())
        .col(ColumnDef::new(IdenCharacters::Portrait128).binary())
        .col(ColumnDef::new(IdenCharacters::Portrait256).binary())
        .col(ColumnDef::new(IdenCharacters::Portrait512).binary())
        .to_owned();

    let index = vec![
        Index::create()
            .if_not_exists()
            .name(format!(
                "index_{}_{}",
                IdenCharacters::Table.to_string(),
                IdenCharacters::Name.to_string(),
            ))
            .table(IdenCharacters::Table)
            .col(IdenCharacters::Name)
            .to_owned(),
    ];

    CreateStatement::new(table, index)
}

fn dt_characters() -> DropStatement {
    let table = Table::drop()
        .if_exists()
        .table(IdenCharacters::Table)
        .to_owned();
    let index = vec![
        Index::drop()
            .if_exists()
            .name(format!(
                "index_{}_{}",
                IdenCharacters::Table.to_string(),
                IdenCharacters::Name.to_string(),
            ))
            .table(IdenCharacters::Table)
            .to_owned(),
    ];
    DropStatement::new(table, index)
}

#[derive(DeriveIden)]
enum IdenCorporations {
    #[sea_orm(iden = "corporations")]
    Table,
    CorporationId,
    Name,
    Ticker,
    DateFounded,
    Description,
}

fn ct_corporations() -> CreateStatement {
    let table = Table::create()
        .if_not_exists()
        .table(IdenCorporations::Table)
        .col(
            ColumnDef::new(IdenCorporations::CorporationId)
                .big_integer()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(IdenCorporations::Name).text().not_null())
        .col(ColumnDef::new(IdenCorporations::Ticker).text().not_null())
        .col(ColumnDef::new(IdenCorporations::DateFounded).big_integer())
        .col(ColumnDef::new(IdenCorporations::Description).text())
        .to_owned();
    let index = vec![];
    CreateStatement::new(table, index)
}

fn dt_corporations() -> DropStatement {
    let table = Table::drop()
        .if_exists()
        .table(IdenCorporations::Table)
        .to_owned();
    let index = vec![];
    DropStatement::new(table, index)
}

#[derive(DeriveIden)]
enum IdenUsers {
    #[sea_orm(iden = "users")]
    Table,
    Id,
    WeChatId,
    WeChatNickName,
    WeChatGroupNickname,
}

fn ct_users() -> CreateStatement {
    let table = Table::create()
        .if_not_exists()
        .table(IdenUsers::Table)
        .col(
            ColumnDef::new(IdenUsers::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(IdenUsers::WeChatId).text())
        .col(ColumnDef::new(IdenUsers::WeChatNickName).text())
        .col(ColumnDef::new(IdenUsers::WeChatGroupNickname).text())
        .to_owned();

    let index = vec![];

    CreateStatement::new(table, index)
}

fn dt_users() -> DropStatement {
    let table = Table::drop().if_exists().table(IdenUsers::Table).to_owned();
    let index = vec![];

    DropStatement::new(table, index)
}

#[derive(DeriveIden)]
enum IdenPapJournal {
    #[sea_orm(iden = "pap_journal")]
    Table,
    Id,
    CharacterId,
    Year,
    Month,
    Pap,
}

fn ct_pap_journal() -> CreateStatement {
    let table = Table::create()
        .if_not_exists()
        .table(IdenPapJournal::Table)
        .col(
            ColumnDef::new(IdenPapJournal::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(
            ColumnDef::new(IdenPapJournal::CharacterId)
                .big_integer()
                .not_null(),
        )
        .col(ColumnDef::new(IdenPapJournal::Year).integer().not_null())
        .col(ColumnDef::new(IdenPapJournal::Month).integer().not_null())
        .col(ColumnDef::new(IdenPapJournal::Pap).integer().not_null())
        .to_owned();

    let index = vec![
        Index::create()
            .if_not_exists()
            .name(format!(
                "index_{}_{}",
                IdenPapJournal::Table.to_string(),
                IdenPapJournal::CharacterId.to_string(),
            ))
            .table(IdenPapJournal::Table)
            .col(IdenPapJournal::CharacterId)
            .to_owned(),
        Index::create()
            .if_not_exists()
            .name(format!(
                "unique_{}_{}_{}_{}",
                IdenPapJournal::Table.to_string(),
                IdenPapJournal::CharacterId.to_string(),
                IdenPapJournal::Year.to_string(),
                IdenPapJournal::Month.to_string(),
            ))
            .table(IdenPapJournal::Table)
            .col(IdenPapJournal::CharacterId)
            .col(IdenPapJournal::Year)
            .col(IdenPapJournal::Month)
            .unique()
            .to_owned(),
    ];

    CreateStatement::new(table, index)
}

fn dt_pap_journal() -> DropStatement {
    let table = Table::drop()
        .if_exists()
        .table(IdenPapJournal::Table)
        .to_owned();
    let index = vec![
        Index::drop()
            .if_exists()
            .name(format!(
                "index_{}_{}",
                IdenPapJournal::Table.to_string(),
                IdenPapJournal::CharacterId.to_string(),
            ))
            .table(IdenPapJournal::Table)
            .to_owned(),
        Index::drop()
            .if_exists()
            .name(format!(
                "unique_{}_{}_{}_{}",
                IdenPapJournal::Table.to_string(),
                IdenPapJournal::CharacterId.to_string(),
                IdenPapJournal::Year.to_string(),
                IdenPapJournal::Month.to_string(),
            ))
            .table(IdenPapJournal::Table)
            .to_owned(),
    ];

    DropStatement::new(table, index)
}

#[derive(DeriveIden)]
enum IdenTaxableList {
    #[sea_orm(iden = "taxable_list")]
    Table,
    Id,
    UserId,
    Year,
    Month,
    PollTax, // true 表示此月份需要征收人头税
    PapTax,  // true 表示此月份需要征收PAP税
}

fn ct_taxable_list() -> CreateStatement {
    let table = Table::create()
        .if_not_exists()
        .table(IdenTaxableList::Table)
        .col(
            ColumnDef::new(IdenTaxableList::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(IdenTaxableList::UserId).integer().not_null())
        .col(ColumnDef::new(IdenTaxableList::Year).integer().not_null())
        .col(ColumnDef::new(IdenTaxableList::Month).integer().not_null())
        .col(
            ColumnDef::new(IdenTaxableList::PollTax)
                .boolean()
                .not_null(),
        )
        .col(ColumnDef::new(IdenTaxableList::PapTax).boolean().not_null())
        .to_owned();

    let index = vec![
        Index::create()
            .if_not_exists()
            .name(format!(
                "index_{}_{}",
                IdenTaxableList::Table.to_string(),
                IdenTaxableList::UserId.to_string()
            ))
            .table(IdenTaxableList::Table)
            .col(IdenTaxableList::UserId)
            .to_owned(),
        Index::create()
            .if_not_exists()
            .name(format!(
                "unique_{}_{}_{}_{}",
                IdenTaxableList::Table.to_string(),
                IdenTaxableList::UserId.to_string(),
                IdenTaxableList::Year.to_string(),
                IdenTaxableList::Month.to_string()
            ))
            .table(IdenTaxableList::Table)
            .col(IdenTaxableList::UserId)
            .col(IdenTaxableList::Year)
            .col(IdenTaxableList::Month)
            .to_owned(),
    ];

    CreateStatement::new(table, index)
}

fn dt_taxable_list() -> DropStatement {
    let table = Table::drop()
        .if_exists()
        .table(IdenTaxableList::Table)
        .to_owned();

    let index = vec![
        Index::drop()
            .if_exists()
            .name(format!(
                "index_{}_{}",
                IdenTaxableList::Table.to_string(),
                IdenTaxableList::UserId.to_string()
            ))
            .table(IdenTaxableList::Table)
            .to_owned(),
        Index::drop()
            .if_exists()
            .name(format!(
                "unique_{}_{}_{}_{}",
                IdenTaxableList::Table.to_string(),
                IdenTaxableList::UserId.to_string(),
                IdenTaxableList::Year.to_string(),
                IdenTaxableList::Month.to_string()
            ))
            .table(IdenTaxableList::Table)
            .to_owned(),
    ];
    DropStatement::new(table, index)
}

#[derive(DeriveIden)]
enum IdenTaxParameters {
    #[sea_orm(iden = "tax_parameters")]
    Table,
    Id,
    Year,
    Month,
    PollTax,     // 指定月份的人头税, 单位 0.01 isk
    PapTax,      // 指定月份的PAP税, 单位 0.01 isk / pap
    PapStandard, // 指定月份的达标PAP分, 单位 0.01分
}

fn ct_tax_parameters() -> CreateStatement {
    let table = Table::create()
        .if_not_exists()
        .table(IdenTaxParameters::Table)
        .col(
            ColumnDef::new(IdenTaxParameters::Id)
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(IdenTaxParameters::Year).integer().not_null())
        .col(
            ColumnDef::new(IdenTaxParameters::Month)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(IdenTaxParameters::PollTax)
                .big_integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(IdenTaxParameters::PapTax)
                .big_integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(IdenTaxParameters::PapStandard)
                .integer()
                .not_null(),
        )
        .to_owned();

    let index = vec![
        Index::create()
            .if_not_exists()
            .name(format!(
                "unique_{}_{}_{}",
                IdenTaxParameters::Table.to_string(),
                IdenTaxParameters::Year.to_string(),
                IdenTaxParameters::Month.to_string(),
            ))
            .table(IdenTaxParameters::Table)
            .col(IdenTaxParameters::Year)
            .col(IdenTaxParameters::Month)
            .to_owned(),
    ];

    CreateStatement::new(table, index)
}

fn dt_tax_parameters() -> DropStatement {
    let table = Table::drop()
        .if_exists()
        .table(IdenTaxParameters::Table)
        .to_owned();

    let index = vec![
        Index::drop()
            .if_exists()
            .name(format!(
                "unique_{}_{}_{}",
                IdenTaxParameters::Table.to_string(),
                IdenTaxParameters::Year.to_string(),
                IdenTaxParameters::Month.to_string(),
            ))
            .table(IdenTaxParameters::Table)
            .to_owned(),
    ];

    DropStatement::new(table, index)
}
