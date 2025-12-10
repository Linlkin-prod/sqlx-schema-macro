use mac_proc_sqlx::define_schema;

/// Example usage of the define_schema! macro
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
                        