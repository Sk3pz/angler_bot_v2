use rand::distr::Alphanumeric;
use rand::Rng;

pub fn generate_error_code() -> String {
    // Define the characters you want to allow in the code
    // const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    // 
    // let mut rng = rand::rng();
    // 
    // // Generate 5 random characters and collect them into a String
    // (0..5)
    //     .map(|_| {
    //         let idx = rng.random_range(0..CHARSET.len());
    //         CHARSET[idx] as char
    //     })
    //     .collect()
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect()
}
