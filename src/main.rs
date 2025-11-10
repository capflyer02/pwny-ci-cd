use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    client: Client,
    wu_api_key: String,
}

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let wu_api_key =
        env::var("WU_API_KEY").expect("WU_API_KEY environment variable must be set");

    let state = AppState {
        client: Client::new(),
        wu_api_key,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/weather", get(get_weather))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ========== UI ==========

async fn index() -> Html<&'static str> {
    // Simple dark UI with JS calling /api/weather
    Html(r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>PWS Weather Viewer</title>
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    body { background:#050816; color:#e5e7eb; font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif; margin:0; padding:0; }
    .wrap { max-width:720px; margin:40px auto; padding:24px; background:#0b1020; border-radius:18px; box-shadow:0 18px 45px rgba(0,0,0,0.65); }
    h1 { font-size:1.9rem; margin-bottom:0.5rem; display:flex; gap:0.4rem; align-items:center; }
    h1 span.logo { font-size:1.6rem; }
    p.sub { color:#9ca3af; margin-top:0; margin-bottom:1.5rem; }
    label { display:block; margin-bottom:0.4rem; color:#9ca3af; }
    input { width:100%; padding:0.55rem 0.7rem; border-radius:0.6rem; border:1px solid #374151; background:#020817; color:#e5e7eb; }
    button { margin-top:0.75rem; padding:0.55rem 1.1rem; border:none; border-radius:0.6rem; background:#2563eb; color:white; cursor:pointer; font-weight:500; }
    button:hover { background:#1d4ed8; }
    .result, .error { margin-top:1.4rem; padding:0.9rem 0.9rem; border-radius:0.75rem; font-size:0.94rem; }
    .result { background:#020817; border:1px solid #111827; }
    .error { background:#1f2937; color:#fecaca; border:1px solid #b91c1c; }
    .meta { font-size:0.78rem; color:#6b7280; margin-top:1.2rem; }
    code { background:#111827; padding:0.12rem 0.35rem; border-radius:0.4rem; font-size:0.75rem; }
  </style>
</head>
<body>
  <div class="wrap">
    <h1><span class="logo">🛰️</span> PWS Weather Viewer</h1>
    <p class="sub">
      Enter a Weather Underground <strong>Station ID</strong> (e.g. <code>KCASANFR70</code>) to see live conditions
      via the Weather Underground / PWS API.
    </p>

    <label for="stationId">Station ID</label>
    <input id="stationId" placeholder="Your station ID (e.g. KXXXXXXX)" />
    <button id="fetchBtn">Get Current Conditions</button>

    <div id="output"></div>

    <div class="meta">
      Backend: Rust + Axum · Deployed via Docker · Source on GitHub.
    </div>
  </div>

<script>
  const btn = document.getElementById('fetchBtn');
  const input = document.getElementById('stationId');
  const out = document.getElementById('output');

  async function fetchWeather() {
    const stationId = input.value.trim();
    if (!stationId) {
      out.innerHTML = '<div class="error">Please enter a Station ID.</div>';
      return;
    }
    out.innerHTML = '<div class="result">Loading current conditions...</div>';
    try {
      const res = await fetch(`/api/weather?station_id=${encodeURIComponent(stationId)}`);
      const data = await res.json();
      if (!res.ok || data.error) {
        out.innerHTML = `<div class="error">${data.error || 'Unknown error from API.'}</div>`;
        return;
      }

      out.innerHTML = `
        <div class="result">
          <div><strong>Station:</strong> ${data.station_id}</div>
          <div><strong>Observed:</strong> ${data.observed_at}</div>
          <div><strong>Temperature:</strong> ${data.temperature_f} °F</div>
          <div><strong>Humidity:</strong> ${data.humidity_pct}%</div>
          <div><strong>Wind:</strong> ${data.wind_mph} mph (gust ${data.wind_gust_mph} mph)</div>
          <div><strong>Pressure:</strong> ${data.pressure_in} inHg</div>
          <div><strong>Precip (last hr):</strong> ${data.precip_in_hr} in</div>
        </div>
      `;
    } catch (err) {
      out.innerHTML = '<div class="error">Request failed. Check console / network.</div>';
      console.error(err);
    }
  }

  btn.addEventListener('click', fetchWeather);
  input.addEventListener('keydown', e => { if (e.key === 'Enter') fetchWeather(); });
</script>
</body>
</html>
"#)
}

// ========== API client / DTOs ==========

#[derive(Deserialize)]
struct WeatherQuery {
    station_id: String,
}

#[derive(Deserialize)]
struct PwsApiResponse {
    observations: Vec<PwsObservation>,
}

#[derive(Deserialize)]
struct PwsObservation {
    #[serde(rename = "stationID")]
    station_id: String,
    #[serde(rename = "obsTimeLocal")]
    obs_time_local: String,
    neighborhood: Option<String>,
    imperial: Imperial,
}

#[derive(Deserialize)]
struct Imperial {
    temp: f64,
    humidity: f64,
    windSpeed: f64,
    windGust: f64,
    windDir: f64,
    pressure: f64,
    precipRate: f64,
}

#[derive(Serialize)]
struct WeatherResponse {
    station_id: String,
    observed_at: String,
    temperature_f: f64,
    humidity_pct: f64,
    wind_mph: f64,
    wind_gust_mph: f64,
    pressure_in: f64,
    precip_in_hr: f64,
    neighborhood: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn get_weather(
    State(state): State<AppState>,
    Query(query): Query<WeatherQuery>,
) -> Result<Json<WeatherResponse>, (StatusCode, Json<ErrorResponse>)> {
    if query.station_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "station_id is required".into(),
            }),
        ));
    }

    let url = format!(
        "https://api.weather.com/v2/pws/observations/current?stationId={}&format=json&units=e&apiKey={}",
        query.station_id, state.wu_api_key
    );

    let resp = match state.client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("WU request error: {:?}", e);
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to reach Weather Underground API".into(),
                }),
            ));
        }
    };

    if !resp.status().is_success() {
        let status = resp.status();
        tracing::warn!(
            "WU non-success status {} for station {}",
            status,
            query.station_id
        );
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: format!(
                    "Weather Underground API error (HTTP {}) for station {}",
                    status, query.station_id
                ),
            }),
        ));
    }

    let parsed: PwsApiResponse = match resp.json().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("WU JSON parse error: {:?}", e);
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to parse Weather Underground response".into(),
                }),
            ));
        }
    };

    let obs = match parsed.observations.into_iter().next() {
        Some(o) => o,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "No observations returned for this station.".into(),
                }),
            ))
        }
    };

    let res = WeatherResponse {
        station_id: obs.station_id,
        observed_at: obs.obs_time_local,
        temperature_f: obs.imperial.temp,
        humidity_pct: obs.imperial.humidity,
        wind_mph: obs.imperial.windSpeed,
        wind_gust_mph: obs.imperial.windGust,
        pressure_in: obs.imperial.pressure,
        precip_in_hr: obs.imperial.precipRate,
        neighborhood: obs.neighborhood,
    };

    Ok(Json(res))
}
