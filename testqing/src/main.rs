use try_catch::*;
fn main() {
    catch! {
        try {
            let x: i32 = "10asd".parse()?; 
            println!("x: {}", x);
        } 
        catch error {
            println!("Error {}", error);
        }
    }
}
