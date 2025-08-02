const FREEZINGP: f64 = 32.0;

fn farenheit_to_celcius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn celcius_to_farenheit(c: f64) -> f64 {
    (c * 9.0 / 5.0) + 32.0
}

fn main() {
    let mut temp_f: f64 = FREEZINGP;
    println!("{temp_f}°F is {:.2}°C", farenheit_to_celcius(temp_f));

    for _i in 1..=5 {
        temp_f += 1.0;
        println!("{temp_f}°F is {:.2}°C", farenheit_to_celcius(temp_f));
    }
}