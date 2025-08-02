fn most_frequent_word(text: &str) -> (String, usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut unique_words: Vec<&str> = Vec::new();
    let mut counts: Vec<usize> = Vec::new();

    let mut max_word = "";
    let mut max_count = 0;

    for &word in words.iter() {
        let mut found = false;
        for (_i, &w) in unique_words.iter().enumerate() {
            if w == word {
                counts[_i] += 1;
                if counts[_i] > max_count {
                    max_count = counts[_i];
                    max_word = w;
                }
                found = true;
                break;
            }
        }
        if !found {
            unique_words.push(word);
            counts.push(1);
            if max_count == 0 {
                max_count = 1;
                max_word = word;
            }
        }
    }
    (max_word.to_string(), max_count)
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text);
    println!("Most frequent word: \"{}\" ({} times)", word, count);
}