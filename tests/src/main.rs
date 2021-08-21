
use try_catch::catch;
use std::io;
fn own<T>(_: Vec<T>) {}

// fn main() {
//     let vector = vec![1, 2, 3];
//     catch! {
//         try {
//             own(vector);
//         }
//     }
// }

// #[test]
// fn test_single_try() {
//     let vector = vec![1, 2, 3];
//     catch! {
//         try {
//             own(vector);
//             println!("reading asd");
//         }
//     }
// }

// #[test]
// fn test_single_catch() {
//     let vector = vec![1, 2, 3];
//     catch! {
//         try {
//             println!("reading asd");
//             fs::read_to_string("ASD")?;

//         }
//         catch error: io::Error {
//             println!("IOError: {}", error)
//         }
//     }
// }

fn test_finally() -> i32 {
    let number: i32 = catch! {
        try {
            let number: i32 = "10".parse()?;
            number
        } catch error {
            0
        }
    };
    // we can't know for sure if all possible errors are handled so the type is still Result.
    let result: Result<i32, _> = catch! {
        try {
            let number: i32 = "10".parse()?;
            number
        } catch error: io::Error {
            0
        }
    };

    10
}
