use average::{Estimate, Skewness};

use rand::distributions::{Distribution, Uniform};
use rand::{rngs::StdRng, SeedableRng};

use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

struct Dice {
    mode: Mode,
    amount: u8,
    kind: u8,
    modifier: i32,
    comment: String,
}

enum Mode {
    Advantage,
    Disadvantage,
    Summation,
}

fn random_rolls(dice: &Dice) -> Vec<i32> {
    // StdRng implements the 'Send' and 'Sync' trait, necessary for async calls
    // Initialize the RNG from the OS entropy source
    let mut rng = StdRng::from_entropy();
    let rng_roll = Uniform::from(1..=dice.kind);
    let mut roll_results: Vec<i32> = Vec::new();
    for _ in 0..dice.amount {
        roll_results.push(rng_roll.sample(&mut rng).into());
    }
    roll_results
}

fn perform_mode(roll_results: &Vec<i32>, dice: &Dice) -> String {
    match dice.mode {
        Mode::Advantage => {
            let max_value = roll_results.iter().max();
            match max_value {
                Some(value) => {
                    let result = value + dice.modifier;
                    result.to_string()
                }
                None => "No maximum value found.".to_string(),
            }
        }
        Mode::Disadvantage => {
            let min_value = roll_results.iter().min();
            match min_value {
                Some(value) => {
                    let result = value + dice.modifier;
                    result.to_string()
                }
                None => "No minimum value found.".to_string(),
            }
        }
        Mode::Summation => {
            let sum_value: i32 = roll_results.iter().sum();
            let result = sum_value + dice.modifier;
            result.to_string()
        }
    }
}

fn parse_args(mut args: Args) -> Result<Dice, String> {
    let mut dice = Dice {
        mode: Mode::Summation,
        amount: 1,
        kind: 0,
        modifier: 0,
        comment: String::new(),
    };

    // Parse prefix before "d" or "w"
    let dice_amount = args.current();
    match dice_amount {
        Some("+") => {
            dice.mode = Mode::Advantage;
            dice.amount = 2;
            args.advance();
        }
        Some("-") => {
            dice.mode = Mode::Disadvantage;
            dice.amount = 2;
            args.advance();
        }
        _ => {
            dice.mode = Mode::Summation;
            dice.amount = args.single::<u8>().unwrap();
            // Sanity check: amount of dice to be rolled
            if dice.amount < 1 || dice.amount > 50 {
                return Err(("Please roll an amount of dice between 1 and 50.").into());
            }
        }
    }

    // Parse dice type
    if let Some(dice_type) = args.current() {
        match dice_type {
            "2" | "3" | "4" | "5" | "6" | "7" | "8" | "10" | "12" | "14" | "16" | "20" | "24"
            | "30" | "100" => {
                dice.kind = args.single::<u8>().unwrap();
            }
            _ => {
                return Err(("Please use an implemented dice type.").into());
            }
        }
    }

    // Parse modifiers if any
    for _ in 0..args.remaining() {
        let modifier_value = args.current();
        match modifier_value {
            Some(value) => {
                let number = value.parse::<i32>();
                if number.is_ok() {
                    dice.modifier += args.single::<i32>().unwrap();
                }
            }
            None => (),
        }
    }

    // Parse any arguments left as a string
    dice.comment = args.rest().to_string();

    Ok(dice)
}

#[command]
async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {    
    // Commands are in the form: "!roll xdy [+/-<values>] [Comments]", where x determines the amount of rolled dice and y
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

    // Perform mode and apply possible modifiers
    let mode_result = perform_mode(&roll_results, &dice);

    // Send the message back to the channel
    let first_part = format!(
        "{} rolled {}d{} {:?} {:+}:  ",
        author_name,
        dice.amount.to_string(),
        dice.kind.to_string(),
        roll_results,
        dice.modifier,
    );
    let response = MessageBuilder::new()
        .push(first_part)
        .push_bold(mode_result)
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
