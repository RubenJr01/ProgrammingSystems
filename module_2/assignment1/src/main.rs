fn sum_with_steps(total: &mut i32, low: i32, high: i32, step: i32) {
    let mut sum = 0;
    let mut current = low;
    while current <= high {
        sum += current;
        current += step;
    }
    *total = sum;
}

fn main() {
    let mut result = 0;
    sum_with_steps(&mut result, 0, 100, 1);
    println!("Sum 0 to 100, step 1: {}", result);

    result = 0;
    sum_with_steps(&mut result, 0, 10, 2);
    println!("Sum 0 to 10, step 2: {}", result);

    result = 0;
    sum_with_steps(&mut result, 5, 15, 3);
    println!("Sum 5 to 15, step 3: {}", result);
}