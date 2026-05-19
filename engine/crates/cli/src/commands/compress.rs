use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::Color;
use first_plan_core::compress::{run_and_compress, CompressedOutput};
use std::io::{self, Read};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long)]
    tool: String,

    #[arg(long)]
    raw_stdin: bool,

    #[arg(long, default_value = "")]
    raw_label: String,

    #[arg(long, default_value = "-")]
    output_json: String,

    #[arg(long)]
    json: bool,

    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

pub fn run(args: Args) -> Result<()> {
    let mode = output_mode(args.json || args.output_json != "-");

    let result = if args.raw_stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        let raw_bytes = buf.len();
        let compressed = first_plan_core::compress::compress(&args.tool, &buf, "");
        let compressed_bytes = compressed.len();
        let savings_pct = if raw_bytes == 0 {
            0.0
        } else {
            (1.0 - compressed_bytes as f32 / raw_bytes as f32) * 100.0
        };
        CompressedOutput {
            tool: args.tool.clone(),
            raw_bytes,
            compressed_bytes,
            savings_pct,
            exit_code: 0,
            output: compressed,
        }
    } else {
        run_and_compress(&args.tool, &args.args)?
    };

    if mode == OutputMode::Pretty {
        render_pretty(&result, &args.raw_label);
        Ok(())
    } else {
        let json = serde_json::to_string_pretty(&result)?;
        if args.output_json == "-" {
            println!("{}", json);
        } else {
            std::fs::write(&args.output_json, json)?;
        }
        Ok(())
    }
}

fn render_pretty(result: &CompressedOutput, label: &str) {
    let title = if label.is_empty() {
        format!("Compressed: {}", result.tool)
    } else {
        format!("Compressed: {} ({})", result.tool, label)
    };
    print_header(&title);
    print_section("Stats");
    print_kv("Raw bytes", &result.raw_bytes.to_string(), Color::White);
    print_kv(
        "Compressed bytes",
        &result.compressed_bytes.to_string(),
        Color::White,
    );
    print_kv_bold(
        "Savings",
        &format!("{:.1}%", result.savings_pct),
        if result.savings_pct >= 50.0 {
            Color::Green
        } else if result.savings_pct >= 20.0 {
            Color::Yellow
        } else {
            Color::DarkGrey
        },
    );
    print_kv("Exit code", &result.exit_code.to_string(), Color::DarkGrey);

    print_section("Output");
    println!();
    println!("{}", result.output);
    flush();
}
