use clap::{arg, Command};
use dotenv::dotenv;
use sqlx::Row;
use sqlx::{query, Executor, PgPool};
use std::{env, fs, path::Path};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load environment variables from .env file

    let app = Command::new("roll")
        .version("0.1.0")
        .author("Troels F. Roennow <troels@wonop.com>")
        .about("Roll is engineered to excel in the realm of database housekeeping, ensuring meticulous management and seamless application of database fixtures. With a focus on precision and reliability, Roll acts as an indispensable assistant, dedicated to the upkeep and orderly arrangement of database environments.")
        .arg(arg!(--database_url <DATABASE_URL> "Sets the database URL"))
        .subcommand(Command::new("loaddata").about("Loads data from fixtures into the database"))
        .get_matches();

    match app.subcommand() {
        Some(("loaddata", matches)) => {
            // Retrieve the database URL from command line arguments or environment variables
            let database_url = matches.get_one::<String>("database_url")
                .map(|s| s.to_string())
                .unwrap_or_else(|| env::var("DATABASE_URL").expect("DATABASE_URL must be set or provided as an argument"));

            let pool = PgPool::connect(&database_url).await.expect("Failed to connect to database");

            // Run query to create the applied_fixtures table if it does not exist
            let create_table_query = r#"
                CREATE TABLE IF NOT EXISTS applied_fixtures (
                    fixture_id SERIAL PRIMARY KEY,
                    fixture_name VARCHAR(255) NOT NULL UNIQUE,
                    applied_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
                );
            "#;
            pool.execute(create_table_query).await.expect("Failed to create applied_fixtures table");

            apply_fixtures_from_dir(&pool, "fixtures").await;

            println!("All fixtures loaded successfully.");
        },
        _ => {
            eprintln!("No subcommand was used. For usage information, use --help.");
            std::process::exit(1);
        }
    }
}

async fn apply_fixtures_from_dir(pool: &PgPool, dir: &str) {
    // Read and sort SQL fixture files from the fixtures directory
    let fixture_paths = fs::read_dir(dir)
        .expect("Failed to read fixtures directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension() == Some(std::ffi::OsStr::new("sql")))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();


        let sorted_fixture_paths = {
            let mut paths = fixture_paths;
            paths.sort();
            paths
        };
    
        apply_fixtures(&pool, &sorted_fixture_paths).await;
        println!("All fixtures loaded successfully.");        
}


async fn apply_fixtures(pool: &PgPool, sorted_fixture_paths: &Vec<PathBuf>) {
    // Apply each fixture if not already applied
    for path in sorted_fixture_paths {
        apply_fixture(pool, path).await;
    }
}

async fn apply_fixture(pool: &PgPool, path: &PathBuf) {
    let fixture_name = path.file_name().unwrap().to_str().unwrap();
    let already_applied = pool
        .fetch_one(query("SELECT EXISTS (SELECT 1 FROM applied_fixtures WHERE fixture_name = $1)").bind(fixture_name))
        .await
        .expect("Failed to check if fixture is applied")
        .get::<bool, _>(0);

    if already_applied {
        println!("Skipping already applied fixture: {:?}", path);
        return;
    }

    let fixture_sql = fs::read_to_string(&path).expect(&format!("Failed to read fixture file {:?}", path));
    pool.execute(&*fixture_sql).await.expect(&format!("Failed to execute fixture file {:?}", path));

    println!("Executed fixture file {:?}", path);
    pool.execute(
        query("INSERT INTO applied_fixtures (fixture_name) VALUES ($1) ON CONFLICT (fixture_name) DO NOTHING").bind(fixture_name),
    )
    .await
    .expect("Failed to record execution of fixture file");

    println!("Recorded execution of fixture file {:?}", path);
}
