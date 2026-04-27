use proc_macro::TokenStream;

// decided to make this a proc macro instead of doing this the
// normal person way because i felt like it :)

#[proc_macro]
pub fn color_from_hex(input: TokenStream) -> TokenStream {
    let mut input = input.to_string().replace("\"", "");

    if input.len() != 7 || input.as_bytes()[0] != b'#' {
        panic!("incorrectly formatted hex literal: {input}");
    }

    input.remove(0);

    for i in input.chars() {
        if !i.is_ascii_hexdigit() {
            panic!("invalid hex literal: {input}")
        }
    }

    let r = u8::from_str_radix(&input[0..2], 16).unwrap();
    let g = u8::from_str_radix(&input[2..4], 16).unwrap();
    let b = u8::from_str_radix(&input[4..6], 16).unwrap();

    format!("Color::from_rgb8({r}, {g}, {b})").parse().unwrap()
}
