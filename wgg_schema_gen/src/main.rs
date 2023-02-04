//! Very simple crate to generate the GraphQL schema for front-end code generation.
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let outdir = args.get(1).map(Path::new).unwrap_or_else(|| Path::new("./gen"));

    std::fs::create_dir_all(outdir).unwrap();

    generate_graphql_schema(outdir);
}

fn generate_graphql_schema(path: &Path) {
    let schema_file = path.join("schema.graphql");
    std::fs::write(schema_file, wgg_http::api::WggSchema::default().sdl()).expect("Failed to write GraphQL schema.");
}
