async fn fetch_releases_list() -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let github_token =
        std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set in .env file");
    let mut completed_fetching_releases_list = false;
    let mut result = Vec::new();
    let mut releases_page = 1;

    while !completed_fetching_releases_list {
        let response = reqwest::Client::new()
            .get(format!(
                "https://api.github.com/repos/godotengine/godot-builds/releases?per_page=100&page={}",
                releases_page
            ))
            .header(reqwest::header::USER_AGENT, "gdvm")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("token {}", github_token),
            )
            .send()
            .await?;
        let response_data: serde_json::Value = response.json().await?;

        if let Some(vec) = response_data.as_array() {
            if vec.len() < 100 {
                completed_fetching_releases_list = true;
            } else {
                releases_page += 1;
            }

            result.extend(vec.iter().cloned());
        }
    }

    Ok(result)
}

fn format_releases_list(releases_list: Vec<serde_json::Value>) -> Vec<(u32, u32, u32, String)> {
    let versions: Vec<String> = releases_list
        .iter()
        .filter(|release| {
            !release
                .get("prerelease")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
                && !release
                    .get("draft")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
        })
        .filter_map(|release| {
            release
                .get("tag_name")?
                .as_str()
                .map(|s| s.replace("-stable", ""))
        })
        .collect();

    let mut version_tuples: Vec<(u32, u32, u32, String)> = versions
        .iter()
        .map(|v| {
            let parts: Vec<&str> = v.split('.').collect();
            let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
            let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            (major, minor, patch, v.clone())
        })
        .collect();

    version_tuples.sort_by(|a, b| b.cmp(a));

    version_tuples
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let releases_list = fetch_releases_list().await?;
    let version_tuples = format_releases_list(releases_list);

    let mut result = Vec::new();
    let mut current_major = None;
    let mut count_in_group = 0;

    const MAX_VERSIONS_GROUPED: i32 = 3;

    for (major, _, _, version) in version_tuples {
        if Some(major) != current_major {
            current_major = Some(major);
            count_in_group = 0;
        }

        if count_in_group < MAX_VERSIONS_GROUPED {
            result.push(version);
            count_in_group += 1;
        }
    }

    println!("\n");
    result.iter().for_each(|version| println!("{}", version));
    println!(
        "\nThis is a partial list. For a complete list, visit: https://godotengine.org/download/archive/"
    );

    Ok(())
}
