set shell := ["nu", "-c"]

default:
    @just --list --unsorted

# =================================================================================================
directory_schema_database := "target/schema_database"
_csd_dir:
    mkdir {{directory_schema_database}}

directory_test_database := "target/test_database"
_ctd_dir:
    mkdir {{directory_test_database}}

# =================================================================================================

# db_wallet.sqlite
path_db_wallet := directory_schema_database / "db_wallet.sqlite"
path_test_db_wallet := directory_test_database / "db_wallet.sqlite"
csd_url_db_wallet := "sqlite://" + path_db_wallet + "?mode=rwc"
ctd_url_db_wallet := "sqlite://" + path_test_db_wallet + "?mode=rwc"
gef_url_db_wallet := "sqlite://" + path_db_wallet + "?mode=ro"
gef_out_db_wallet := "db_wallet/src/entities"

# create schema database: db_wallet.sqlite
csd_db_wallet: _csd_dir
    cargo run --package db_wallet_generate -- -u "{{csd_url_db_wallet}}" fresh

# create test database: db_wallet.sqlite
ctd_db_wallet: _ctd_dir
    cargo run --package db_wallet_generate -- -u "{{ctd_url_db_wallet}}" fresh

# generate entities files: db_wallet
gef_db_wallet: csd_db_wallet
    sea-orm-cli generate entity \
        --database-schema SQLite \
        --database-url "{{gef_url_db_wallet}}" \
        --output-dir "{{gef_out_db_wallet}}"


# upgrade corporation wallet journal
run_upgrade_cwj:
    cargo run --package corporation_tax -- \
        --db_path "{{path_test_db_wallet}}" \
        upgrade_wallet_journal \
            --token_path "target/token.txt" \
            --https_proxy "http://127.0.0.1:9098"

# upgrade characters and corporations information
run_upgrade_information:
    cargo run --package corporation_tax -- \
        --db_path "{{path_test_db_wallet}}" \
        upgrade_information \
            --https_proxy "http://127.0.0.1:9098"

# generate report
run_generate_report:
    cargo run --package corporation_tax -- \
        --db_path "{{path_test_db_wallet}}" \
        generate_report \
            --output_path "target/t.xlsx" \
            --start_time "2025-09-01T00:00:00Z" \
            --end_time "2025-10-01T00:00:00Z"
