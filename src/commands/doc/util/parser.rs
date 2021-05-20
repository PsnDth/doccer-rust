use serenity::{
    prelude::Context,
    model::prelude::{
        Guild,
        GuildChannel,
        ChannelId,
        ChannelType
    },
};

use std::collections::HashSet;

use regex::Regex;
use lazy_static::lazy_static;

use date_time_parser::DateParser;
use chrono::prelude::{
    Date,
    Utc,
};

#[derive(Clone)]
enum ChannelMatch {
    NoMatch,
    SoftMatch(Option<ChannelId>),
    HardMatch(Option<ChannelId>),
}

pub fn parse_date(date_arg: &String) -> Option<Date<Utc>> {
    if let Some(date) = DateParser::parse(date_arg) {
        Some(Date::from_utc(date, Utc))
    } else {
        None
    }
}

fn _channel_to_match(channel: &GuildChannel, prev_match: &ChannelMatch, name: &String) -> ChannelMatch {
    let channel_name = channel.name.to_lowercase();
    let curr_match = if channel_name == *name {
        ChannelMatch::HardMatch(Some(channel.id))
    }
    else if channel_name.contains(name) {
        ChannelMatch::SoftMatch(Some(channel.id))
    }
    else {
        ChannelMatch::NoMatch
    };
    match channel.kind {
        ChannelType::Text | ChannelType::Category => match (prev_match, &curr_match) {
            (ChannelMatch::NoMatch, _) | (ChannelMatch::SoftMatch(_), ChannelMatch::HardMatch(_)) => curr_match,
            (ChannelMatch::SoftMatch(_), ChannelMatch::SoftMatch(_)) => ChannelMatch::SoftMatch(None),
            (ChannelMatch::HardMatch(_), ChannelMatch::HardMatch(_)) => ChannelMatch::HardMatch(None),
            (orig_val, _) => orig_val.clone()
        },
        _ => ChannelMatch::NoMatch
    }

}

async fn find_channel_in_guild(ctx: &Context, guild: &Guild, channel: &String) -> Option<ChannelId> {
    let mut text_match = ChannelMatch::NoMatch;
    let mut category_match = ChannelMatch::NoMatch;
    let mut seen_categories = HashSet::new();
    let clean_name =  channel.to_lowercase();

    for channel in guild.channels.values() {
        text_match = _channel_to_match(&channel.clone(), &text_match, &clean_name);
        
        if let Some(category_id) = channel.category_id {
            if seen_categories.contains(&category_id) {
                continue;
            }
            let category = match category_id.to_channel(ctx).await {
                Ok(c) => if let Some(c) = c.guild() { c } else { continue; },
                _ => { continue; }
            };
            category_match = _channel_to_match(&category, &category_match, &clean_name);
            seen_categories.insert(category_id);
        }
    }

    match (text_match, category_match) {
        (ChannelMatch::HardMatch(Some(val)), _) | (ChannelMatch::SoftMatch(Some(val)), _) |
        (_, ChannelMatch::HardMatch(Some(val))) | (_, ChannelMatch::SoftMatch(Some(val))) => Some(val),
        _ => None
    }
}

pub async fn parse_channel(ctx: &Context, guild: &Guild, channel_arg: &String) -> Option<GuildChannel> {
    lazy_static! {
        static ref ID_PATTERN: Regex = Regex::new(r"^(?:<#)?(?P<id>[0-9]+)>?$").unwrap();
    }
    println!("Parsing channel {:?}", channel_arg);
    if let Some(cid_capture) = ID_PATTERN.captures(channel_arg) {
        let channel = ChannelId(cid_capture["id"].parse().unwrap_or(0)).to_channel(ctx).await.ok()?.guild()?;
        if channel.guild_id != guild.id {
            return None;
        }
        match channel.kind {
            ChannelType::Text | ChannelType::Category => Some(channel),
            _ => None
        }
    }
    else if let Some(channel_id) = find_channel_in_guild(&ctx, &guild, &channel_arg).await {
        // No need for complex checks since this channel will be of the correct types, from the guild
        channel_id.to_channel(ctx).await.ok()?.guild()
    }
    else {
        None
    }
}