fn is_even(n: i32) -> bool {
    n % 2 == 0
}

fn main() {
    let numbers = [3, 5, 8, 12, 15, 21, 25, 30, 42, 50];

    for &num in numbers.iter() {
        let even_odd = if is_even(num) { "Even" } else { "Odd" };

        let fizzbuzz: &str = if num % 3 == 0 && num % 5 == 0 {
            "FizzBuzz"
        }
        else if num % 3 == 0 {
            "Fizz"
        }
        else if num % 5 == 0 {
            "Buzz"
        }
        else {
            ""
        };

        println!("Number: {num} - {even_odd} {fizzbuzz}");
    }

    let mut sum = 0;
    let mut _i = 0;
    while _i < numbers.len() {
        sum += numbers[_i];
        _i += 1;
    }
    println!("\nSum of numbers: {sum}");

    let mut largest = numbers[0];
    for &num in numbers.iter() {
        if num > largest {
            largest = num;
        }
    }
    println!("Largest number: {largest}");
}