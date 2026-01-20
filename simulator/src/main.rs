mod api;
mod llm;
mod server;
mod snapshots;
mod x_api;

use clap::{Args, Parser, Subcommand};
use std::io::{self, Read};
use std::path::Path;
use virality_sim::{
    format_float, format_number, format_percent, simulate_with_llm, MediaType, SimulatorInput,
};

#[derive(Parser)]
#[command(name = "virality-sim", about = "Tweet virality simulator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Simulate(SimulateArgs),
    Serve(ServeArgs),
}

#[derive(Args, Debug, Clone)]
struct SimulateArgs {
    #[arg(long)]
    text: Option<String>,
    #[arg(long, default_value_t = 1000)]
    followers: u64,
    #[arg(long, default_value_t = 500)]
    following: u64,
    #[arg(long, default_value_t = 365)]
    account_age_days: u32,
    #[arg(long, default_value_t = 0.02)]
    avg_engagement_rate: f64,
    #[arg(long, default_value_t = 2.0)]
    posts_per_day: f64,
    #[arg(long)]
    verified: bool,
    #[arg(long, default_value_t = 12)]
    hour: u8,
    #[arg(long, default_value = "none")]
    media: String,
    #[arg(long, conflicts_with = "no_link")]
    link: bool,
    #[arg(long, conflicts_with = "link")]
    no_link: bool,
    #[arg(long, default_value_t = 0.5)]
    novelty: f64,
    #[arg(long, default_value_t = 0.5)]
    timeliness: f64,
    #[arg(long, default_value_t = 0.5)]
    topic_saturation: f64,
    #[arg(long, default_value_t = 0.6)]
    audience_fit: f64,
    #[arg(long, default_value_t = 0.3)]
    controversy: f64,
    #[arg(long, default_value_t = 0.1)]
    sentiment: f64,
    #[arg(long)]
    ai: bool,
    #[arg(long)]
    ai_model: Option<String>,
    #[arg(long)]
    details: bool,
}

impl Default for SimulateArgs {
    fn default() -> Self {
        Self {
            text: None,
            followers: 1000,
            following: 500,
            account_age_days: 365,
            avg_engagement_rate: 0.02,
            posts_per_day: 2.0,
            verified: false,
            hour: 12,
            media: "none".to_string(),
            link: false,
            no_link: false,
            novelty: 0.5,
            timeliness: 0.5,
            topic_saturation: 0.5,
            audience_fit: 0.6,
            controversy: 0.3,
            sentiment: 0.1,
            ai: false,
            ai_model: None,
            details: false,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 8787)]
    port: u16,
    #[arg(long, default_value = "../webapp/dist")]
    web_root: String,
}

#[tokio::main]
async fn main() {
    load_dotenv();
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Simulate(SimulateArgs::default()));

    match command {
        Command::Simulate(args) => run_simulate(args).await,
        Command::Serve(args) => server::serve(args).await,
    }
}

async fn run_simulate(args: SimulateArgs) -> Result<(), String> {
    let mut input = SimulatorInput::default();
    input.followers = args.followers;
    input.following = args.following;
    input.account_age_days = args.account_age_days;
    input.avg_engagement_rate = args.avg_engagement_rate;
    input.posts_per_day = args.posts_per_day;
    input.verified = args.verified;
    input.hour_of_day = validate_hour(args.hour)?;
    input.novelty = args.novelty;
    input.timeliness = args.timeliness;
    input.topic_saturation = args.topic_saturation;
    input.audience_fit = args.audience_fit;
    input.controversy = args.controversy;
    input.sentiment = args.sentiment;

    if args.link {
        input.has_link_override = Some(true);
    }
    if args.no_link {
        input.has_link_override = Some(false);
    }

    input.media = MediaType::from_str(&args.media)
        .ok_or_else(|| format!("invalid media type: {}", args.media))?;

    let text = read_text(args.text)?;
    input.text = text;

    let llm_result = if args.ai {
        let client = llm::LlmClient::from_env(args.ai_model)
            .ok_or_else(|| "XAI_API_KEY is not set".to_string())?;
        Some(client.score_text(&input.text).await?)
    } else {
        None
    };

    let output = simulate_with_llm(
        &input,
        llm_result.as_ref().map(|result| &result.score),
        llm_result.as_ref().map(|result| &result.trace),
    );

    println!(
        "Virality score: {} ({})",
        format_float(output.score, 1),
        output.tier.label()
    );
    println!("Weighted score: {}", format_float(output.weighted_score, 2));
    println!(
        "Estimated impressions: {} (in-network {} | out-of-network {})",
        format_number(output.impressions_total),
        format_number(output.impressions_in),
        format_number(output.impressions_oon)
    );
    println!(
        "Expected engaged users: {} ({})",
        format_number(output.expected_unique_engagements),
        format_percent(output.unique_engagement_rate)
    );
    println!(
        "Total action volume: {} ({} actions per impression)",
        format_number(output.expected_action_volume),
        format_float(output.action_volume_rate, 2)
    );

    let likes = output.impressions_total * output.actions.like;
    let replies = output.impressions_total * output.actions.reply;
    let reposts = output.impressions_total * output.actions.repost;
    let shares = output.impressions_total * output.actions.share;

    println!(
        "Action volume: likes {} | replies {} | reposts {} | shares {}",
        format_number(likes),
        format_number(replies),
        format_number(reposts),
        format_number(shares)
    );

    println!(
        "Signals: quality {} | hook {} | author {} | negative risk {} | shareability {}",
        format_float(output.signals.content_quality, 2),
        format_float(output.signals.hook, 2),
        format_float(output.signals.author_quality, 2),
        format_float(output.signals.negative_risk, 2),
        format_float(output.signals.shareability, 2)
    );

    if let Some(llm) = output.llm {
        println!(
            "AI scores: hook {} | clarity {} | novelty {} | shareability {} | controversy {} | sentiment {}",
            format_float(llm.hook, 2),
            format_float(llm.clarity, 2),
            format_float(llm.novelty, 2),
            format_float(llm.shareability, 2),
            format_float(llm.controversy, 2),
            format_float(llm.sentiment, 2)
        );
    }

    if args.details {
        println!("\nAction probabilities:");
        println!("  like: {}", format_percent(output.actions.like));
        println!("  reply: {}", format_percent(output.actions.reply));
        println!("  repost: {}", format_percent(output.actions.repost));
        println!("  quote: {}", format_percent(output.actions.quote));
        println!("  share: {}", format_percent(output.actions.share));
        println!("  click: {}", format_percent(output.actions.click));
        println!("  profile_click: {}", format_percent(output.actions.profile_click));
        println!("  follow_author: {}", format_percent(output.actions.follow_author));
        println!("  video_view: {}", format_percent(output.actions.video_view));
        println!("  photo_expand: {}", format_percent(output.actions.photo_expand));
        println!("  dwell: {}", format_percent(output.actions.dwell));
        println!("  not_interested: {}", format_percent(output.actions.not_interested));
        println!("  mute: {}", format_percent(output.actions.mute));
        println!("  block: {}", format_percent(output.actions.block));
        println!("  report: {}", format_percent(output.actions.report));
    }

    if !output.suggestions.is_empty() {
        println!("\nSuggestions:");
        for suggestion in output.suggestions {
            println!("- {}", suggestion);
        }
    }

    Ok(())
}

fn read_text(arg: Option<String>) -> Result<String, String> {
    if let Some(text) = arg {
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|err| format!("failed reading stdin: {}", err))?;
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Err("missing tweet text: pass --text or pipe stdin".to_string());
    }
    Ok(trimmed.to_string())
}

fn validate_hour(value: u8) -> Result<u8, String> {
    if value > 23 {
        return Err(format!("invalid hour (0-23): {}", value));
    }
    Ok(value)
}

fn load_dotenv() {
    let _ = dotenvy::dotenv();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = Path::new(manifest_dir).join(".env");
    let _ = dotenvy::from_path(manifest_path);
}
