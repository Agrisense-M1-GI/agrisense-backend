use tokio_cron_scheduler::{Job, JobScheduler};
use std::sync::Arc;
use crate::config::Config;

pub async fn lancer_scheduler(
    config: Arc<Config>,
    http_client: reqwest::Client,
) {
    let scheduler = JobScheduler::new().await
        .expect("Impossible de créer le scheduler");

    let config_clone  = config.clone();
    let client_clone  = http_client.clone();

    // Déclenche l'analyse des métriques tous les jours à 18h00
    let job = Job::new_async("0 0 18 * * *", move |_uuid, _lock| {
        let config  = config_clone.clone();
        let client  = client_clone.clone();

        Box::pin(async move {
            tracing::info!("⏰ 18h — Déclenchement analyse journalière des métriques");

            let callback_url = format!(
                "http://{}:{}/api/ia/callback/metriques",
                config.server_host,
                config.server_port
            );

            // Configure l'URL source et callback dans le service Python
            // via le déclenchement de /metrics/trigger
            // (le service Python lit BACKEND_METRICS_SOURCE_URL depuis sa config)
            match client
                .post(format!("{}/metrics/trigger", config.python_ai_url))
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
                Ok(res) if res.status().is_success() => {
                    tracing::info!("✅ Analyse métriques déclenchée avec succès");
                }
                Ok(res) => {
                    tracing::error!("❌ Erreur déclenchement métriques : {}", res.status());
                }
                Err(e) => {
                    tracing::error!("❌ Service IA injoignable pour métriques : {}", e);
                }
            }

            // Supprime le callback_url non utilisé — la config côté Python
            // doit pointer vers notre endpoint
            let _ = callback_url;
        })
    })
    .expect("Impossible de créer le job");

    scheduler.add(job).await.expect("Impossible d'ajouter le job");
    scheduler.start().await.expect("Impossible de démarrer le scheduler");

    tracing::info!("⏰ Scheduler démarré — analyse métriques programmée à 18h00 chaque jour");

    // Maintient le scheduler en vie
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}