use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Json, Router,
};
use http::Method;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    client: Client,
    wu_api_key: String,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Require a real WU_API_KEY for runtime
    let wu_api_key =
        env::var("WU_API_KEY").expect("WU_API_KEY environment variable must be set");

    let state = AppState {
        client: Client::new(),
        wu_api_key,
    };

    // CORS: allow GET from any origin (tighten later if desired)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/api/weather", get(get_weather))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>PWS Weather Viewer</title>
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    body { background:#050816; color:#e5e7eb; font-family: system-ui,-apple-system,BlinkMacSystemFont,sans-serif; margin:0; padding:0; }
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
      Enter a Weather Underground <strong>Station ID</strong> (e.g. <code>KCASANFR70</code>) to see live conditions.
    </p>

    <label for="stationId">Station ID</label>
    <input id="stationId" placeholder="Your station ID (e.g. KXXXXXXX)" />
    <button id="fetchBtn">Get Current Conditions</button>

    <div id="output"></div>

    <div class="meta">
      Backend: Rust + Axum · Dockerized · CI/CD via GitHub Actions.
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
          <div><strong>Wind:</strong> ${data.wind_mph} mph (gust ${data.wind_gust_mph} mph, dir ${data.wind_dir_deg}°)</div>
          <div><strong>Pressure:</strong> ${data.pressure_in} inHg</div>
          <div><strong>Precip (last hr):</strong> ${data.precip_in_hr} in</div>
          ${data.neighborhood ? `<div><strong>Neighborhood:</strong> ${data.neighborhood}</div>` : ''}
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
"#,
    )
}

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
    #[serde(rename = "temp")]
    temp: f64,
    #[serde(rename = "humidity")]
    humidity: f64,
    #[serde(rename = "windSpeed")]
    wind_speed: f64,
    #[serde(rename = "windGust")]
    wind_gust: f64,
    #[serde(rename = "windDir")]
    wind_dir: f64,
    #[serde(rename = "pressure")]
    pressure: f64,
    #[serde(rename = "precipRate")]
    precip_rate: f64,
}

#[derive(Serialize)]
struct WeatherResponse {
    station_id: String,
    observed_at: String,
    temperature_f: f64,
    humidity_pct: f64,
    wind_mph: f64,
    wind_gust_mph: f64,
    wind_dir_deg: f64,
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
        "https://api.weather.com/v2/pws/observations/current\
         ?stationId={}&format=json&units=e&apiKey={}",
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

    let status = resp.status();

    if !status.is_success() {
        let code = status.as_u16();
        let msg = format!(
            "Weather Underground API error (HTTP {}) for station {}",
            code, query.station_id
        );
        tracing::warn!("{msg}");
        return Err((StatusCode::BAD_GATEWAY, Json(ErrorResponse { error: msg })));
    }

    let body = match resp.text().await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("WU body read error: {:?}", e);
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Failed to read Weather Underground response body".into(),
                }),
            ));
        }
    };

    let trimmed = body.trim();

    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("data expired")
        || trimmed.eq_ignore_ascii_case("no data")
    {
        tracing::warn!(
            "No current observation data for station {} (empty/Data Expired). Raw: {:?}",
            query.station_id,
            trimmed
        );
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!(
                    "No current observation data available for station {}.",
                    query.station_id
                ),
            }),
        ));
    }

    let parsed: PwsApiResponse = match serde_json::from_str(trimmed) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(
                "WU JSON parse error for station {}: {:?}. Raw body: {}",
                query.station_id,
                e,
                trimmed
            );
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Weather Underground returned an unexpected response format."
                        .into(),
                }),
            ));
        }
    };

    let obs = match parsed.observations.into_iter().next() {
        Some(o) => o,
        None => {
            tracing::warn!(
                "No observations array entries for station {}. Raw body: {}",
                query.station_id,
                trimmed
            );
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!(
                        "No observations returned for station {}.",
                        query.station_id
                    ),
                }),
            ));
        }
    };

    let res = WeatherResponse {
        station_id: obs.station_id,
        observed_at: obs.obs_time_local,
        temperature_f: obs.imperial.temp,
        humidity_pct: obs.imperial.humidity,
        wind_mph: obs.imperial.wind_speed,
        wind_gust_mph: obs.imperial.wind_gust,
        wind_dir_deg: obs.imperial.wind_dir,
        pressure_in: obs.imperial.pressure,
        precip_in_hr: obs.imperial.precip_rate,
        neighborhood: obs.neighborhood,
    };

    Ok(Json(res))
}

