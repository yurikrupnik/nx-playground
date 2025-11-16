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
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the entities directory
    #[arg(short, long, default_value = "apps/zerg/api/src/entities")]
    entities_path: String,

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

    if args.verbose {
        println!("Parsing entities from: {}", args.entities_path);
    }

    // Parse entities
    let parser = EntityParser::new(args.entities_path.clone());
    let mut schema = parser.parse()?;

    // Parse relations separately (they're in the Relation enum)
    let relations = parser::parse_relations(&args.entities_path)?;

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

    println!("✓ Schema diagrams generated successfully!");

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
