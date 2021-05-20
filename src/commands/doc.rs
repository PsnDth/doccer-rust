mod util;

use serenity::{
    prelude::Context,
    model::prelude::Message,
    framework::standard::{
        Args,
        CommandResult,
        macros::command,
    },
    http::AttachmentType,
};

use chrono::{
    MIN_DATE,
    prelude::Utc,
};

use util::{
    parser::{
        parse_date,
        parse_channel,
    },
    doccer::{
        Doc,
        DocSection,
    },
};



#[command]
#[only_in(guilds)]
#[required_permissions(MANAGE_MESSAGES)]
#[description = "Parse messages in provided date range and extracts important (pinned) ones"]
#[usage = "doc [<start_date> [<end_date>]] [<channels> ...]"]
#[example("doc 1/1")]
#[example("doc \"Jan 1st\" \"May 30th\"")]
#[example("doc 1/1 #channel1 #channel2")]
#[example("doc 1/1 3/31 123123123123")]
async fn doc(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (start_date, end_date) = match (
        parse_date(&args.single().unwrap_or("".to_string())),
        parse_date(&args.single().unwrap_or("".to_string()))
    ) {
        (Some(parsed_start), Some(parsed_end)) => (parsed_start, parsed_end),
        (Some(parsed_start), None) => {
            args.rewind();
            (parsed_start, Utc::today())
        },
        _ => {
            args.rewind().rewind();
            (MIN_DATE, Utc::today())
        }
    };
    // args.iter().map(|a| a.value());
    let mut doc = Doc::new(start_date, end_date);
    if let Some(guild) = msg.guild(&ctx.cache).await {
        for carg in args.iter::<String>() {
            let channel_arg = carg.unwrap_or("".to_string());
            if let Some(channel) = parse_channel(&ctx, &guild, &channel_arg).await {
                doc.add_channel(channel);
            }
            else {
                msg.reply(&ctx, &format!("Found invalid channel argument => {:?}", &channel_arg)).await?;
                return Ok(());
            }
        }
    }

    match doc.doc(&ctx).await {
        Some(doc) => msg.channel_id.send_message(&ctx.http, |mut builder| {
                builder = builder
                    .reference_message(msg)
                    .allowed_mentions(|f| f.replied_user(true));
                builder.content("Here's the generated summary doc");
                builder.add_file(AttachmentType::from((doc.as_bytes(), "summary_doc.md")));
                builder
            }).await?,
        None => msg.reply_ping(&ctx, &format!("Couldn't create a summary document for date range {:?}", doc.render_date_range())).await?,
    };
    
    Ok(())
}