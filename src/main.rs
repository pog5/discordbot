use serenity::async_trait;
use serenity::model::prelude::UserId;
use serenity::prelude::*;
use serenity::model::user::User;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::StandardFramework;
use reqwest::*;
use serde::{Deserialize, Serialize};
use serenity::json::prelude::from_value;
use serenity::json::Value;
use rand::prelude::SliceRandom;
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;

#[derive(Debug, Deserialize, Serialize)]
struct GifObject {
    media: Vec<MediaObject>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MediaObject {
    gif: GifData,
}

#[derive(Debug, Deserialize, Serialize)]
struct GifData {
    url: String,
}

// random cat gif from tenor
async fn get_random_cat_gif() -> Result<String> {
    let tenorkey = "LIVDSRZULELA"; // taken from tenor docs, todo change into env var soon 
    let query = "cat";
    let url = format!("https://g.tenor.com/v1/search?q={}&key={}&limit=50", query, tenorkey);
    let response = reqwest::get(&url).await?;
    let json = response.json::<Value>().await?;

    let gif_objects: Vec<GifObject> = json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| from_value(result.clone()).unwrap())
        .collect();

    let random_gif = gif_objects.choose(&mut rand::thread_rng()).unwrap();
    let gif_url = &random_gif.media[0].gif.url;

    Ok(gif_url.to_string())
}

#[group]
#[commands(ping)]
#[group]
#[commands(meow)]
#[group]
#[commands(daily)]
#[group]
#[commands(bal)]
#[group]
#[commands(pay)]

struct General;

struct Handler;

pub struct UserCoinCache;

impl TypeMapKey for UserCoinCache {
    type Value = Arc<Mutex<HashMap<UserId, u32>>>;
}

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let token = std::env::var("TOKEN").expect("Please set the bot token as the TOKEN environment variable.");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut bot = serenity::Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = bot.data.write().await;
        data.insert::<UserCoinCache>(Arc::new(Mutex::new(HashMap::default())));
    }

    if let Err(why) = bot.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[command]
async fn meow(ctx: &Context, msg: &Message) -> CommandResult {
    let catlink = get_random_cat_gif().await?;
    let catlink_string = format!("{}", catlink);
    msg.reply(ctx, &catlink_string).await?;
    Ok(())
}

#[command]
async fn bal(ctx: &Context, msg: &Message) -> CommandResult {
    let data_read = ctx.data.read().await;
    let coin_cache = data_read.get::<UserCoinCache>().expect("exception has occured").clone();
    let hash_map = coin_cache.lock().await;
    
    let balance = hash_map.get(&msg.author.id).unwrap_or(&0);
    let response = format!("Your balance is **{}** coins", balance);
    msg.reply(ctx, &response).await?;
    Ok(())
}


#[command]
async fn pay(ctx: &Context, msg: &Message) -> CommandResult {
    let regexid = Regex::new(r"^\d{18}$").unwrap();
    let recieverid = assert!(regexid.is_match(&msg.content));
    let recievername = UserId::to_user_cached(recieverid);
    let response = format!("Sent <@{}> **{}** coins", recievername, coins);
    println!("{} coins sent from {} to {}", coins, msg.author.id, recieverid);
    msg.reply(ctx, response).await?;
    Ok(())
}

 
#[command]
async fn daily(ctx: &Context, msg: &Message) -> CommandResult {
    let amount = rand::thread_rng().gen_range(20..100);
    let data_read = ctx.data.read().await;
    let coin_cache = data_read.get::<UserCoinCache>().expect("exception has occured").clone();
    let mut hash_map = coin_cache.lock().await;
    let lock = hash_map.entry(msg.author.id).or_insert(0);
    *lock += amount;
    let response = format!("You have received **{}** coins for today!", amount);
    msg.reply(ctx, &response).await?;
    Ok(())
}


