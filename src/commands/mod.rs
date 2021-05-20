mod ping;
mod doc;

use std::collections::HashSet;
use serenity::{
    prelude::Context,
    framework::standard::{
        Args,
        CommandGroup,
        CommandResult,
        HelpOptions,
        help_commands::{with_embeds as help_with_embeds},
        macros::{
            group,
            help,
        },
    },
    model::{
        id::UserId,
        channel::Message
    },
};

use ping::*;
use doc::*;

#[group]
#[commands(ping, doc)]
pub struct General;

#[help]
#[lacking_permissions  = "Nothing"]
pub async fn exec_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
