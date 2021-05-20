use async_trait::async_trait;

use serenity::{
    prelude::Context,
    model::prelude::{
        GuildChannel,
        ChannelType
    },
};

use chrono::{
    MIN_DATE,
    prelude::{
        Datelike,
        Date,
        Utc,
    },
};

#[async_trait]
pub trait DocSection {
    async fn doc(&self, ctx: &Context) -> Option<String>;
    async fn doc_as_subsection(&self, ctx: &Context) -> Option<String>
        where Self: Sized
    {
        let subdoc = self.doc(ctx).await?;
        if subdoc.is_empty() {
            Some(subdoc)
        }
        else {
            Some(format!("#{}", subdoc))
        }
    }
    fn empty_doc(&self) -> String {
        String::default()
    }
}

pub struct Doc {
    start_date: Date<Utc>,
    end_date: Date<Utc>,
    sections: Vec<Box<dyn DocSection + Send + Sync>>,
}

impl Doc {
    pub fn new(start_date: Date<Utc>, end_date: Date<Utc>) -> Self {
        Self { start_date, end_date, sections: Vec::new() }
    }

    fn render_date(date: &Date<Utc>, with_year: bool) -> String {
        date.format(format!(
            "%b %-d{}{}", 
            match date.day() {
                1 | 21 | 31 => "st",
                2 | 22 => "nd",
                3 | 23  => "rd",
                _ => "th",
            }, 
            if with_year {", %Y"} else {""}
        ).as_str()).to_string()
    }

    pub fn render_date_range(&self) -> String {
        let same_year = self.start_date.year() == self.end_date.year();
        let end_is_today = self.end_date.year() == Utc::today().year();
        let since_beginning = self.start_date == MIN_DATE;
        let formatted_start = if since_beginning { 
            String::from("Up until")
        } else {
            format!("{} -", Doc::render_date(&self.start_date, !same_year))
        };
        format!("{} {}", formatted_start, Doc::render_date(&self.end_date, !(same_year || (since_beginning && end_is_today))))
    }

    pub fn add_channel(&mut self, channel: GuildChannel) {
        self.sections.push(match channel.kind {
            ChannelType::Text => Box::new(ChannelSection::new(channel, self.start_date, self.end_date, false)),
            ChannelType::Category => Box::new(CategorySection::new(channel, self.start_date, self.end_date)),
            _ => {return}
        })
    }
}

#[async_trait]
impl DocSection for Doc {
    async fn doc(&self, ctx: &Context) -> Option<String> {
        let mut doc = vec![format!("Important Messages: {}", self.render_date_range())];
        doc.reserve(1 + self.sections.len());
        doc.push("=".repeat(doc.get(0).unwrap_or(&String::default()).len()));
        for section in &self.sections {
            doc.push(section.doc(ctx).await?);
        }
        Some(doc.join("\n"))
    }
}


struct ChannelSection {
    channel: GuildChannel,
    start_date: Date<Utc>,
    end_date: Date<Utc>,
    is_sub: bool,
}

impl ChannelSection {
    fn new(channel: GuildChannel, start_date: Date<Utc>, end_date: Date<Utc>, is_sub: bool) -> Self {
        Self { channel, start_date, end_date, is_sub }
    }
}

#[async_trait]
impl DocSection for ChannelSection {
    fn empty_doc(&self) -> String {
        format!("Nothing interesting, but check the channel for [more discussions](https://discord.com/channels/{}/{})", self.channel.guild_id.as_u64(), self.channel.id.as_u64())
    }

    async fn doc(&self, ctx: &Context) -> Option<String> {
        let mut doc = vec![format!("## {}", self.channel.name)];
        let msgs: Vec<_> = self.channel.pins(ctx).await.ok()?
                                       .into_iter()
                                       .filter(|m| (self.start_date..self.end_date).contains(&m.timestamp.date()))
                                       .collect();
        doc.reserve(msgs.len() + 2);
        for msg in &msgs {
            doc.push(format!("* {} ([source]({}))", msg.content_safe(ctx).await, msg.link_ensured(ctx).await))
        }
        if msgs.is_empty() {
            if self.is_sub { return Some(String::default()) }
            doc.push(self.empty_doc());
        }
        doc.push(String::default());
        Some(doc.join("\n"))
    }
}


struct CategorySection {
    channel: GuildChannel,
    start_date: Date<Utc>,
    end_date: Date<Utc>,
}

impl CategorySection {
    fn new(channel: GuildChannel, start_date: Date<Utc>, end_date: Date<Utc>) -> Self {
        Self { channel, start_date, end_date }
    }
}

#[async_trait]
impl DocSection for CategorySection {
    fn empty_doc(&self) -> String {
        format!("Nothing interesting, but check the channel for [more discussions](https://discord.com/channels/{}/{})", self.channel.guild_id.as_u64(), self.channel.id.as_u64())
    }

    async fn doc(&self, ctx: &Context) -> Option<String> {
        let mut doc = vec![format!("## {}\n", self.channel.name)];
        let subsections: Vec<_> = if let Ok(channels) = self.channel.guild_id.channels(&ctx.http).await {
            channels.into_values()
                    .filter_map(|c| 
                        if c.category_id == Some(self.channel.id) && c.kind == ChannelType::Text { 
                            Some(ChannelSection::new(c, self.start_date, self.end_date, true)) 
                        }
                        else { 
                            None 
                        })
                    .collect()
        }
        else { 
            return None;
        };
        doc.reserve(2+subsections.len());
        for section in &subsections {
            match section.doc_as_subsection(ctx).await.filter(|d| !d.is_empty()) {
                Some(subdoc) => doc.push(subdoc),
                None => { continue; }
            }
        }
        if doc.len() == 1 {
            doc.push(self.empty_doc());
            doc.push(String::default());
        }
        Some(doc.join(""))
    }
}
