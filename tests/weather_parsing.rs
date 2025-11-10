use serde::Deserialize;
use serde_json;

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

#[test]
fn parses_pws_response_and_maps_fields() {
    let sample = r#"
    {
      "observations": [
        {
          "stationID": "TESTSTATION1",
          "obsTimeLocal": "2025-11-09 12:34:56",
          "neighborhood": "Test Neighborhood",
          "imperial": {
            "temp": 72.5,
            "humidity": 55.0,
            "windSpeed": 3.1,
            "windGust": 5.0,
            "windDir": 180.0,
            "pressure": 29.92,
            "precipRate": 0.01
          }
        }
      ]
    }
    "#;

    let parsed: PwsApiResponse = serde_json::from_str(sample).expect("should parse JSON");
    assert_eq!(parsed.observations.len(), 1);

    let obs = &parsed.observations[0];
    assert_eq!(obs.station_id, "TESTSTATION1");
    assert_eq!(obs.obs_time_local, "2025-11-09 12:34:56");
    assert_eq!(obs.neighborhood.as_deref(), Some("Test Neighborhood"));

    assert!((obs.imperial.temp - 72.5).abs() < f64::EPSILON);
    assert!((obs.imperial.humidity - 55.0).abs() < f64::EPSILON);
    assert!((obs.imperial.wind_speed - 3.1).abs() < f64::EPSILON);
    assert!((obs.imperial.wind_gust - 5.0).abs() < f64::EPSILON);
    assert!((obs.imperial.wind_dir - 180.0).abs() < f64::EPSILON);
    assert!((obs.imperial.pressure - 29.92).abs() < f64::EPSILON);
    assert!((obs.imperial.precip_rate - 0.01).abs() < f64::EPSILON);
}

