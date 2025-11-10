use serde_json::json;

// Import your structs from the crate:
// Adjust paths depending on how you structure modules.
use your_crate_name::{PwsApiResponse, PwsObservation, Imperial};

#[test]
fn parses_pws_response_and_maps_fields() {
    let sample = json!({
        "observations": [{
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
        }]
    });

    let s = sample.to_string();
    let parsed: PwsApiResponse = serde_json::from_str(&s).expect("should parse");

    assert_eq!(parsed.observations.len(), 1);
    let obs = &parsed.observations[0];
    assert_eq!(obs.station_id, "TESTSTATION1");
    assert_eq!(obs.imperial.temp, 72.5);
    assert_eq!(obs.imperial.humidity, 55.0);
}
