mod generators;
mod parser;
mod schema;

use clap::{Parser, ValueEnum};
use color_eyre::Result;
use generators::{DbmlGenerator, DiagramGenerator, MermaidGenerator};
use parser::EntityParser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Generate database schema diagrams from SeaORM entities"
)]
struct Args {
    /// Paths to entity directories or files (can specify multiple)
    /// Defaults to libs/domains/*/src for this repo's structure
    #[arg(short, long, num_args = 1..)]
    entities_path: Option<Vec<String>>,

    /// Output directory for generated diagrams
    #[arg(short, long, default_value = "docs")]
    output: PathBuf,

    /// Format of the diagram to generate
    #[arg(short, long, value_enum, default_value = "all")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Mermaid,
    Dbml,
    All,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    // Default paths for this repo's structure
    let entities_paths = args.entities_path.clone().unwrap_or_else(|| {
        vec![
            "libs/domains/tasks/src".to_string(),
            "libs/domains/projects/src".to_string(),
            "libs/domains/cloud_resources/src".to_string(),
            "libs/domains/users/src".to_string(),
        ]
    });

    if args.verbose {
        println!("Parsing entities from:");
        for path in &entities_paths {
            println!("  - {}", path);
        }
    }

    // Parse entities
    let parser = EntityParser::new(entities_paths.clone());
    let mut schema = parser.parse()?;

    // Parse relations separately
    let relations = parser::parse_relations(&entities_paths)?;

    // Add relations to schema
    for (table_name, table_relations) in relations {
        if let Some(table) = schema.tables.iter_mut().find(|t| t.name == table_name) {
            for relation in table_relations {
                table.add_relation(relation);
            }
        }
    }

    if args.verbose {
        println!("Found {} tables", schema.tables.len());
        for table in &schema.tables {
            println!(
                "  - {} ({} fields, {} relations)",
                table.name,
                table.fields.len(),
                table.relations.len()
            );
        }
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;

    // Generate diagrams based on format
    match args.format {
        OutputFormat::Mermaid => {
            generate_mermaid(&schema, &args)?;
        }
        OutputFormat::Dbml => {
            generate_dbml(&schema, &args)?;
        }
        OutputFormat::All => {
            generate_mermaid(&schema, &args)?;
            generate_dbml(&schema, &args)?;
        }
    }

    println!("Schema diagrams generated successfully!");

    Ok(())
}

fn generate_mermaid(schema: &schema::DatabaseSchema, args: &Args) -> Result<()> {
    let generator = MermaidGenerator::new();
    let output = generator.generate(schema);

    let output_path = args.output.join("schema.md");
    fs::write(&output_path, output)?;

    println!("Generated Mermaid diagram: {}", output_path.display());

    Ok(())
}

fn generate_dbml(schema: &schema::DatabaseSchema, args: &Args) -> Result<()> {
    let generator = DbmlGenerator::new();
    let output = generator.generate(schema);

    let output_path = args.output.join("schema.dbml");
    fs::write(&output_path, output)?;

    println!("Generated DBML diagram: {}", output_path.display());

    Ok(())
}
