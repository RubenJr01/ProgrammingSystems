fn check_guess(guess: i32, secret:i32) -> i32 {
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

fn main() {
    let secret = 42;

    let mut guess;
    let mut attempts = 0;

    loop {
        attempts += 1;
        guess = 25 + attempts;
        let result = check_guess(guess, secret);

        if result == 0 {
            println!("You guessed {guess}. Correct!");
            break;
        } else if result == 1 {
            println!("You guessed {guess}. Too high!");
        } else {
            println!("You guessed {guess}. Too low!");
        }
    }

    println!("It took you {attempts} guesses to find the number!");
}