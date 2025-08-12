#[allow(dead_code)]
use {
    axum::{Json, Router, extract::Path, response::IntoResponse, routing::get},
    serde_json::json,
    std::sync::LazyLock,
};

mod routes;

static PORT: LazyLock<u16> = LazyLock::new(|| {
    if let Ok(s) = std::env::var("API_ARUN_GG_PORT") {
        if let Ok(port) = s.parse() {
            return port;
        }
    }

    let default = 3000;
    tracing::warn!(
        "No value provided for API_ARUN_GG_PORT. Defaulting to :{}",
        default
    );
    default
});

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt::init();

    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/coin", get(flip_a_coin))
        .route("/random_number", get(random_number))
        .route("/random_colour", get(random_colour))
        .route("/unit/{n}", get(unit_conversion))
        .nest("/next", routes::next::next());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", *PORT)).await?;
    tracing::info!("Listening on {:?}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn flip_a_coin() -> impl IntoResponse {
    libarun::random::flip_a_coin()
}

async fn random_number() -> impl IntoResponse {
    libarun::random::random_number().to_string()
}

async fn random_colour() -> impl IntoResponse {
    let j = libarun::random::random_colour();
    Json(j)
}

#[axum::debug_handler]
async fn unit_conversion(Path(num): Path<f64>) -> impl IntoResponse {
    let j = json!(
        {
            "temperature": {
                "farenheit_to_celsius": libarun::unit_conversion::farenheit_to_celsius(num),
                "celsius_to_farenheit": libarun::unit_conversion::celsius_to_farenheit(num),
            },
            "length": {
                "meters_to_feet": libarun::unit_conversion::meters_to_feet(num),
                "feet_to_meters": libarun::unit_conversion::feet_to_meters(num),
                "miles_to_km": libarun::unit_conversion::miles_to_km(num),
                "km_to_miles": libarun::unit_conversion::km_to_miles(num),
                "inches_to_cm": libarun::unit_conversion::inches_to_cm(num),
                "cm_to_inches": libarun::unit_conversion::cm_to_inches(num),
                "yards_to_meters": libarun::unit_conversion::yards_to_meters(num),
                "meters_to_yards": libarun::unit_conversion::meters_to_yards(num),

            },
            "volume": {
                "gallons_to_liters": libarun::unit_conversion::gallons_to_liters(num),
                "liters_to_gallons": libarun::unit_conversion::liters_to_gallons(num),
                "pints_to_liters": libarun::unit_conversion::pints_to_liters(num),
                "liters_to_pints": libarun::unit_conversion::liters_to_pints(num),
                "liters_to_quarts": libarun::unit_conversion::liters_to_quarts(num),
                "quarts_to_liters": libarun::unit_conversion::quarts_to_liters(num),
                "cups_to_milliliters": libarun::unit_conversion::cups_to_milliliters(num),
                "milliliters_to_cups": libarun::unit_conversion::milliliters_to_cups(num),
                "tablespoons_to_milliliters": libarun::unit_conversion::tablespoons_to_milliliters(num),
                "milliliters_to_tablespoons": libarun::unit_conversion::milliliters_to_tablespoons(num),
                "teaspoons_to_milliliters": libarun::unit_conversion::teaspoons_to_milliliters(num),
                "milliliters_to_teaspoons": libarun::unit_conversion::milliliters_to_teaspoons(num),
                "milliliters_to_fluid_ounces": libarun::unit_conversion::milliliters_to_fluid_ounces(num),
                "fluid_ounces_to_milliliters": libarun::unit_conversion::fluid_ounces_to_milliliters(num),

            },
            "mass": {
                "lbs_to_kg": libarun::unit_conversion::lbs_to_kg(num),
                "kg_to_lbs": libarun::unit_conversion::kg_to_lbs(num),
                "oz_to_g": libarun::unit_conversion::ounces_to_grams(num),
                "g_to_oz": libarun::unit_conversion::grams_to_ounces(num),
            },
            "time": {

            },
            "area": {

            },
            "speed": {

            },
            "force": {

            },
            "pressure": {

            },
            "energy": {

            },
            "power": {

            },
            "voltage": {

            },
            "current": {

            },
            "resistance": {

            },
            "capacitance": {

            },
            "inductance": {

            },
        }
    );
    Json(j)
}

// macro_rules! conversion_endpoint {
//     ($func_name:ident) => {
//         pub async fn $func_name(Path(num): Path<f64>) -> impl IntoResponse {
//             libarun::unit_conversion::$func_name(num).to_string()
//         }
//     };
// }

// there's probably a way to make a proc macro that loops through every function in
// libarun::unit_conversion and passes each function name into this macro here...

// conversion_endpoint!(celsius_to_farenheit);
// conversion_endpoint!(farenheit_to_celsius);

// output from gemini asking how to do the proc macro
/*
 * =====================================================================================
 *
 * SETUP INSTRUCTIONS
 *
 * This single code block contains a full Cargo project with two crates.
 * To run this, you need to create the following file structure and copy the
 * code into the respective files.
 *
 * PROJECT STRUCTURE:
 * ------------------
 * fn_lister_project/
 * ├── Cargo.toml              (Workspace definition)
 * │
 * ├── fn_lister/              (The procedural macro crate)
 * │   ├── Cargo.toml
 * │   └── src/
 * │       └── lib.rs
 * │
 * └── my_app/                 (The binary crate that uses the macro)
 * ├── Cargo.toml
 * ├── src/
 * │   ├── main.rs
 * │   └── unit_conversion.rs
 *
 * After setting up the files, run `cargo run` from inside the `my_app` directory.
 *
 * =====================================================================================
 */

// === File: fn_lister_project/Cargo.toml ===
/*
[workspace]
members = [
    "fn_lister",
    "my_app",
]
*/

// === File: fn_lister_project/fn_lister/Cargo.toml ===
/*
[package]
name = "fn_lister"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
*/

// === File: fn_lister_project/fn_lister/src/lib.rs ===
/*
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Path};
use std::fs;
use std::path::PathBuf;
use std::env;

/// A procedural macro that reads a module's source file and creates a vector
/// of its public function names.
///
/// ## Usage:
///
/// ```rust
/// // Given a module `crate::my_module` in `src/my_module.rs`
/// let functions = list_fns_in_path!(crate::my_module);
/// ```
///
/// ## Limitations:
///
/// - **Path Resolution:** This macro assumes a standard `src/` directory layout.
///   It converts a Rust path like `crate::unit_conversion` into a file path like
///   `.../src/unit_conversion.rs`. It does not handle more complex module
///   structures (e.g., `mod/mod.rs` or `#[path(...)]` attributes).
///
/// - **Conditional Compilation:** The macro does not process `#[cfg(...)]` attributes.
///   It will list any public function it finds in the file, even if that function
///   would have been compiled out for the target platform.
///
/// - **Macros:** It cannot see functions generated by other macros within the target module.
///   It only sees the literal source code.
#[proc_macro]
pub fn list_fns_in_path(input: TokenStream) -> TokenStream {
    // 1. Parse the macro input which should be a Rust-style path (e.g., `crate::unit_conversion`).
    let path_ast = parse_macro_input!(input as Path);

    // 2. Resolve the Rust path to a file system path. This is the most fragile part.
    // We get the directory of the crate using the macro.
    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            let error = syn::Error::new_spanned(
                path_ast,
                "This macro requires the `CARGO_MANIFEST_DIR` environment variable, which is set by Cargo."
            ).to_compile_error();
            return TokenStream::from(error);
        }
    };

    // Build the path to the source file. We skip the first segment if it's `crate` or `self`.
    let mut file_path = manifest_dir;
    file_path.push("src");
    let segments_to_join = path_ast.segments.iter()
        .filter(|s| s.ident != "crate" && s.ident != "self")
        .map(|s| s.ident.to_string());

    for segment in segments_to_join {
        file_path.push(segment);
    }
    file_path.set_extension("rs");

    // 3. Read the source file content.
    let source_code = match fs::read_to_string(&file_path) {
        Ok(code) => code,
        Err(e) => {
            let error_msg = format!("Failed to read source file at '{}'. Error: {}", file_path.display(), e);
            let error = syn::Error::new_spanned(path_ast, error_msg).to_compile_error();
            return TokenStream::from(error);
        }
    };

    // 4. Parse the source code into a syntax tree using the `syn` crate.
    let ast = match syn::parse_file(&source_code) {
        Ok(file) => file,
        Err(e) => {
            let error_msg = format!("Failed to parse Rust source code from '{}'. Error: {}", file_path.display(), e);
            let error = syn::Error::new_spanned(path_ast, error_msg).to_compile_error();
            return TokenStream::from(error);
        }
    };

    // 5. Iterate through the items in the syntax tree and collect the names of public functions.
    let mut fn_names = Vec::new();
    for item in ast.items {
        if let syn::Item::Fn(item_fn) = item {
            // Check if the function's visibility is `pub`.
            if let syn::Visibility::Public(_) = item_fn.vis {
                let fn_name = item_fn.sig.ident.to_string();
                fn_names.push(fn_name);
            }
        }
    }

    // 6. Use the `quote` crate to generate the final code.
    // This will expand to `vec!["name1", "name2", ...];`
    let output = quote! {
        vec![#(#fn_names),*]
    };

    TokenStream::from(output)
}
*/

// === File: fn_lister_project/my_app/Cargo.toml ===
/*
[package]
name = "my_app"
version = "0.1.0"
edition = "2021"

[dependencies]
fn_lister = { path = "../fn_lister" }
*/

// === File: fn_lister_project/my_app/src/unit_conversion.rs ===
/*
// This module contains functions we want to discover.

pub fn meters_to_feet(m: f64) -> f64 {
    m * 3.28084
}

pub fn feet_to_meters(f: f64) -> f64 {
    f / 3.28084
}

pub fn kilograms_to_pounds(kg: f64) -> f64 {
    kg * 2.20462
}

// This is a private function and should NOT be included in the list.
#[allow(dead_code)]
fn internal_calculation() -> f64 {
    42.0
}

pub fn pounds_to_kilograms(lb: f64) -> f64 {
    lb / 2.20462
}
*/

// === File: fn_lister_project/my_app/src/main.rs ===
/*
// Import the procedural macro from our other crate.
use fn_lister::list_fns_in_path;

// Define the module whose functions we want to list.
// This makes `crate::unit_conversion` a valid path.
mod unit_conversion;

fn main() {
    println!("Invoking macro to get function list...");

    // Here we invoke the macro, passing it the path to our module.
    // At compile time, this line will be replaced by:
    // vec!["meters_to_feet", "feet_to_meters", "kilograms_to_pounds", "pounds_to_kilograms"];
    let function_names = list_fns_in_path!(crate::unit_conversion);

    println!("\nDiscovered public functions in 'unit_conversion.rs':");
    for (i, name) in function_names.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }

    // You can now use this vector for whatever you need.
    assert_eq!(function_names.len(), 4);
    assert!(function_names.contains(&"meters_to_feet"));
    assert!(!function_names.contains(&"internal_calculation")); // Ensure private function is excluded

    println!("\nSuccessfully verified the function list.");
}
*/
