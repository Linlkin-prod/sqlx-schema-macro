use mac_proc_sqlx::{create_tables,add_rows};

/// Example usage of the define_schema! macro
fn main() {
    let mut sql = create_tables! {
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

    sql = add_rows! {
        Product {
            (1, "Laptop", 999.99, NULL),
            (2, "Smartphone", 499.49, "Electronics")
        },

        Client {
            (1, "alice", "alice@example.com"),
            (2, "bob", "bob@example.com")
        },
        Cart {
            (1, 1),
            (2, 2)
        }
    };

    println!("{}", sql);
}