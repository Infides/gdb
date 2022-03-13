use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let x = args.single::<f64>()?;
    let y = args.single::<f64>()?;

    let product = x + y;

    msg.channel_id.say(&ctx.http, product).await?;

    Ok(())
}

#[command]
pub async fn subtract(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let x = args.single::<f64>()?;
    let y = args.single::<f64>()?;

    let product = x - y;

    msg.channel_id.say(&ctx.http, product).await?;

    Ok(())
}

#[command]
pub async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let x = args.single::<f64>()?;
    let y = args.single::<f64>()?;

    let product = x * y;

    msg.channel_id.say(&ctx.http, product).await?;

    Ok(())
}

#[command]
pub async fn divide(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let x = args.single::<f64>()?;
    let y = args.single::<f64>()?;

    let product = x / y;

    msg.channel_id.say(&ctx.http, product).await?;

    Ok(())
}
