use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::num::ParseIntError;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use tokio::sync::RwLock;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{ArgError, Args, CommandResult};
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::prelude::Message;
use serenity::model::prelude::ReactionType;
use serenity::model::user::User;
use serenity::prelude::*;

use tracing::{error, info};

use lazy_static::lazy_static;

use nom::{
    bytes::complete::{tag, take_until1},
    IResult,
};

lazy_static! {
    static ref LAST_LC: RwLock<String> = RwLock::new(String::default());
    static ref LAST_SRIRACHA_EMBED_MESSAGE: RwLock<Option<Message>> = RwLock::new(None);
    static ref BOTS: HashMap<&'static str, u64> = HashMap::from([
        ("sriracha", 607661949194469376),
        ("ohsheet", 640402425395675178),
        ("lc", 661826254215053324),
        ("fort checker", 1014282115086565486)
    ]);
}

fn is_sriracha_bot(user: &User) -> bool {
    vec![BOTS.get("sriracha").unwrap(), BOTS.get("ohsheet").unwrap()].contains(&user.id.as_u64())
}

fn is_lc_bot(user: &User) -> bool {
    vec![
        BOTS.get("ohsheet").unwrap(),
        BOTS.get("lc").unwrap(),
        BOTS.get("fort checker").unwrap(),
    ]
    .contains(&user.id.as_u64())
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

fn author_get(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("Looking up ")(input)?;
    let (input, _) = take_until1(" by ")(input)?;
    let (input, _) = tag(" by ")(input)?;
    let (input, author) = take_until1(".")(input)?;

    Ok((input, author))
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if is_sriracha_bot(&msg.author) {
            if msg.content.starts_with(".lc") {
                let mut last_lc = LAST_LC.write().await;
                *last_lc = msg.content.clone();
            } else if msg.embeds.first().is_some() {
                let message_id = msg.id;
                {
                    let mut last_sriracha_embed_message = LAST_SRIRACHA_EMBED_MESSAGE.write().await;
                    *last_sriracha_embed_message = Some(msg);
                }
                info!("Last sriracha embed message: {}", message_id);
            }
        } else if is_lc_bot(&msg.author) && msg.content.starts_with("Looking up") {
            match author_get(&msg.content) {
                Ok((_, author)) => {
                    sleep(Duration::from_secs(3));
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, format!("sauce -qa {author}"))
                        .await;
                }
                Err(_) => {
                    let _ = msg.channel_id.say(&ctx.http, "Could not find author").await;
                }
            }
        }
    }
}

#[group]
#[commands(en, jp)]
struct General;

#[group]
#[prefix = "lc"]
#[commands(lc_list, lc_move, lc_delete, lc_retry)]
struct Lc;

#[group]
#[prefix = "st"]
#[commands(st_list, st_move, st_delete)]
struct St;

#[group]
#[prefix = "qc"]
#[commands(qc_list, qc_move, qc_delete)]
struct Qc;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in environment");
    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access app info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("*"))
        .group(&GENERAL_GROUP)
        .group(&LC_GROUP)
        .group(&ST_GROUP)
        .group(&QC_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

fn get_id(mut args: Args) -> Result<u32, ArgError<ParseIntError>> {
    if args.is_empty() {
        Ok(1)
    } else {
        args.single::<u32>()
    }
}

#[command]
#[aliases("")]
async fn lc_list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce lc 3#{id}"))
        .await?;

    Ok(())
}

#[command]
#[aliases("move")]
async fn lc_move(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce move 3#{id} 4"))
        .await?;

    Ok(())
}

#[command]
#[aliases("del", "delet", "delete")]
async fn lc_delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce delete 3#{id}"))
        .await?;

    Ok(())
}

#[command]
#[aliases("retry")]
async fn lc_retry(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let retried_message = LAST_LC.read().await.clone();

    msg.channel_id.say(&ctx.http, retried_message).await?;

    Ok(())
}

#[command]
#[aliases("")]
async fn st_list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce 2#{id}"))
        .await?;

    Ok(())
}

#[command]
#[aliases("move")]
async fn st_move(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce move 2#{id} 3"))
        .await?;

    Ok(())
}

#[command]
#[aliases("del", "delet", "delete")]
async fn st_delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce delete 2#{id}"))
        .await?;

    Ok(())
}

#[command]
#[aliases("")]
async fn qc_list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce 1#{id}"))
        .await?;

    Ok(())
}

#[command]
#[aliases("move")]
async fn qc_move(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce move 1#{id} 2"))
        .await?;

    Ok(())
}

#[command]
#[aliases("del", "delet", "delete")]
async fn qc_delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let id = get_id(args)?;
    msg.channel_id
        .say(&ctx.http, format!("sauce delete 1#{id}"))
        .await?;

    Ok(())
}

#[command]
async fn en(ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let last_sriracha_embed_message = LAST_SRIRACHA_EMBED_MESSAGE.read().await;

    if let Some(real_message) = &*last_sriracha_embed_message {
        let _ = real_message
            .delete_reaction_emoji(ctx, ReactionType::from_str("🇺🇸").unwrap())
            .await;
        real_message
            .react(ctx, ReactionType::from_str("🇺🇸").unwrap())
            .await?;
    }
    

    Ok(())
}

#[command]
async fn jp(ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let last_sriracha_embed_message = LAST_SRIRACHA_EMBED_MESSAGE.read().await;

    if let Some(real_message) = &*last_sriracha_embed_message {
        let _ = real_message
            .delete_reaction_emoji(ctx, ReactionType::from_str("🇯🇵").unwrap())
            .await;
        real_message
            .react(ctx, ReactionType::from_str("🇯🇵").unwrap())
            .await?;
    }

    Ok(())
}
