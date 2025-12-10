# sqlx-schema-macro

A Rust procedural macro for defining SQL database schemas with a clean, intuitive DSL syntax.

## Overview

`sqlx-schema-macro` simplifies database schema definition by allowing you to write SQL table definitions using a custom Rust-based DSL. The macro automatically generates the corresponding SQL `CREATE TABLE` statements.

## Features

- **Intuitive Schema Syntax**: Define tables and fields using a readable DSL
- **Field Modifiers**: Support for `PRIMARY`, `FOREIGN`, and `NULLABLE` field attributes
- **Explicit Foreign Keys**: Define foreign key relationships with explicit table references
- **Type Mapping**: Automatic conversion from Rust types to SQL types
- **Code Generation**: Generates valid SQL at compile time

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sqlx-schema-macro = "0.1.0"
```

## Usage

### Basic Example

```rust
use sqlx_schema_macro::define_schema;

fn main() {
    let sql = define_schema! {
        Product {
            PRIMARY id: i32,
            name: str,
            price: f32,
            NULLABLE tag: str
        },

        Client {
            PRIMARY id: i32,
            username: str,
            email: str
        },

        Cart {
            FOREIGN (Product) product_id: i32,
            FOREIGN (Client) client_id: i32
        }
    };
    
    println!("{}", sql);
}
```

### Output

The macro generates the following SQL:

```sql
CREATE TABLE Product (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price REAL NOT NULL,
    tag TEXT
);

CREATE TABLE Client (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL
);

CREATE TABLE Cart (
    product_id INTEGER NOT NULL,
    client_id INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product(id),
    FOREIGN KEY (client_id) REFERENCES Client(id)
);
```

## Schema Definition Syntax

### Table Declaration

```rust
TableName {
    // fields here
}
```

### Field Definition

```rust
[MODIFIER] field_name: type
```

### Field Modifiers

- **`PRIMARY`**: Marks the field as a primary key
  ```rust
  PRIMARY id: i32
  ```

- **`NULLABLE`**: Allows NULL values (fields are NOT NULL by default)
  ```rust
  NULLABLE description: str
  ```

- **`FOREIGN (TableName)`**: Creates a foreign key constraint
  ```rust
  FOREIGN (Product) product_id: i32
  ```

### Supported Types

| Rust Type | SQL Type |
|-----------|----------|
| `i32`     | INTEGER  |
| `i64`     | BIGINT   |
| `str`     | TEXT     |
| `f32`     | REAL     |

## Foreign Key Inference

If you don't explicitly specify a foreign table, the macro will infer it from the field name:

```rust
// These are equivalent:
FOREIGN (Product) product_id: i32
FOREIGN product_id: i32  // Infers "product" from "product_id"
```

## Example: E-commerce Schema

```rust
use sqlx_schema_macro::define_schema;

fn main() {
    let schema = define_schema! {
        User {
            PRIMARY id: i32,
            email: str,
            username: str,
            password: str,
            NULLABLE phone: str
        },

        Product {
            PRIMARY id: i32,
            name: str,
            description: str,
            price: f32,
            stock: i32
        },

        Order {
            PRIMARY id: i64,
            FOREIGN (User) user_id: i32,
            created_at: str,
            total_amount: f32
        },

        OrderItem {
            PRIMARY id: i64,
            FOREIGN (Order) order_id: i64,
            FOREIGN (Product) product_id: i32,
            quantity: i32,
            unit_price: f32
        }
    };
    
    println!("{}", schema);
}
```

## How It Works

The macro uses `syn` for parsing the Rust token stream and generates SQL strings at compile time. This provides:

- **Type Safety**: Syntax errors are caught at compile time
- **Performance**: SQL generation happens during compilation
- **Zero Runtime Overhead**: No runtime parsing or evaluation

## Contributing

Contributions are welcome! Feel free to submit issues and pull requests.
