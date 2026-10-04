use serde_json::{json, Value};
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn milliseconds(value: &Value) -> Result<u64, Box<dyn Error>> {
    let parts = value
        .as_str()
        .ok_or("Missing timestamp")?
        .split([':', ','])
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.len() != 4 {
        return Err("Invalid timestamp".into());
    }
    Ok((parts[0] * 3600 + parts[1] * 60 + parts[2]) * 1000 + parts[3])
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(
        env::args_os()
            .nth(1)
            .ok_or("Usage: screenshot-fixture <local media repository>")?,
    );
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("package-work/screenshots");
    fs::create_dir_all(&output)?;
    let mut fixtures = Vec::new();
    for entry in fs::read_dir(env::temp_dir())? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("bibiparrot-analysis-mp4-e2e-")
        {
            let file = entry.path().join("segmentations/segments.json");
            if file.is_file() {
                fixtures.push((fs::metadata(&file)?.modified()?, file));
            }
        }
    }
    fixtures.sort();
    let latest = &fixtures
        .last()
        .ok_or("Run the real MP4 analysis test first")?
        .1;
    let mut sentences: Vec<Value> = serde_json::from_str(&fs::read_to_string(latest)?)?;
    sentences.retain(|sentence| {
        !sentence["text"]
            .as_str()
            .unwrap_or_default()
            .starts_with(['[', '('])
    });
    for sentence in &mut sentences {
        sentence["start_ms"] = json!(milliseconds(&sentence["start"])?);
        sentence["end_ms"] = json!(milliseconds(&sentence["end"])?);
        for word in sentence["words"].as_array_mut().ok_or("Missing words")? {
            word["start_ms"] = json!(milliseconds(&word["start"])?);
            word["end_ms"] = json!(milliseconds(&word["end"])?);
        }
    }
    let notes = "# 听懂一句，记住一词\n\n## 今日练习：自然地打招呼\n\n[Hi, how are you?](bibi://sentence/s001)\n\n先听一遍，再试着写下来。难点可以单独回听。\n\n## 记住这些表达\n\n**How are you?** 你好吗？\n\n**How's it going?** 最近怎么样？\n\n**What are you up to?** 最近在忙什么？\n\n## 我的听写\n\nHi, how are you?\n\nHey, how's it going?\n\n## 复习时，双击蓝色文字\n\n[Hey, how's it going?](bibi://sentence/s002)\n";
    let video = root.join("How are you - Relax - Phrases ( lingoneo.org ).mp4");
    let lesson = json!({"id":"daily-english", "title":"Daily English · 问候与近况", "source_path":video, "playback_path":video, "workspace_path":output.join("daily-english"), "kind":"Video", "duration_ms":226040, "sentences":sentences, "markdown":notes, "analyzed":true});
    let mut sample: Value = serde_json::from_str(&fs::read_to_string(
        root.join("first_snowfall_bp/workspace.json"),
    )?)?;
    sample["id"] = json!("first-snowfall");
    sample["title"] = json!("First snowfall · 每日听写");
    sample["source_path"] = json!(root.join("first_snowfall.mp3"));
    sample["playback_path"] = sample["source_path"].clone();
    sample["workspace_path"] = json!(output.join("first-snowfall"));
    sample["markdown"] = json!(notes
        .replace("Hi, how are you?", "Today is November 26th.")
        .replace(
            "[Hey, how's it going?](bibi://sentence/s002)",
            "[Today is November 26th.](bibi://sentence/s001)"
        ));
    sample["analyzed"] = json!(true);
    let project = json!({"version":6, "active_lesson":0, "selected_sentence":0, "selected_word":0, "loop_mode":"Off", "playback_rate":0.75, "volume":80, "lessons":[lesson,sample]});
    fs::write(
        output.join("library-v2.json"),
        serde_json::to_string_pretty(&project)?,
    )?;
    println!("{}", output.display());
    Ok(())
}
