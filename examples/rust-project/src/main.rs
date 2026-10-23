use tiny_tally::count_words;

// @handwritten:start
fn main() {
    let text = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    println!("{}", count_words(&text));
}
// @handwritten:end
