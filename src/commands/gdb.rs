use average::{Estimate, Skewness};

use rand::distributions::{Distribution, Uniform};
use rand::{rngs::StdRng, SeedableRng};

use regex::Regex;

use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

struct Dice {
    info: Vec<DiceInfo>,
    modifier: i32,
    comment: String,
}

struct DiceInfo {
    mode: Mode,
    amount: u8,
    kind: u8,
    results: Vec<u8>,
}

enum Mode {
    Advantage,
    Disadvantage,
    Summation,
}

fn random_rolls(dice: &Dice) {
    for dice_roll in dice.info.iter() {
        // StdRng implements the 'Send' and 'Sync' trait, necessary for async calls
        // Initialize the RNG from the OS entropy source
        let mut rng = StdRng::from_entropy();
        let rng_roll = Uniform::from(1..=dice_roll.kind);
        for _ in 0..dice_roll.amount {
            dice_roll.results.push(rng_roll.sample(&mut rng).into());
        }
    }
}

// fn perform_mode(roll_results: &Vec<i32>, dice: &Dice) -> String {
//     match dice.mode {
//         Mode::Advantage => {
//             let max_value = roll_results.iter().max();
//             match max_value {
//                 Some(value) => {
//                     let result = value + dice.modifier;
//                     result.to_string()
//                 }
//                 None => "No maximum value found.".to_string(),
//             }
//         }
//         Mode::Disadvantage => {
//             let min_value = roll_results.iter().min();
//             match min_value {
//                 Some(value) => {
//                     let result = value + dice.modifier;
//                     result.to_string()
//                 }
//                 None => "No minimum value found.".to_string(),
//             }
//         }
//         Mode::Summation => {
//             let sum_value: i32 = roll_results.iter().sum();
//             let result = sum_value + dice.modifier;
//             result.to_string()
//         }
//     }
// }

fn parse_dice_infos(capture: regex::Captures) -> Result<DiceInfo, String> {
    let mut dice_info = DiceInfo {
        mode: Mode::Summation,
        amount: 1,
        kind: 0,
        results: Vec::new(),
    };

    let amount = capture.get(1).unwrap().as_str();
    match amount {
        "+" => {
            dice_info.mode = Mode::Advantage;
            dice_info.amount = 2;
        }
        "-" => {
            dice_info.mode = Mode::Disadvantage;
            dice_info.amount = 2;
        }
        _ => {
            dice_info.mode = Mode::Summation;
            dice_info.amount = amount.parse::<u8>().unwrap();
            // Sanity check: amount of dice to be rolled
            if dice_info.amount < 1 || dice_info.amount > 50 {
                return Err(("Please roll an amount of dice between 1 and 50.").to_string());
            }
        }
    }
    println!("Dice amount: {}", dice_info.amount);

    let kind = capture.get(2).unwrap().as_str().parse::<u8>().unwrap();
    if kind < 2 || kind > 100 {
        return Err(("Please use a dice type between 2 and 100.").to_string());
    } else {
        dice_info.kind = kind;
    }
    println!("Dice type: {}", dice_info.kind);

    Ok(dice_info)
}

fn parse_args(mut args: Args) -> Result<Dice, String> {
    let mut dice = Dice {
        info: Vec::new(),
        modifier: 0,
        comment: String::new(),
    };

    // Parse multiple dice information
    for _ in 0..args.len() {
        let arg = args.current().unwrap();
        // Regex matches: {+/-/0..99}{d/w}{0..999}
        let dice_regex =
            Regex::new(r"^(\+|\-|\d{1,2}?)[d|w](\d{1,3}?)$").expect("Regex initialization failed.");
        if dice_regex.is_match(&arg) == true {
            let capture = dice_regex
                .captures(&arg)
                .expect("Cannot capture dice regex information.");
            let dice_info = parse_dice_infos(capture)?;
            dice.info.push(dice_info);
            args.advance();
        }
    }
    println!("Args remaining: {}", args.remaining());

    // Parse dice modifiers if any
    for _ in 0..args.remaining() {
        let arg = args.current().unwrap();
        match arg.parse::<i32>() {
            Ok(value) => {
                dice.modifier += value;
                args.advance();
            }
            Err(_) => (),
        }
        println!("Dice modifier: {}", dice.modifier);
    }
    println!("Dice modifiers summed up: {}", dice.modifier);
    println!("Args remaining: {}", args.remaining());

    // Parse any arguments left as a string
    dice.comment = args.rest().to_string();
    println!("Dice comment: {}", dice.comment);

    Ok(dice)
}

#[command]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // let message = format!(
    //     "Argument length: {}\nArgument rest: {}",
    //     args.len(),
    //     args.rest()
    // );
    // msg.channel_id.say(&ctx.http, message).await?;
    // for arg in args.iter::<String>() {
    //     let argument = format!("{:?}", arg);
    //     msg.channel_id.say(&ctx.http, argument).await?;
    // }

    // Commands are in the form: "!roll {xdy} {+/-<values>} {comments}", where x determines the amount of rolled dice and y
    // the dice type. As the delimiter is set to "d" or "w", x and y are directly accessible as arguments.

    // Get the nickname of the author or the name itself
    let author_name;
    match msg.author_nick(&ctx).await {
        Some(nick) => author_name = nick,
        None => author_name = msg.author.name.clone(),
    };

    let dice = match parse_args(args) {
        Ok(dice) => dice,
        Err(error) => {
            let error_message = format!("{}: {}", author_name, error);
            msg.channel_id.say(&ctx.http, error_message).await?;
            return Ok(());
        }
    };

    // If amount of dice could be determined and which dice type, start rolling :)
    let roll_results = random_rolls(&dice);
    println!("Roll results: {:?}", roll_results);

    // Perform mode and apply possible modifiers
    //let mode_result = perform_mode(&roll_results, &dice);

    // Send the message back to the channel
    // let first_part = format!(
    //     "{} rolled {}d{} {:?} {:+}:  ",
    //     author_name,
    //     dice.amount.to_string(),
    //     dice.kind.to_string(),
    //     roll_results,
    //     dice.modifier,
    // );
    // let response = MessageBuilder::new()
    //     .push(first_part)
    //     .push_bold(mode_result)
    //     .push("  ")
    //     .push(dice.comment)
    //     .build();
    // msg.channel_id.say(&ctx.http, response).await?;

    Ok(())
}

#[command]
#[owners_only]
async fn test_randomness(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let dice_count = args.single::<u32>()?;
    let dice_type = args.single::<u32>()?;

    let message = format!(
        "Statistical tests for {} samples regarding dice type {}",
        dice_count, dice_type
    );
    msg.channel_id.say(&ctx.http, message).await?;

    let mut rng = StdRng::from_entropy();

    match dice_type {
        2 | 3 | 4 | 5 | 6 | 7 | 8 | 10 | 12 | 14 | 16 | 20 | 24 | 30 | 100 => {
            let dice = Uniform::from(1..=dice_type);
            let mut test_data = Skewness::new();
            for _ in 0..dice_count {
                test_data.add(dice.sample(&mut rng) as f64);
            }
            // Prepare the message, which will be sent to the channel
            let message = format!(
                "Mean: {}\nLength: {}\nVariance: {}\nError: {}\nSkewness: {}",
                test_data.mean(),
                test_data.len(),
                test_data.sample_variance(),
                test_data.error_mean(),
                test_data.skewness(),
            );
            msg.channel_id.say(&ctx.http, message).await?;
        }
        _ => {
            let wrong_dice_type = format!(
                "{} Please use an implemented dice type.",
                msg.author.to_string()
            );
            msg.channel_id.say(&ctx.http, wrong_dice_type).await?;
            return Ok(());
        }
    }

    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    let message = format!(
        "Gaming Dice Bot Help
         Available commands are: !roll, !ping, !add, !subtract, !multiply, !divide
         Command !roll lets you roll an amount of dice between 1 and 50. You can choose between the dice types of: 2, 4, 6, 8, 10, 12, 16, 20, 100
         An advantage and disadvantage roll with '+' or '-', which picks either the maximum value of two dice rolls (advantage) or the minimum of two dice rolls (disadvantage) is also available.
         Modifiers can also be added in the form of e.g. '+3' or '-24' after the specification of the amount of dice and the dice type.
         Everything after these modifiers will be treated as a comment.
         A proper command looks like this:
             !roll 1d20 +5 Goblin attack
         The general command syntax is:
             !roll <amount>d<type> <modifiers> <comments>
         Whitespace must be used between after the first two arguments (amount and type), between each modifier, and also the comment."
    );
    msg.channel_id.say(&ctx.http, message).await?;
    Ok(())
}
