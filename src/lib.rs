use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, Result, Token,
};

#[derive(Debug)]
struct Schema {
    tables: Vec<Table>, // List of tables in the schema
}

#[derive(Debug)]
struct Table {
    name: Ident, // Table name
    fields: Vec<Field>, // List of fields in the table
}

#[derive(Debug)]
struct Field {
    modifier: Option<FieldModifier>, // Field modifier (PRIMARY, FOREIGN, NULLABLE)
    name: Ident, // Field name
    ty: Ident, // Field type
    foreign_table: Option<Ident>, // Referenced table for FOREIGN keys
}

#[derive(Debug, PartialEq)]
enum FieldModifier {
    Primary, // Primary key
    Foreign, // Foreign key
    Nullable, // Nullable field
}

struct SchemaInsert {
    rows: Vec<RowInsert>, // List of row insertions
}

struct RowInsert {
    table_name: Ident, // Table name
    values: Vec<syn::ExprTuple>, // List of value tuples to insert
}


impl Parse for Schema {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut tables = Vec::new();
        
        while !input.is_empty() {
            tables.push(input.parse::<Table>()?);
            
            // Optional trailing comma after each table
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(Schema { tables })
    }
}

// Parse implementation for Table
impl Parse for Table {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse table name
        let name = input.parse::<Ident>()?;
        
        // Parse the brace-enclosed fields
        let content;
        syn::braced!(content in input);
        
        let mut fields = Vec::new();
        
        while !content.is_empty() {
            fields.push(content.parse::<Field>()?);
            
            // Expect comma after each field
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }
        
        Ok(Table { name, fields })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        // Try to parse optional modifier (PRIMARY, FOREIGN, NULLABLE)
        let mut modifier = None;
        let mut foreign_table = None;
        
        if input.peek(Ident) {
            let ident = input.fork().parse::<Ident>()?;
            match ident.to_string().as_str() {
                "PRIMARY" => {
                    input.parse::<Ident>()?; // consume it
                    modifier = Some(FieldModifier::Primary);
                }
                "FOREIGN" => {
                    input.parse::<Ident>()?; // consume it
                    modifier = Some(FieldModifier::Foreign);
                    
                    // Parse optional table reference: (TableName)
                    if input.peek(syn::token::Paren) {
                        let paren_content;
                        syn::parenthesized!(paren_content in input);
                        foreign_table = Some(paren_content.parse::<Ident>()?);
                    }
                }
                "NULLABLE" => {
                    input.parse::<Ident>()?; // consume it
                    modifier = Some(FieldModifier::Nullable);
                }
                _ => {}
            }
        }
        
        // Parse field name
        let name = input.parse::<Ident>()?;
        
        // Parse colon
        input.parse::<Token![:]>()?;
        
        // Parse type
        let ty = input.parse::<Ident>()?;
        
        Ok(Field { modifier, name, ty, foreign_table })
    }
}

impl Parse for SchemaInsert {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut rows = Vec::new();
        
        while !input.is_empty() {
            rows.push(input.parse::<RowInsert>()?);
            
            // Optional trailing comma after each table
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        
        Ok(SchemaInsert { rows })
    }
}

impl Parse for RowInsert {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse table name
        let table_name = input.parse::<Ident>()?;
        
        // Parse the brace-enclosed value tuples
        let content;
        syn::braced!(content in input);
        
        let mut values = Vec::new();
        
        while !content.is_empty() {
            values.push(content.parse::<syn::ExprTuple>()?);
            
            // Expect comma after each tuple
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }
        
        Ok(RowInsert { table_name, values })
    }
}

#[proc_macro]
pub fn create_tables(input: TokenStream) -> TokenStream {
    // Parse the input tokens directly as our Schema type
    let schema = parse_macro_input!(input as Schema);
    
    // Generate SQL from the parsed schema
    let sql = generate_sql_create(&schema.tables);
    
    // Generate the output code
    let expanded = quote! {
        {
            const SQL: &str = #sql;
            SQL
        }
    };
    
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn add_rows(input: TokenStream) -> TokenStream {

    let schema_insert = parse_macro_input!(input as SchemaInsert);

    let sql = generate_sql_insert(&schema_insert.rows);

    // Placeholder for add_rows! macro implementation
    let expanded = quote! {
        {
            const SQL_INSERT: &str = #sql;
            SQL_INSERT
        }
    };
    
    TokenStream::from(expanded)
}

fn generate_sql_create(tables: &[Table]) -> String {
    let mut sql = String::new();
    
    // Generate SQL for each table
    for table in tables {
        sql.push_str(&format!("CREATE TABLE {} (\n", table.name));
        
        let mut field_defs = Vec::new();
        let mut constraints = Vec::new();
        
        // Generate field definitions
        for field in &table.fields {
            let field_name = &field.name;
            let sql_type = match field.ty.to_string().as_str() {
                "i32" => "INTEGER",
                "str" => "TEXT",
                "f32" => "REAL",
                "i64" => "BIGINT",
                _ => "TEXT",
            };
            
            // Generate field definition line
            let mut def = format!("    {} {}", field_name, sql_type);
            
            // Handle field modifiers
            if let Some(ref modifier) = field.modifier {
                match modifier {
                    FieldModifier::Primary => {
                        def.push_str(" PRIMARY KEY");
                    }
                    FieldModifier::Nullable => {
                        // NULL is default so no action needed
                    }
                    FieldModifier::Foreign => {
                        // Use explicit foreign table if provided, otherwise infer from field name
                        let table_name = if let Some(ref fk_table) = field.foreign_table {
                            fk_table.to_string()
                        } else {
                            // Extract table name from field name (e.g., product_id -> product)
                            let fk_name = field_name.to_string();
                            fk_name.strip_suffix("_id")
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| fk_name.clone())
                        };
                        
                        // Add foreign key constraint
                        constraints.push(format!(
                            "    FOREIGN KEY ({}) REFERENCES {}(id)",
                            field_name, table_name
                        ));
                    }
                }
            } else {
                def.push_str(" NOT NULL");
            }
            
            field_defs.push(def);
        }
        
        sql.push_str(&field_defs.join(",\n"));
        
        if !constraints.is_empty() {
            sql.push_str(",\n");
            sql.push_str(&constraints.join(",\n"));
        }
        
        sql.push_str("\n);\n\n");
    }
    
    sql
}

fn generate_sql_insert (rows: &[RowInsert]) -> String {
    let mut sql = String::new();

    for row_insert in rows {
        let table_name = &row_insert.table_name;
        
        for values in &row_insert.values {
            let value_strings: Vec<String> = values.elems.iter().map(|expr| {
                match expr {
                    syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => format!("'{}'", s.value()),
                    syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. }) => i.base10_digits().to_string(),
                    syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Float(f), .. }) => f.base10_digits().to_string(),
                    syn::Expr::Path(p) if p.path.is_ident("NULL") => "NULL".to_string(),
                    _ => "NULL".to_string(), // Fallback for unsupported types
                }
            }).collect();

            let values_str = value_strings.join(", ");
            sql.push_str(&format!("INSERT INTO {} VALUES ({});\n", table_name, values_str));
        }
    }

    sql
}
