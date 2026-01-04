//! Super Admin Creation Script
//!
//! Creates a super admin user with full system access.
//!
//! Run with:
//!   cargo run --bin create_super_admin -- --email "admin@example.com" --password "SecurePassword123" --name "Super Admin"

use bson::oid::ObjectId;
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};
use tokio::main;
use bcrypt::{hash, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    email: String,
    password_hash: String,
    name: String,
    role: String,
    roles: Vec<String>,
    perm_version: u32,
    status: String,
    email_verified: bool,
    last_login_at: Option<mongodb::bson::DateTime>,
    created_at: mongodb::bson::DateTime,
    updated_at: mongodb::bson::DateTime,
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("MMO API - Create Super Admin");
    println!("========================================");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut email = String::new();
    let mut password = String::new();
    let mut name = String::from("Super Admin");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--email" => {
                if i + 1 < args.len() {
                    email = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --email".into());
                }
            }
            "--password" => {
                if i + 1 < args.len() {
                    password = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --password".into());
                }
            }
            "--name" => {
                if i + 1 < args.len() {
                    name = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --name".into());
                }
            }
            "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }

    // Validate required arguments
    if email.is_empty() {
        return Err("Email is required. Use --email argument.".into());
    }
    if password.is_empty() {
        return Err("Password is required. Use --password argument.".into());
    }

    // Validate email format
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format".into());
    }

    // Validate password strength
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".into());
    }

    println!("\nCreating Super Admin:");
    println!("  Email: {}", email);
    println!("  Name: {}", name);
    println!("  Role: SUPER_ADMIN");
    println!();

    // Load MongoDB URL from environment or use default
    // Supports both MONGODB_URL and MONGODB_URI environment variables
    // For authenticated MongoDB, use format: mongodb://username:password@localhost:27017
    let mongo_url = std::env::var("MONGODB_URL")
        .or_else(|_| std::env::var("MONGODB_URI"))
        .unwrap_or_else(|_| {
            println!("Note: Using default MongoDB URL. If authentication is required,");
            println!("      set MONGODB_URL or MONGODB_URI environment variable:");
            println!("      export MONGODB_URL='mongodb://username:password@localhost:27017'");
            println!();
            "mongodb://localhost:27017".to_string()
        });

    println!("Connecting to MongoDB at: {}", mongo_url.split('@').last().unwrap_or(&mongo_url));
    let client = Client::with_uri_str(&mongo_url).await?;
    let db = client.database("mmo_api");
    let users_collection: Collection<User> = db.collection("users");

    // Verify SUPER_ADMIN role exists
    let roles_collection: Collection<mongodb::bson::Document> = db.collection("roles");
    let super_admin_role = roles_collection
        .find_one(doc! { "name": "SUPER_ADMIN", "is_active": true })
        .await?;

    if super_admin_role.is_none() {
        println!("\nWARNING: SUPER_ADMIN role not found in database!");
        println!("Please run: cargo run --bin seed_roles");
        println!("\nTo seed roles, run:");
        println!("  MONGODB_URI='{}' cargo run --bin seed_roles", mongo_url);
        return Err("SUPER_ADMIN role not found".into());
    }
    println!("SUPER_ADMIN role verified in database.");

    // Check if user already exists
    println!("\nChecking if user already exists...");
    let existing_user = users_collection
        .find_one(doc! { "email": &email })
        .await?;

    if existing_user.is_some() {
        println!("WARNING: User with email {} already exists!", email);
        print!("Do you want to update this user to SUPER_ADMIN? (yes/no): ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "yes" {
            println!("Operation cancelled.");
            return Ok(());
        }

        // Update existing user to SUPER_ADMIN
        let password_hash = hash(&password, DEFAULT_COST)?;
        let now = mongodb::bson::DateTime::now();

        users_collection
            .update_one(
                doc! { "email": &email },
                doc! {
                    "$set": {
                        "name": &name,
                        "password_hash": password_hash,
                        "role": "SUPER_ADMIN",
                        "roles": ["SUPER_ADMIN"],
                        "perm_version": 1,
                        "status": "active",
                        "email_verified": true,
                        "updated_at": now,
                    }
                },
            )
            .await?;

        println!("\nUser updated successfully!");
    } else {
        // Hash password
        println!("Hashing password...");
        let password_hash = hash(&password, DEFAULT_COST)?;

        // Create super admin user
        let now = mongodb::bson::DateTime::now();
        let user = User {
            id: None,
            email: email.clone(),
            password_hash,
            name: name.clone(),
            role: "SUPER_ADMIN".to_string(),
            roles: vec!["SUPER_ADMIN".to_string()],
            perm_version: 1,
            status: "active".to_string(),
            email_verified: true,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        };

        // Insert user
        println!("Creating user in database...");
        users_collection.insert_one(user).await?;
        println!("User created successfully!");
    }

    // Print curl command to test login
    println!("\n========================================");
    println!("Super Admin created successfully!");
    println!("========================================");
    println!("\nTest the login with:");
    println!("curl -X POST http://localhost:8080/api/auth/login \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{\"email\":\"{}\",\"password\":\"{}\"}}'", email, password);
    println!("\nOr use the token to access protected endpoints:");
    println!("export TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!("  -d '{{\"email\":\"{}\",\"password\":\"{}\"}}' \\", email, password);
    println!("  | jq -r '.data.access_token')");
    println!("\ncurl -X GET http://localhost:8080/api/admin/users \\");
    println!("  -H \"Authorization: Bearer $TOKEN\"");
    println!("\n========================================\n");

    Ok(())
}

fn print_help() {
    println!("\nUsage:");
    println!("  cargo run --bin create_super_admin -- --email <email> --password <password> [--name <name>]");
    println!("\nRequired arguments:");
    println!("  --email <email>      Admin email address");
    println!("  --password <password> Admin password (min 8 characters)");
    println!("\nOptional arguments:");
    println!("  --name <name>        Admin display name (default: 'Super Admin')");
    println!("  --help               Show this help message");
    println!("\nEnvironment variables:");
    println!("  MONGODB_URL / MONGODB_URI  MongoDB connection string");
    println!("                       Default: mongodb://localhost:27017");
    println!("                       With auth: mongodb://username:password@localhost:27017");
    println!("\nExample:");
    println!("  export MONGODB_URI='mongodb://admin:secret@localhost:27017'");
    println!("  cargo run --bin create_super_admin -- --email admin@example.com --password SecurePass123");
    println!();
}
