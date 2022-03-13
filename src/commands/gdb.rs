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
    kind: u16,
    results: Vec<i32>,
}

enum Mode {
    Advantage,
    Disadvantage,
    Summation,
}

fn random_rolls(dice: &mut Dice) {
    for dice_roll in dice.info.iter_mut() {
        // StdRng implements the 'Send' and 'Sync' trait, necessary for async calls
        // Initialize the RNG from the OS entropy source
        let mut rng = StdRng::from_entropy();
        let rng_roll = Uniform::from(1..=dice_roll.kind);
        for _ in 0..dice_roll.amount {
            dice_roll.results.push(rng_roll.sample(&mut rng).into());
        }
    }
}

fn calculate_result(dice: &Dice) -> String {
    // Add the results from all dice rolls
    let mut result: i32 = 0;
    for dice_roll in dice.info.iter() {
        match dice_roll.mode {
            Mode::Advantage => {
                let max_value = dice_roll.results.iter().max();
                match max_value {
                    Some(value) => {
                        result += value;
                    }
                    None => (),
                }
            }
            Mode::Disadvantage => {
                let min_value = dice_roll.results.iter().min();
                match min_value {
                    Some(value) => {
                        result += value;
                    }
                    None => (),
                }
            }
            Mode::Summation => {
                let sum_value: i32 = dice_roll.results.iter().sum();
                result += sum_value;
            }
        }
    }
    // Apply the modifier on it
    result += dice.modifier;
    // Turn the result into a string
    result.to_string()
}

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

    let kind = capture.get(2).unwrap().as_str().parse::<u16>().unwrap();
    if kind < 2 || kind > 100 {
        return Err(("Please use a dice type between 2 and 100.").to_string());
    } else {
        dice_info.kind = kind;
    }

    Ok(dice_info)
}

fn parse_args(mut args: Args) -> Result<Dice, String> {
    let mut dice = Dice {
        info: Vec::new(),
        modifier: 0,
        comment: String::new(),
    };

    // Parse multiple dice information and comments
    let mut found_dice_rolls = false;
    for arg in args.iter::<String>() {
        let arg = arg.unwrap();
        // Regex matches: {+/-/0..99}{d/w}{0..999}
        let dice_regex =
            Regex::new(r"^(\+|\-|\d{1,2}?)[d|w](\d{1,3}?)$").expect("Regex initialization failed.");
        if dice_regex.is_match(&arg) == true {
            let capture = dice_regex
                .captures(&arg)
                .expect("Cannot capture dice regex information.");
            let dice_info = parse_dice_infos(capture)?;
            dice.info.push(dice_info);
            found_dice_rolls = true;
        }
        // Find start of comments
        if arg == "!" {
            dice.comment = args.rest().to_string();
            break;
        }
    }

    // Abort if no dice pattern could be found
    if found_dice_rolls == false {
        return Err("No suitable dice roll found. See **!help** for the syntax.".to_string());
    }

    // Restore args so that the modifiers can be parsed
    args.restore();

    // Parse dice modifiers if any
    for arg in args.iter::<i32>() {
        match arg {
            Ok(value) => {
                dice.modifier += value;
            }
            Err(_) => (),
        }
    }

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

    let mut dice = match parse_args(args) {
        Ok(dice) => dice,
        Err(error) => {
            let error_message = format!("{}: {}", author_name, error);
            msg.channel_id.say(&ctx.http, error_message).await?;
            return Ok(());
        }
    };

    // If amount of dice could be determined and which dice type, start rolling :)
    random_rolls(&mut dice);

    // Extract the result from the rolls and apply the modifier on it
    let dice_result = calculate_result(&dice);

    // Send the result back to the channel
    let author_string = format!("{} rolled", author_name);
    let mut dice_string = String::new();
    for dice_roll in dice.info.iter() {
        let dice_info = format!(
            " {}d{} {:?}",
            dice_roll.amount, dice_roll.kind, dice_roll.results
        );
        dice_string.push_str(dice_info.as_str());
    }
    let modifier_string = format!(" {:+}", dice.modifier);
    let response = MessageBuilder::new()
        .push(author_string)
        .push(dice_string)
        .push(modifier_string)
        .push(":  ")
        .push_bold(dice_result)
        .push("  ")
        .push(dice.comment)
        .build();
    msg.channel_id.say(&ctx.http, response).await?;

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
"__**Gaming Dice Bot Help**__
Available commands are: **!roll**, **!ping**, **!add**, **!subtract**, **!multiply**, **!divide**
Command **!roll** will let you roll an amount of dice between **1** and **50**. You can choose all dice types between **2** and **100**.
The roll command is build by three different parts: 1) dice amount and type (necessary), 2) dice modifiers (optional), 3) comments (optional).
An advantage and disadvantage roll with **+** or **-**, selecting either the maximum/minimum value of two dice rolls (advantage/disadvantage) is also available.
Modifiers can be added in the form of e.g. **+3** or **-24**.
Comments are parsed after the **!** character.
Whitespace is used as a separator. It is therefore necessary to use it between the different parts of the dice rolls in order to parse them as arguments.
A dice roll could look like the follwing:
    `!roll -d20 +2 ! Goblin attack`
A multiple dice roll could be as follows:
    `!roll 4d8 +3 2d6 +5 ! Combo damage`
The general command syntax is:
    `!roll <amount>d/w<type> <modifiers> ! <comments>`
"
    );
    msg.channel_id.say(&ctx.http, message).await?;
    Ok(())
}
