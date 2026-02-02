use reqwest::Client;

pub async fn fetch_processes(agent_url: &str) -> anyhow::Result<String> {
    let res = Client::new()
        .get(format!("{}/processes", agent_url))
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}
